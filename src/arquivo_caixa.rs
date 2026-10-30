//! Publicação local exclusiva de caixa validada; nunca grava uma leitura parcial.
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroArquivoCaixa {
    DiretorioPaiAusente,
    DestinoJaExistente,
    CaminhoDestinoSemNome,
    DiretorioPaiInacessivel,
    TemporarioNaoCriado,
    ConteudoNaoSerializado,
    FalhaAoGravarTemporario,
    FalhaAoSincronizarTemporario,
    FalhaAoPublicarDestino,
    AindaNaoImplementado,
}
impl ErroArquivoCaixa {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::DiretorioPaiAusente => "diretorio_da_caixa_ausente",
            Self::DestinoJaExistente => "arquivo_da_caixa_ja_existe",
            Self::CaminhoDestinoSemNome => "caminho_da_caixa_sem_nome",
            Self::DiretorioPaiInacessivel => "diretorio_da_caixa_inacessivel",
            Self::TemporarioNaoCriado => "temporario_da_caixa_nao_criado",
            Self::ConteudoNaoSerializado => "json_da_caixa_nao_serializado",
            Self::FalhaAoGravarTemporario => "temporario_da_caixa_nao_gravado",
            Self::FalhaAoSincronizarTemporario => "temporario_da_caixa_nao_sincronizado",
            Self::FalhaAoPublicarDestino => "arquivo_da_caixa_nao_publicado",
            Self::AindaNaoImplementado => "exportacao_da_caixa_nao_implementada",
        }
    }
}
pub fn validar_destino(_destino: &Path) -> Result<(), ErroArquivoCaixa> {
    Err(ErroArquivoCaixa::AindaNaoImplementado)
}
pub fn salvar_exclusivo(_destino: &Path, _valor: &Value) -> Result<(), ErroArquivoCaixa> {
    Err(ErroArquivoCaixa::AindaNaoImplementado)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn teste_pasta() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ies-exportacao-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn exportacao_cria_json_privado_e_nao_sobrescreve_destino() {
        let pasta = teste_pasta();
        let destino = pasta.join("processos.json");
        let valor = serde_json::json!({"recebidos":{"processos":[]},"gerados":{"processos":[]}});
        assert_eq!(validar_destino(&destino), Ok(()));
        assert_eq!(salvar_exclusivo(&destino, &valor), Ok(()));
        let lido: Value = serde_json::from_slice(&fs::read(&destino).unwrap()).unwrap();
        assert_eq!(lido, valor);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&destino).unwrap().permissions().mode() & 0o777, 0o600);
        }
        assert_eq!(validar_destino(&destino), Err(ErroArquivoCaixa::DestinoJaExistente));
        assert_eq!(salvar_exclusivo(&destino, &Value::Null), Err(ErroArquivoCaixa::DestinoJaExistente));
        assert_eq!(fs::read_to_string(&destino).unwrap().contains("recebidos"), true);
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn diretorio_ausente_falha_com_erro_especifico_sem_arquivo_parcial() {
        let pasta = teste_pasta();
        let destino = pasta.join("nao-existe").join("processos.json");
        assert_eq!(validar_destino(&destino), Err(ErroArquivoCaixa::DiretorioPaiAusente));
        assert_eq!(salvar_exclusivo(&destino, &serde_json::json!({})), Err(ErroArquivoCaixa::DiretorioPaiAusente));
        assert!(!destino.exists());
        fs::remove_dir_all(pasta).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn destino_symlink_ja_existente_nunca_e_substituido() {
        use std::os::unix::fs::symlink;
        let pasta = teste_pasta();
        let alvo = pasta.join("alvo.txt");
        let destino = pasta.join("processos.json");
        fs::write(&alvo, b"preservar").unwrap();
        symlink(&alvo, &destino).unwrap();
        assert_eq!(salvar_exclusivo(&destino, &Value::Null), Err(ErroArquivoCaixa::DestinoJaExistente));
        assert_eq!(fs::read(&alvo).unwrap(), b"preservar");
        fs::remove_dir_all(pasta).unwrap();
    }
}
