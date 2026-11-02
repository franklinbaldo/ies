//! Falha `fonte_invalida` na saída: JSON e linha humana. Uma recusa HTTP
//! acrescenta `status_http` e, se houver, `mensagem_do_servidor` saneada
//! (contracts/cli.md, "Status e mensagem do SEI nas recusas HTTP").
use crate::transporte::StatusHttpRecusado;
use serde_json::{Value, json};
use std::process::ExitCode;

/// O objeto JSON da falha; os campos HTTP só existem numa recusa HTTP.
pub(crate) fn json_de_erro_fonte(
    subcodigo: &str,
    mensagem: &str,
    http: Option<&StatusHttpRecusado>,
) -> Value {
    let mut valor = json!({
        "codigo": "fonte_invalida",
        "subcodigo": subcodigo,
        "erro": mensagem,
        "retentavel": false
    });
    if let (Some(http), Some(objeto)) = (http, valor.as_object_mut()) {
        objeto.insert("status_http".to_owned(), json!(http.status));
        if let Some(texto) = &http.mensagem_do_servidor {
            objeto.insert("mensagem_do_servidor".to_owned(), json!(texto));
        }
    }
    valor
}

/// A linha de `stderr` sem `--json`.
pub(crate) fn linha_humana(
    subcodigo: &str,
    mensagem: &str,
    http: Option<&StatusHttpRecusado>,
) -> String {
    let linha = format!("IES: {mensagem} [{subcodigo}]");
    match http {
        None => linha,
        Some(StatusHttpRecusado {
            status,
            mensagem_do_servidor: None,
        }) => format!("{linha} (HTTP {status})"),
        Some(StatusHttpRecusado {
            status,
            mensagem_do_servidor: Some(texto),
        }) => format!("{linha} (HTTP {status}: {texto})"),
    }
}

/// Imprime a falha e devolve a saída 1.
pub(crate) fn erro_fonte_http(
    json_mode: bool,
    subcodigo: &str,
    mensagem: &str,
    http: Option<&StatusHttpRecusado>,
) -> ExitCode {
    if json_mode {
        println!("{}", json_de_erro_fonte(subcodigo, mensagem, http));
    } else {
        eprintln!("{}", linha_humana(subcodigo, mensagem, http));
    }
    ExitCode::from(1)
}

#[cfg(test)]
#[path = "saida_erro/tests.rs"]
mod tests;
