//! Falha `fonte_invalida` na saída: JSON e linha humana. Uma recusa HTTP
//! acrescenta `status_http` e, se houver, `mensagem_do_servidor` saneada
//! (contracts/cli.md, "Status e mensagem do SEI nas recusas HTTP"). A
//! validação da Infra (HTTP 200) publica só a mensagem.
use crate::transporte::StatusHttpRecusado;
use serde_json::Value;
use std::process::ExitCode;

/// Projeção pública dos dados de uma falha, sem reconstruir sua identidade.
#[derive(serde::Serialize)]
pub(crate) struct FalhaPublica {
    #[serde(skip_serializing_if = "Option::is_none")]
    mensagem_do_servidor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_http: Option<u16>,
}

impl FalhaPublica {
    pub(crate) fn nova(http: Option<StatusHttpRecusado>) -> Self {
        let status_http = http
            .as_ref()
            .map(|h| h.status)
            .filter(|status| *status != 200);
        let mensagem_do_servidor = http.and_then(|h| h.mensagem_do_servidor);
        Self {
            mensagem_do_servidor,
            status_http,
        }
    }
}

/// Projeção pública da falha de leitura. A classificação permanece no
/// erro de domínio; esta estrutura existe apenas na fronteira de saída.
#[derive(serde::Serialize)]
struct FalhaFontePublica<'a> {
    codigo: &'static str,
    erro: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    mensagem_do_servidor: Option<&'a str>,
    retentavel: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_http: Option<u16>,
}

fn falha_fonte_publica<'a>(
    mensagem: &'a str,
    http: Option<&'a StatusHttpRecusado>,
) -> FalhaFontePublica<'a> {
    FalhaFontePublica {
        codigo: "fonte_invalida",
        erro: mensagem,
        mensagem_do_servidor: http.and_then(|h| h.mensagem_do_servidor.as_deref()),
        retentavel: false,
        status_http: http.map(|h| h.status).filter(|status| *status != 200),
    }
}

/// O objeto JSON da falha; os campos HTTP só existem numa recusa HTTP.
pub(crate) fn json_de_erro_fonte(mensagem: &str, http: Option<&StatusHttpRecusado>) -> Value {
    // Consumidores legados ainda requerem Value; o stdout não.
    serde_json::to_value(falha_fonte_publica(mensagem, http))
        .expect("projeção de falha composta apenas de valores JSON")
}

/// A linha de `stderr` sem `--json`.
pub(crate) fn linha_humana(mensagem: &str, http: Option<&StatusHttpRecusado>) -> String {
    let linha = format!("IES: {mensagem}");
    match http {
        None => linha,
        Some(StatusHttpRecusado {
            status: 200,
            mensagem_do_servidor: Some(texto),
        }) => format!("{linha} (SEI: {texto})"),
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
    mensagem: &str,
    http: Option<&StatusHttpRecusado>,
) -> ExitCode {
    if json_mode {
        if let Err(erro) =
            crate::saida::imprimir_json_compacto(&falha_fonte_publica(mensagem, http))
        {
            eprintln!("IES: falha ao emitir JSON: {erro}");
            return ExitCode::from(1);
        }
    } else {
        eprintln!("{}", linha_humana(mensagem, http));
    }
    ExitCode::from(1)
}

/// Atalho da CLI para falha sem resposta HTTP observada.
pub(crate) fn erro_fonte(json_mode: bool, mensagem: &str) -> ExitCode {
    erro_fonte_http(json_mode, mensagem, None)
}

#[cfg(test)]
#[path = "saida_erro/tests.rs"]
mod tests;
