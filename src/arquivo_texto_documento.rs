//! Persistência privada de texto de documento já verificado offline.
//! Nunca sobrescreve destino existente, não publica conteúdo em diagnóstico.
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroArquivoTexto {
    NomeAusente,
    DiretorioAusente,
    DestinoJaExiste,
    EscritaFalhou,
    ImplementacaoPendente,
}
impl ErroArquivoTexto {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::NomeAusente => "destino_texto_documento_sem_nome",
            Self::DiretorioAusente => "diretorio_texto_documento_ausente",
            Self::DestinoJaExiste => "arquivo_texto_documento_ja_existe",
            Self::EscritaFalhou => "escrita_texto_documento_falhou",
            Self::ImplementacaoPendente => "exportacao_texto_documento_nao_implementada",
        }
    }
}
pub fn salvar(destino: &Path, texto: &str) -> Result<usize, ErroArquivoTexto> {
    let _ = (destino, texto);
    Err(ErroArquivoTexto::ImplementacaoPendente)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn pasta() -> std::path::PathBuf {
        let pasta = std::env::temp_dir().join(format!(
            "ies-texto-documento-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&pasta).unwrap();
        pasta
    }

    #[test]
    fn salva_texto_exato_em_arquivo_privado_novo_sem_sobrescrever() {
        let pasta = pasta();
        let destino = pasta.join("0007.txt");
        assert_eq!(salvar(&destino, "Decisão\nAssinada"), Ok("Decisão\nAssinada".len()));
        assert_eq!(fs::read_to_string(&destino).unwrap(), "Decisão\nAssinada");
        assert_eq!(salvar(&destino, "OUTRO"), Err(ErroArquivoTexto::DestinoJaExiste));
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&destino).unwrap().permissions().mode() & 0o777, 0o600);
        }
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn ausencia_de_diretorio_tem_erro_tipado_e_nao_cria_destino() {
        let pasta = pasta();
        let destino = pasta.join("inexistente/0007.txt");
        assert_eq!(salvar(&destino, "conteúdo"), Err(ErroArquivoTexto::DiretorioAusente));
        assert!(!destino.exists());
        fs::remove_dir_all(pasta).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn destino_simbolico_nao_pode_alterar_arquivo_existente() {
        use std::os::unix::fs::symlink;
        let pasta = pasta();
        let original = pasta.join("original.txt");
        fs::write(&original, "NAO SUBSTITUIR").unwrap();
        let destino = pasta.join("texto.txt");
        symlink(&original, &destino).unwrap();
        assert_eq!(salvar(&destino, "INTRUSO"), Err(ErroArquivoTexto::DestinoJaExiste));
        assert_eq!(fs::read_to_string(&original).unwrap(), "NAO SUBSTITUIR");
        fs::remove_dir_all(pasta).unwrap();
    }
}
