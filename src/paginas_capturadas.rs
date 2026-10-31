//! Complementos privados de páginas já observadas, sem POST nem coleta.
use crate::transporte::FalhaTransporte;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FalhaPaginaCapturada {
    Arquivo(FalhaTransporte),
    QuantidadeAcimaDoLimite,
    CaminhosRepetidos,
    BytesAcimaDoLimite,
}
impl FalhaPaginaCapturada {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Arquivo(erro) => erro.subcodigo(),
            Self::QuantidadeAcimaDoLimite => "paginas_privadas_acima_do_limite",
            Self::CaminhosRepetidos => "arquivos_de_pagina_repetidos",
            Self::BytesAcimaDoLimite => "bytes_de_paginas_privadas_acima_do_limite",
        }
    }
    pub(crate) fn mensagem(self) -> &'static str {
        match self {
            Self::Arquivo(erro) => erro.mensagem(),
            Self::QuantidadeAcimaDoLimite => "mais de 199 páginas privadas complementares",
            Self::CaminhosRepetidos => "o mesmo arquivo foi indicado mais de uma vez",
            Self::BytesAcimaDoLimite => "o conteúdo total das páginas supera o limite",
        }
    }
}
/// Shim do contrato; testes RED devem reprovar uma lista de caminhos válidos.
pub(crate) fn conferir_caminhos(_caminhos: &[PathBuf]) -> Result<(), FalhaPaginaCapturada> {
    Err(FalhaPaginaCapturada::QuantidadeAcimaDoLimite)
}

/// Shim de leitura, sem rede. A implementação fará leitura privada com limite.
pub(crate) fn ler_complementos(
    _caminhos: &[PathBuf],
    _bytes_pagina_inicial: usize,
) -> Result<Vec<String>, FalhaPaginaCapturada> {
    Err(FalhaPaginaCapturada::QuantidadeAcimaDoLimite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn pasta() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ies-paginas-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn uma_pagina_privada_validada_e_reutilizada_sem_rede() {
        let dir = pasta();
        let arquivo = dir.join("segunda.html");
        fs::write(&arquivo, "<html>SEGUNDA PÁGINA</html>").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert_eq!(conferir_caminhos(&[arquivo.clone()]), Ok(()));
        assert_eq!(
            ler_complementos(&[arquivo], 10),
            Ok(vec!["<html>SEGUNDA PÁGINA</html>".to_owned()])
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn recusa_reutilizar_o_mesmo_arquivo_como_se_fosse_pagina_distinta() {
        let arquivos = vec![PathBuf::from("segunda.html"), PathBuf::from("segunda.html")];
        assert_eq!(
            conferir_caminhos(&arquivos),
            Err(FalhaPaginaCapturada::CaminhosRepetidos)
        );
    }

    #[test]
    fn limite_de_199_complementos_e_aplicado_antes_de_ler_credenciais() {
        let arquivos = (0..200)
            .map(|n| PathBuf::from(format!("pagina-{n}.html")))
            .collect::<Vec<_>>();
        assert_eq!(
            conferir_caminhos(&arquivos),
            Err(FalhaPaginaCapturada::QuantidadeAcimaDoLimite)
        );
    }

    #[cfg(unix)]
    #[test]
    fn pagina_extra_nao_pode_ser_link_simbolico() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let dir = pasta();
        let original = dir.join("original.html");
        let atalho = dir.join("atalho.html");
        fs::write(&original, "<html>privado</html>").unwrap();
        fs::set_permissions(&original, fs::Permissions::from_mode(0o600)).unwrap();
        symlink(&original, &atalho).unwrap();
        assert_eq!(
            ler_complementos(&[atalho], 0),
            Err(FalhaPaginaCapturada::Arquivo(FalhaTransporte::ArquivoPrivadoLinkSimbolico))
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
