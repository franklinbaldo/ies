//! Cache privado e idempotente de documentos (#236).
//! Nunca guarda URLs assinadas ou material de sessão.
use crate::arquivo_privado::{self, Politica};
use crate::foto_documento::sha256;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Registro {
    pub(crate) caminho: PathBuf,
    pub(crate) versao: Option<u32>,
    pub(crate) sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FalhaCache {
    Diretorio,
    Gravacao,
    ArquivoDivergente,
    ConteudoInseguro,
}

impl FalhaCache {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Diretorio => "diretorio_cache_documento_inseguro",
            Self::Gravacao => "gravacao_cache_documento_falhou",
            Self::ArquivoDivergente => "cache_documento_local_divergente",
            Self::ConteudoInseguro => "conteudo_com_link_assinado",
        }
    }
    pub(crate) fn mensagem(self) -> &'static str {
        match self {
            Self::Diretorio => "diretório do cache local indisponível ou inseguro",
            Self::Gravacao => "não foi possível gravar o cache privado",
            Self::ArquivoDivergente => "cópia existente no cache diverge do SHA-256",
            Self::ConteudoInseguro => "conteúdo com link assinado não pode ir para o cache",
        }
    }
}

/// A flag prevalece sobre o ambiente; só o valor literal 0 desativa.
pub(crate) fn habilitado(sem_cache: bool, ambiente: Option<&str>) -> bool {
    !sem_cache && ambiente != Some("0")
}

pub(crate) fn habilitado_ambiente(sem_cache: bool) -> bool {
    habilitado(sem_cache, std::env::var("IES_CACHE_DOCUMENTOS").ok().as_deref())
}

pub(crate) fn raiz() -> Result<PathBuf, FalhaCache> {
    if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME") {
        let pasta = PathBuf::from(xdg);
        if pasta.is_absolute() {
            return Ok(pasta);
        }
    }
    let home = std::env::var_os("HOME").ok_or(FalhaCache::Diretorio)?;
    Ok(PathBuf::from(home).join(".cache"))
}

pub(crate) fn nome(versao: Option<u32>, hash: &str) -> String {
    nome_ext(versao, hash, "html")
}

pub(crate) fn nome_ext(versao: Option<u32>, hash: &str, extensao: &str) -> String {
    let prefixo = versao.map_or_else(|| "x".to_owned(), |n| format!("v{n}"));
    format!("{prefixo}-{}.{}", &hash[..hash.len().min(16)], extensao)
}

pub(crate) fn caminho_em(
    raiz: &Path, host: &str, numero: &str, versao: Option<u32>, hash: &str, extensao: &str,
) -> PathBuf {
    raiz.join("ies").join(host).join(numero).join(nome_ext(versao, hash, extensao))
}

fn conferir_identidade(host: &str, numero: &str, extensao: &str) -> Result<(), FalhaCache> {
    if !crate::instalacao::host_valido(host)
        || numero.is_empty()
        || !numero.bytes().all(|b| b.is_ascii_digit())
        || !matches!(extensao, "html" | "pdf" | "txt")
    {
        return Err(FalhaCache::Diretorio);
    }
    Ok(())
}

/// Um destino existente só é no-op se regular, privado e byte a byte igual.
pub(crate) fn conferir_existente(destino: &Path, bytes: &[u8]) -> Result<(), FalhaCache> {
    let info = fs::symlink_metadata(destino).map_err(|_| FalhaCache::Gravacao)?;
    if !info.file_type().is_file() {
        return Err(FalhaCache::ArquivoDivergente);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if info.permissions().mode() & 0o077 != 0 {
            return Err(FalhaCache::ArquivoDivergente);
        }
    }
    let existente = fs::read(destino).map_err(|_| FalhaCache::Gravacao)?;
    if existente != bytes {
        return Err(FalhaCache::ArquivoDivergente);
    }
    Ok(())
}

/// Os bytes do HTML não podem carregar credenciais em URL assinada.
pub(crate) fn gravar_em(
    raiz: &Path, host: &str, numero: &str, versao: Option<u32>, extensao: &str,
    bytes: &[u8],
) -> Result<PathBuf, FalhaCache> {
    conferir_identidade(host, numero, extensao)?;
    if extensao == "html" && bytes.windows(b"infra_hash".len()).any(|b| b == b"infra_hash") {
        return Err(FalhaCache::ConteudoInseguro);
    }
    let hash = sha256(bytes);
    let destino = caminho_em(raiz, host, numero, versao, &hash, extensao);
    let politica = Politica::exclusiva().diretorio_do_ies().criando_diretorios();
    match fs::symlink_metadata(&destino) {
        Ok(_) => conferir_existente(&destino, bytes)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            match arquivo_privado::gravar(&destino, bytes, politica) {
                Ok(()) => {},
                Err(arquivo_privado::FalhaDaGravacao::DestinoJaExiste) => {
                    conferir_existente(&destino, bytes)?;
                }
                Err(_) => return Err(FalhaCache::Gravacao),
            }
        }
        Err(_) => return Err(FalhaCache::Gravacao),
    }
    Ok(destino)
}

pub(crate) fn gravar(
    host: &str, numero: &str, versao: Option<u32>, extensao: &str,
    bytes: &[u8],
) -> Result<Registro, FalhaCache> {
    let hash = sha256(bytes);
    let caminho = gravar_em(&raiz()?, host, numero, versao, extensao, bytes)?;
    Ok(Registro { caminho, versao, sha256: hash })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nome_conhecido_e_desconhecido() {
        assert_eq!(nome(Some(2), "aabbccddeeff001122"), "v2-aabbccddeeff0011.html");
        assert_eq!(nome(None, "aabbccddeeff001122"), "x-aabbccddeeff0011.html");
    }

    #[test]
    fn cache_idempotente_em_xdg_temporario() {
        let home = std::env::temp_dir().join(format!("ies-cache-red-{}", std::process::id()));
        let alvo = caminho_em(&home, "sei.exemplo", "0011519", Some(2), "aabbccddeeff001122", "html");
        assert_eq!(alvo, home.join("ies/sei.exemplo/0011519/v2-aabbccddeeff0011.html"));
        let bytes = b"<html><body>teste</body></html>";
        let alvo = gravar_em(&home, "sei.exemplo", "0011519", Some(2), "html", bytes).unwrap();
        let primeira = std::fs::metadata(&alvo).unwrap().modified().unwrap();
        let segunda = gravar_em(&home, "sei.exemplo", "0011519", Some(2), "html", bytes).unwrap();
        assert_eq!(alvo, segunda);
        assert_eq!(std::fs::metadata(&alvo).unwrap().modified().unwrap(), primeira);
        std::fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn sem_cache_nao_escreve() {
        assert!(!habilitado(true, None));
        assert!(!habilitado(false, Some("0")));
        assert!(habilitado(false, None));
    }

    #[test]
    fn conteudo_assinado_nao_e_persistido() {
        let home = std::env::temp_dir().join(format!("ies-cache-seguranca-{}", std::process::id()));
        assert_eq!(gravar_em(&home, "sei.exemplo", "1", None, "html", b"infra_hash=x"),
                   Err(FalhaCache::ConteudoInseguro));
        assert!(!home.exists());
    }
}
