//! Leitura binária privada de cópias locais, independente de sessão SEI.
use crate::copia_documento::ErroCopia;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::Path;

const LIMITE_COPIA: u64 = 8_000_000;

/// Confere arquivo regular privado no descritor efetivamente lido, sem seguir link final.
pub(crate) fn ler_regular(alvo: &Path, ausente: ErroCopia) -> Result<Vec<u8>, ErroCopia> {
    // A observação do caminho só determina o erro legível. A decisão
    // de segurança deve usar o mesmo descritor que vai fornecer os bytes.
    let observado = fs::symlink_metadata(alvo).map_err(|_| ausente)?;
    if observado.file_type().is_symlink() {
        return Err(ErroCopia::LinkSimbolico);
    }
    if !observado.is_file() {
        return Err(ErroCopia::DestinoInseguro);
    }

    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // O_NOFOLLOW impede troca do último componente por link.
        // O_NONBLOCK impede bloquear numa FIFO colocada no caminho.
        options.custom_flags(0o400_000 | 0o4_000);
    }
    let arquivo = options.open(alvo).map_err(|erro| {
        if cfg!(target_os = "linux") && erro.raw_os_error() == Some(40) {
            ErroCopia::LinkSimbolico
        } else if erro.kind() == std::io::ErrorKind::NotFound {
            ausente
        } else {
            ErroCopia::LeituraFalhou
        }
    })?;
    let dados = arquivo.metadata().map_err(|_| ErroCopia::LeituraFalhou)?;
    if !dados.is_file() || dados.len() > LIMITE_COPIA {
        return Err(ErroCopia::DestinoInseguro);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if dados.permissions().mode() & 0o077 != 0 {
            return Err(ErroCopia::PermissoesInseguras);
        }
    }

    let mut bytes = Vec::new();
    arquivo
        .take(LIMITE_COPIA + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ErroCopia::LeituraFalhou)?;
    if bytes.len() > 8_000_000 {
        return Err(ErroCopia::DestinoInseguro);
    }
    Ok(bytes)
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn pasta() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ies-copia-privada-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    #[cfg(unix)]
    #[test]
    fn recusa_copia_com_permissoes_de_outros_usuarios() {
        use std::os::unix::fs::PermissionsExt;
        let pasta = pasta();
        let arquivo = pasta.join("001.v2.html");
        fs::write(&arquivo, b"texto de teste").unwrap();
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            ler_regular(&arquivo, ErroCopia::ArquivoAusente),
            Err(ErroCopia::PermissoesInseguras)
        );
        fs::remove_dir_all(pasta).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn recusa_link_simbolico_mesmo_para_arquivo_regular_privado() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let pasta = pasta();
        let original = pasta.join("original");
        let atalho = pasta.join("atalho");
        fs::write(&original, b"segredo demonstrativo").unwrap();
        fs::set_permissions(&original, fs::Permissions::from_mode(0o600)).unwrap();
        symlink(&original, &atalho).unwrap();
        assert_eq!(
            ler_regular(&atalho, ErroCopia::ArquivoAusente),
            Err(ErroCopia::LinkSimbolico)
        );
        fs::remove_dir_all(pasta).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn recusa_arquivo_acima_de_oito_megabytes_antes_da_leitura() {
        use std::os::unix::fs::PermissionsExt;
        let pasta = pasta();
        let arquivo = pasta.join("001.v2.html");
        let file = fs::File::create(&arquivo).unwrap();
        file.set_len(8_000_001).unwrap();
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            ler_regular(&arquivo, ErroCopia::ArquivoAusente),
            Err(ErroCopia::DestinoInseguro)
        );
        fs::remove_dir_all(pasta).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn copia_regular_privada_preserva_bytes_sem_forcar_utf8() {
        use std::os::unix::fs::PermissionsExt;
        let pasta = pasta();
        let arquivo = pasta.join("001.v2.html");
        let corpo = [0xff, b'\n', b'<', b'p', b'>'];
        fs::write(&arquivo, corpo).unwrap();
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            ler_regular(&arquivo, ErroCopia::ArquivoAusente),
            Ok(corpo.to_vec())
        );
        fs::remove_dir_all(pasta).unwrap();
    }
}
