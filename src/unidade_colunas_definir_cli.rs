//! `ies unidade colunas --definir todas|LISTA [--confirmar]`
//! (contracts/cli/unidade-definir-colunas.md): prévia sem escrita; com
//! `--confirmar`, um POST de salvar, conferido por leitura.
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn definir_colunas(
    unidade: Option<&str>,
    json_mode: bool,
    definir: &str,
    confirmar: bool,
) -> ExitCode {
    let _ = (unidade, definir, confirmar);
    if json_mode {
        println!(
            "{}",
            json!({"codigo": "nao_implementado", "erro": "--definir ainda sem implementação", "retentavel": false})
        );
    } else {
        eprintln!("IES: --definir ainda sem implementação");
    }
    ExitCode::from(1)
}
