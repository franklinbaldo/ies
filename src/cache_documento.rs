//! Cache privado e idempotente de documentos (#236).
//! Nunca guarda URLs assinadas ou material de sessão.
use crate::arquivo_privado::{self, FalhaDaGravacao, Politica};
use crate::foto_documento::sha256;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Registro {
    pub(crate) caminho: PathBuf,
    pub(crate) versao: Option<u32>,
    pub(crate) sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaCache {
    Diretorio,
    Gravacao,
    /// Leitura ou escrita de um arquivo do cache falhou; guarda o tipo de E/S.
    ArquivoDoCacheInacessivel(std::io::ErrorKind),
    /// A gravação privada do arquivo do cache foi recusada.
    GravacaoPrivadaFalhou(#[source] FalhaDaGravacao),
    ArquivoDivergente,
    ConteudoInseguro,
}

/// A flag prevalece sobre o ambiente; só o valor literal 0 desativa.
pub(crate) fn habilitado(sem_cache: bool, ambiente: Option<&str>) -> bool {
    !sem_cache && ambiente != Some("0")
}

pub(crate) fn habilitado_ambiente(sem_cache: bool) -> bool {
    habilitado(
        sem_cache,
        std::env::var("IES_CACHE_DOCUMENTOS").ok().as_deref(),
    )
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

#[cfg(test)]
pub(crate) fn nome(versao: Option<u32>, hash: &str) -> String {
    nome_ext(versao, hash, "html")
}

pub(crate) fn nome_ext(versao: Option<u32>, hash: &str, extensao: &str) -> String {
    let prefixo = versao.map_or_else(|| "x".to_owned(), |n| format!("v{n}"));
    let curto: String = hash.chars().take(16).collect();
    format!("{prefixo}-{curto}.{extensao}")
}

pub(crate) fn caminho_em(
    raiz: &Path,
    host: &str,
    numero: &str,
    versao: Option<u32>,
    hash: &str,
    extensao: &str,
) -> PathBuf {
    raiz.join("ies")
        .join(host)
        .join(numero)
        .join(nome_ext(versao, hash, extensao))
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
    let info = fs::symlink_metadata(destino)
        .map_err(|erro| FalhaCache::ArquivoDoCacheInacessivel(erro.kind()))?;
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
    let existente =
        fs::read(destino).map_err(|erro| FalhaCache::ArquivoDoCacheInacessivel(erro.kind()))?;
    if existente != bytes {
        return Err(FalhaCache::ArquivoDivergente);
    }
    Ok(())
}

/// Os bytes do HTML não podem carregar credenciais em URL assinada.
pub(crate) fn gravar_em(
    raiz: &Path,
    host: &str,
    numero: &str,
    versao: Option<u32>,
    extensao: &str,
    bytes: &[u8],
) -> Result<PathBuf, FalhaCache> {
    conferir_identidade(host, numero, extensao)?;
    if extensao == "html"
        && bytes
            .windows(b"infra_hash".len())
            .any(|b| b == b"infra_hash")
    {
        return Err(FalhaCache::ConteudoInseguro);
    }
    let hash = sha256(bytes);
    let destino = caminho_em(raiz, host, numero, versao, &hash, extensao);
    let politica = Politica::exclusiva()
        .diretorio_do_ies()
        .criando_diretorios();
    match fs::symlink_metadata(&destino) {
        Ok(_) => conferir_existente(&destino, bytes)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            match arquivo_privado::gravar(&destino, bytes, politica) {
                Ok(()) => {}
                Err(arquivo_privado::FalhaDaGravacao::DestinoJaExiste) => {
                    conferir_existente(&destino, bytes)?;
                }
                Err(falha) => return Err(FalhaCache::GravacaoPrivadaFalhou(falha)),
            }
        }
        Err(erro) => return Err(FalhaCache::ArquivoDoCacheInacessivel(erro.kind())),
    }
    Ok(destino)
}

pub(crate) fn gravar(
    host: &str,
    numero: &str,
    versao: Option<u32>,
    extensao: &str,
    bytes: &[u8],
) -> Result<Registro, FalhaCache> {
    let hash = sha256(bytes);
    let caminho = gravar_em(&raiz()?, host, numero, versao, extensao, bytes)?;
    Ok(Registro {
        caminho,
        versao,
        sha256: hash,
    })
}

fn hash_de_arquivo(caminho: &Path) -> Result<String, FalhaCache> {
    use sha2::{Digest, Sha256};
    let info = fs::symlink_metadata(caminho)
        .map_err(|erro| FalhaCache::ArquivoDoCacheInacessivel(erro.kind()))?;
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
    let mut arquivo = fs::File::open(caminho)
        .map_err(|erro| FalhaCache::ArquivoDoCacheInacessivel(erro.kind()))?;
    let mut buffer = [0u8; 16 * 1024];
    let mut sha = Sha256::new();
    loop {
        let lidos = arquivo
            .read(&mut buffer)
            .map_err(|erro| FalhaCache::ArquivoDoCacheInacessivel(erro.kind()))?;
        if lidos == 0 {
            break;
        }
        sha.update(buffer.split_at(lidos.min(buffer.len())).0);
    }
    Ok(hex::encode(sha.finalize()))
}

/// Publica o parcial binário já integralmente conferido; seu Drop o elimina.
/// A colisão do prefixo de 16 hex é conferida pelo hash SHA-256 completo.
pub(crate) fn publicar_parcial(
    parcial: &Path,
    raiz: &Path,
    host: &str,
    numero: &str,
    hash: &str,
    extensao: &str,
) -> Result<Registro, FalhaCache> {
    conferir_identidade(host, numero, extensao)?;
    let caminho = caminho_em(raiz, host, numero, None, hash, extensao);
    match fs::hard_link(parcial, &caminho) {
        Ok(()) => {}
        Err(erro) if erro.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err(FalhaCache::Gravacao),
    }
    if hash_de_arquivo(&caminho)? != hash {
        return Err(FalhaCache::ArquivoDivergente);
    }
    Ok(Registro {
        caminho,
        versao: None,
        sha256: hash.to_owned(),
    })
}

/// A cópia explícita tem inode próprio, para a edição local não alterar
/// o cache privado. Gravação exclusiva em fluxo, sem seguir symlink final.
pub(crate) fn copiar_para_saida(
    origem: &Path,
    destino: &Path,
    hash: &str,
) -> Result<bool, FalhaCache> {
    use crate::arquivo_privado::{FalhaDaGravacao, Parcial};
    if fs::symlink_metadata(destino).is_ok() {
        if hash_de_arquivo(destino)? == hash {
            return Ok(false);
        }
        return Err(FalhaCache::ArquivoDivergente);
    }
    // A cópia do usuário deve ter inode próprio: sua edição não pode adulterar o cache.
    let mut arquivo = fs::File::open(origem)
        .map_err(|erro| FalhaCache::ArquivoDoCacheInacessivel(erro.kind()))?;
    let mut parcial = Parcial::abrir(destino, Politica::exclusiva().diretorio_do_usuario())
        .map_err(FalhaCache::GravacaoPrivadaFalhou)?;
    let mut buffer = [0u8; 16 * 1024];
    loop {
        let lidos = arquivo
            .read(&mut buffer)
            .map_err(|erro| FalhaCache::ArquivoDoCacheInacessivel(erro.kind()))?;
        if lidos == 0 {
            break;
        }
        parcial
            .escrever(buffer.split_at(lidos.min(buffer.len())).0)
            .map_err(FalhaCache::GravacaoPrivadaFalhou)?;
    }
    match parcial.confirmar() {
        Ok(()) => Ok(true),
        Err(FalhaDaGravacao::DestinoJaExiste) if hash_de_arquivo(destino)? == hash => Ok(false),
        Err(_) => Err(FalhaCache::Gravacao),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nome_conhecido_e_desconhecido() {
        assert_eq!(
            nome(Some(2), "aabbccddeeff001122"),
            "v2-aabbccddeeff0011.html"
        );
        assert_eq!(nome(None, "aabbccddeeff001122"), "x-aabbccddeeff0011.html");
    }

    #[test]
    fn cache_idempotente_em_xdg_temporario() {
        let home = std::env::temp_dir().join(format!("ies-cache-red-{}", std::process::id()));
        let alvo = caminho_em(
            &home,
            "sei.exemplo",
            "0011519",
            Some(2),
            "aabbccddeeff001122",
            "html",
        );
        assert_eq!(
            alvo,
            home.join("ies/sei.exemplo/0011519/v2-aabbccddeeff0011.html")
        );
        let bytes = b"<html><body>teste</body></html>";
        let alvo = gravar_em(&home, "sei.exemplo", "0011519", Some(2), "html", bytes).unwrap();
        let primeira = std::fs::metadata(&alvo).unwrap().modified().unwrap();
        let segunda = gravar_em(&home, "sei.exemplo", "0011519", Some(2), "html", bytes).unwrap();
        assert_eq!(alvo, segunda);
        assert_eq!(
            std::fs::metadata(&alvo).unwrap().modified().unwrap(),
            primeira
        );
        std::fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn sem_cache_nao_escreve() {
        assert!(!habilitado(true, None));
        assert!(!habilitado(false, Some("0")));
        assert!(habilitado(false, None));
    }

    #[test]
    fn externo_pdf_em_fluxo_e_copia_opcional() {
        use crate::arquivo_privado::{Parcial, Politica};
        let raiz = std::env::temp_dir().join(format!(
            "ies-cache-anexo-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("teste"),
        ));
        let destino_parcial = raiz.join("ies/sei.exemplo/0007/0007.cache.pdf");
        let mut parcial = Parcial::abrir(
            &destino_parcial,
            Politica::exclusiva()
                .diretorio_do_ies()
                .criando_diretorios(),
        )
        .unwrap();
        let bytes = b"%PDF-1.7\nTESTE";
        parcial.escrever(bytes).unwrap();
        parcial.sincronizar().unwrap();
        let hash = sha256(bytes);
        let registro = publicar_parcial(
            parcial.temporario(),
            &raiz,
            "sei.exemplo",
            "0007",
            &hash,
            "pdf",
        )
        .unwrap();
        assert_eq!(registro.versao, None);
        assert!(registro.caminho.ends_with(format!("x-{}.pdf", &hash[..16])));
        drop(parcial);
        assert_eq!(fs::read(&registro.caminho).unwrap(), bytes);
        let pasta = raiz.join("saida");
        fs::create_dir(&pasta).unwrap();
        let copia = pasta.join("0007.pdf");
        assert_eq!(
            copiar_para_saida(&registro.caminho, &copia, &hash),
            Ok(true)
        );
        assert_eq!(
            copiar_para_saida(&registro.caminho, &copia, &hash),
            Ok(false)
        );
        assert_eq!(fs::read(&copia).unwrap(), bytes);
        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn diferentes_versoes_nao_sobrescrevem() {
        let raiz = std::env::temp_dir().join(format!("ies-cache-versoes-{}", std::process::id()));
        let um = gravar_em(&raiz, "sei.exemplo", "001", Some(1), "html", b"<p>um</p>").unwrap();
        let dois = gravar_em(&raiz, "sei.exemplo", "001", Some(2), "html", b"<p>dois</p>").unwrap();
        let desconhecida =
            gravar_em(&raiz, "sei.exemplo", "001", None, "html", b"<p>x</p>").unwrap();
        assert_ne!(um, dois);
        assert_ne!(desconhecida, um);
        assert!(
            desconhecida
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("x-")
        );
        fs::remove_dir_all(raiz).unwrap();
    }

    // Apoio técnico ao BDD-60-03: a foto PDF editável do usuário não
    // pode compartilhar inode com o cache privado de conteúdo observado.
    #[test]
    fn bdd_60_03_copia_pdf_editada_nao_adultera_cache_privado() {
        let raiz = std::env::temp_dir().join(format!(
            "ies-cache-pdf-independente-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let pdf = b"%PDF-1.7 original";
        let origem = gravar_em(&raiz, "sei.exemplo", "0007", None, "pdf", pdf).unwrap();
        let saida = raiz.join("saida");
        fs::create_dir(&saida).unwrap();
        let destino = saida.join("0007.pdf");
        let hash = sha256(pdf);
        assert_eq!(copiar_para_saida(&origem, &destino, &hash), Ok(true));
        fs::write(&destino, b"%PDF-1.7 edicao local").unwrap();
        assert_eq!(
            fs::read(&origem).unwrap(),
            pdf,
            "cache não pode ser editado pela foto"
        );
        assert_eq!(
            copiar_para_saida(&origem, &destino, &hash),
            Err(FalhaCache::ArquivoDivergente),
            "a edição local não pode ser sobrescrita"
        );
        fs::remove_dir_all(raiz).unwrap();
    }

    // Contraprova da #719: a consulta do caminho falhou no SO, não foi
    // uma recusa genérica da gravação. Não grava nem acessa o SEI.
    #[cfg(unix)]
    #[test]
    fn consulta_do_cache_preserva_not_a_directory() {
        let raiz = std::env::temp_dir().join(format!(
            "ies-cache-causa-diretorio-{}",
            std::process::id()
        ));
        fs::write(&raiz, b"arquivo, nao diretorio").unwrap();
        let causa = gravar_em(&raiz, "sei.exemplo", "001", None, "html", b"teste")
            .unwrap_err();
        assert_eq!(
            causa,
            FalhaCache::ArquivoDoCacheInacessivel(std::io::ErrorKind::NotADirectory)
        );
        assert_eq!(fs::read(&raiz).unwrap(), b"arquivo, nao diretorio");
        fs::remove_file(raiz).unwrap();
    }

    // O erro tipado da gravação privada já existe; o cache não deve
    // substituí-lo por `Gravacao` quando o diretório é inseguro.
    #[cfg(unix)]
    #[test]
    fn cache_preserva_causa_da_gravacao_privada() {
        use std::os::unix::fs::PermissionsExt;
        let raiz = std::env::temp_dir().join(format!(
            "ies-cache-causa-permissoes-{}",
            std::process::id()
        ));
        fs::create_dir(&raiz).unwrap();
        fs::set_permissions(&raiz, fs::Permissions::from_mode(0o770)).unwrap();
        let causa = gravar_em(&raiz, "sei.exemplo", "001", None, "html", b"teste")
            .unwrap_err();
        assert_eq!(
            causa,
            FalhaCache::GravacaoPrivadaFalhou(
                FalhaDaGravacao::DiretorioGravavelPorOutros
            )
        );
        fs::set_permissions(&raiz, fs::Permissions::from_mode(0o700)).unwrap();
        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn conteudo_assinado_nao_e_persistido() {
        let home = std::env::temp_dir().join(format!("ies-cache-seguranca-{}", std::process::id()));
        assert_eq!(
            gravar_em(&home, "sei.exemplo", "1", None, "html", b"infra_hash=x"),
            Err(FalhaCache::ConteudoInseguro)
        );
        assert!(!home.exists());
    }
}
