//! `ies unidade colunas --definir todas|LISTA [--confirmar]`
//! (contracts/cli/unidade-definir-colunas.md): prévia sem escrita; com
//! `--confirmar`, um POST de salvar, conferido por leitura.
use crate::visao_do_controle::{ColunasDefinidas, ErroDefinirColunas, ErroVisaoDoControle};
use serde_json::{Value, json};
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

/// JSON do sucesso (prévia, inalterado ou salvo e conferido).
pub(crate) fn json_de_sucesso(feito: &ColunasDefinidas) -> Value {
    let _ = feito;
    todo!("RED: json de sucesso")
}

/// JSON da falha; `volta` é a falha da restauração da R, se houve.
pub(crate) fn json_de_falha(
    erro: &ErroDefinirColunas,
    volta: Option<&ErroVisaoDoControle>,
) -> Value {
    let _ = (erro, volta);
    todo!("RED: json de falha")
}

#[cfg(test)]
#[path = "unidade_colunas_definir_cli/tests.rs"]
mod tests;
