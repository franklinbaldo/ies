//! Leitura privada única: o par de leitura de [`super::gravar`]. Lê um
//! arquivo `0600`, regular, não vazio e até um limite, sem seguir link no
//! nome final, como texto UTF-8. Serve a sessão do transporte, as
//! credenciais do login e o HTML privado. Cada chamador embrulha o
//! [`ErroArquivoPrivado`] uma vez, com o nome do arquivo que lia.
use std::fs;
use std::io::{ErrorKind, Read};
use std::path::Path;
use std::str::Utf8Error;

/// Por que o arquivo privado não foi lido: um fato por variante, sem
/// caminho nem conteúdo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroArquivoPrivado {
    /// Nada existe no caminho (`NotFound`).
    Ausente,
    /// Os metadados do caminho ou do arquivo aberto não foram lidos (o
    /// motivo do sistema, que não é `NotFound`).
    MetadadosIlegiveis(ErrorKind),
    /// O nome final é um link simbólico.
    LinkSimbolico,
    /// O caminho existe e não é arquivo regular.
    NaoRegular,
    /// O modo dá alguma permissão a grupo ou a outros.
    #[cfg(unix)]
    PermissoesAbertas,
    Vazio,
    AcimaDoLimite,
    /// O sistema recusou abrir o arquivo (o motivo do sistema).
    AberturaRecusada(ErrorKind),
    /// O conteúdo não foi lido até o fim (o motivo do sistema).
    ConteudoIlegivel(ErrorKind),
    /// O conteúdo não é UTF-8 (só a posição do erro, sem o texto).
    NaoUtf8(#[source] Utf8Error),
}

pub(crate) fn ler(path: &Path, limite: u64) -> Result<String, ErroArquivoPrivado> {
    // A checagem do caminho fornece erros específicos; a decisão de
    // segurança usa o descritor aberto, não metadados sujeitos a TOCTOU.
    let observado = fs::symlink_metadata(path).map_err(|erro| {
        if erro.kind() == ErrorKind::NotFound {
            ErroArquivoPrivado::Ausente
        } else {
            ErroArquivoPrivado::MetadadosIlegiveis(erro.kind())
        }
    })?;
    if observado.file_type().is_symlink() {
        return Err(ErroArquivoPrivado::LinkSimbolico);
    }
    if !observado.is_file() {
        return Err(ErroArquivoPrivado::NaoRegular);
    }

    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // O_NOFOLLOW impede a troca do último componente por symlink.
        // O_NONBLOCK impede que a troca por FIFO bloqueie a leitura.
        options.custom_flags(0o400_000 | 0o4_000);
    }
    let arquivo = options.open(path).map_err(|erro| {
        if cfg!(target_os = "linux") && erro.raw_os_error() == Some(40) {
            ErroArquivoPrivado::LinkSimbolico
        } else {
            ErroArquivoPrivado::AberturaRecusada(erro.kind())
        }
    })?;
    let metadata = arquivo
        .metadata()
        .map_err(|erro| ErroArquivoPrivado::MetadadosIlegiveis(erro.kind()))?;
    if !metadata.is_file() {
        return Err(ErroArquivoPrivado::NaoRegular);
    }
    if metadata.len() == 0 {
        return Err(ErroArquivoPrivado::Vazio);
    }
    if metadata.len() > limite {
        return Err(ErroArquivoPrivado::AcimaDoLimite);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(ErroArquivoPrivado::PermissoesAbertas);
        }
    }
    let mut bytes = Vec::new();
    arquivo
        .take(limite + 1)
        .read_to_end(&mut bytes)
        .map_err(|erro| ErroArquivoPrivado::ConteudoIlegivel(erro.kind()))?;
    if u64::try_from(bytes.len()).is_ok_and(|tamanho| tamanho > limite) {
        return Err(ErroArquivoPrivado::AcimaDoLimite);
    }
    if bytes.is_empty() {
        return Err(ErroArquivoPrivado::Vazio);
    }
    String::from_utf8(bytes).map_err(|erro| ErroArquivoPrivado::NaoUtf8(erro.utf8_error()))
}
