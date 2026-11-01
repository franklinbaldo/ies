//! Contrato E2E: importar a exportação multipágina sem abrir conexão ao SEI.
#[cfg(unix)]
mod tests {
    use serde_json::{Value, json};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn arquivo_temporario() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "ies-caixa-multipagina-{}-{}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn executar(caminho: &std::path::Path) -> (i32, Value) {
        let resultado = Command::new(env!("CARGO_BIN_EXE_ies"))
            .args(["processo", "000123/2026", "--resumo", "--caixa-local"])
            .arg(caminho)
            .arg("--json")
            .env_remove("IES_CONFIG_FILE")
            .env_remove("IES_URL_CONTROLE_FILE")
            .env_remove("IES_COOKIE_FILE")
            .env_remove("IES_ORIGEM_FILE")
            .output()
            .unwrap();
        (
            resultado.status.code().unwrap(),
            serde_json::from_slice(&resultado.stdout).unwrap(),
        )
    }

    #[test]
    fn processo_de_exportacao_multipagina_e_encontrado_sem_rede() {
        let caminho = arquivo_temporario();
        let mut dados = json!({
            "origem": "web_com_paginas_privadas",
            "unidade": "PPI",
            "recebidos": {
                "total_declarado": 1,
                "processos": [
                    {"id": "0042", "numero": "000123/2026", "visualizado": false}
                ]
            },
            "gerados": {"total_declarado": 0, "processos": []}
        });
        fs::write(&caminho, dados.to_string()).unwrap();
        fs::set_permissions(&caminho, fs::Permissions::from_mode(0o600)).unwrap();
        let (codigo, resultado) = executar(&caminho);
        assert_eq!(codigo, 0);
        assert_eq!(resultado["numero"], "000123/2026");
        assert_eq!(resultado["fonte"], "arquivo_local");
        assert_eq!(resultado["autenticacao_remota"], false);
        assert_eq!(resultado["ocorrencias"][0]["processo"]["id"], "0042");

        dados["recebidos"]["total_declarado"] = json!(2);
        fs::write(&caminho, dados.to_string()).unwrap();
        let (codigo, erro) = executar(&caminho);
        assert_eq!(codigo, 1);
        assert_eq!(
            erro["subcodigo"],
            "total_recebidos_do_arquivo_nao_comprovado"
        );
        fs::remove_file(caminho).unwrap();
    }
}
