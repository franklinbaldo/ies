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
#[path = "cache_documento/tests.rs"]
mod tests;
