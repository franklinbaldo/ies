//! Projeção e exportação privadas de um resumo da caixa de processos.
use crate::arquivo_caixa::{self, ErroArquivoCaixa};
use crate::resumo_processo::Ocorrencia;
use serde_json::{Value, json};
use std::path::Path;

pub(crate) fn modelo(numero: &str, unidade: Option<&str>, achados: &[Ocorrencia]) -> Value {
    json!({
        "numero": numero,
        "escopo": "resumo_da_caixa",
        "ocorrencias": achados,
        "unidade": unidade
    })
}

/// Shim utilizado nos testes locais; fluxo Web usa pasta preparada antes da rede.
#[allow(dead_code)]
pub(crate) fn gravar(destino: &Path, valor: &Value) -> Result<usize, ErroArquivoCaixa> {
    arquivo_caixa::gravar(destino, valor)
}

#[cfg(test)]
mod testes {
    use super::*;
    use clap::Parser;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn saida_exige_resumo_mas_pode_ser_usada_com_json() {
        assert!(
            crate::Cli::try_parse_from(["ies", "processo", "00007", "--saida", "r.json"]).is_err()
        );
        assert!(
            crate::Cli::try_parse_from([
                "ies", "processo", "00007", "--resumo", "--saida", "r.json", "--json"
            ])
            .is_ok()
        );
    }

    #[test]
    fn exporta_projecao_tipificada_sem_sobrescrever_arquivo() {
        let destino = std::env::temp_dir().join(format!(
            "ies-resumo-{}-{}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let valor = modelo("00007", Some("TESTE"), &[]);
        assert_eq!(valor["escopo"], "resumo_da_caixa");
        assert!(valor.get("versao_remota").is_none());
        assert!(gravar(&destino, &valor).is_ok());
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(&destino).unwrap()).unwrap(),
            valor
        );
        assert_eq!(
            gravar(&destino, &valor),
            Err(ErroArquivoCaixa::DestinoJaExiste)
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&destino).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        fs::remove_file(destino).unwrap();
    }
}
