//! Leitura binária privada de cópias locais, independente de sessão SEI.
use crate::copia_documento::ErroCopia;
use std::fs;
use std::io::Read;
use std::path::Path;

const LIMITE_COPIA: u64 = 8_000_000;

/// Shim inicial: os testes RED exigem validação do descritor, não só do caminho.
pub(crate) fn ler_regular(alvo: &Path, ausente: ErroCopia) -> Result<Vec<u8>, ErroCopia> {
    let dados = fs::symlink_metadata(alvo).map_err(|_| ausente)?;
    if !dados.is_file() || dados.file_type().is_symlink() || dados.len() > LIMITE_COPIA {
        return Err(ErroCopia::DestinoInseguro);
    }
    let arquivo = fs::File::open(alvo).map_err(|_| ErroCopia::LeituraFalhou)?;
    let mut bytes = Vec::new();
    arquivo
        .take(LIMITE_COPIA + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ErroCopia::LeituraFalhou)?;
    if bytes.len() as u64 > LIMITE_COPIA {
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
