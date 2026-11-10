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
    subcodigo: &'static str,
}

impl FalhaPublica {
    pub(crate) fn nova(subcodigo: &'static str, http: Option<StatusHttpRecusado>) -> Self {
        let status_http = http
            .as_ref()
            .map(|h| h.status)
            .filter(|status| *status != 200);
        let mensagem_do_servidor = http.and_then(|h| h.mensagem_do_servidor);
        Self {
            mensagem_do_servidor,
            status_http,
            subcodigo,
        }
    }
}

/// Projeção pública da falha de leitura. A classificação permanece no
/// erro de domínio; esta estrutura existe apenas na fronteira de saída.
#[derive(serde::Serialize)]
struct FalhaFontePublica<'a, S: serde::Serialize + ?Sized> {
    codigo: &'static str,
    erro: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    mensagem_do_servidor: Option<&'a str>,
    retentavel: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_http: Option<u16>,
    subcodigo: &'a S,
}

fn falha_fonte_publica<'a, S: serde::Serialize + ?Sized>(
    subcodigo: &'a S,
    mensagem: &'a str,
    http: Option<&'a StatusHttpRecusado>,
) -> FalhaFontePublica<'a, S> {
    FalhaFontePublica {
        codigo: "fonte_invalida",
        erro: mensagem,
        mensagem_do_servidor: http.and_then(|h| h.mensagem_do_servidor.as_deref()),
        retentavel: false,
        status_http: http.map(|h| h.status).filter(|status| *status != 200),
        subcodigo,
    }
}

/// O objeto JSON da falha; os campos HTTP só existem numa recusa HTTP.
pub(crate) fn json_de_erro_fonte(
    subcodigo: &str,
    mensagem: &str,
    http: Option<&StatusHttpRecusado>,
) -> Value {
    // Consumidores legados ainda requerem Value; o stdout não.
    serde_json::to_value(falha_fonte_publica(subcodigo, mensagem, http))
        .expect("projeção de falha composta apenas de valores JSON")
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
    subcodigo: &str,
    mensagem: &str,
    http: Option<&StatusHttpRecusado>,
) -> ExitCode {
    if json_mode {
        if let Err(erro) =
            crate::saida::imprimir_json_compacto(&falha_fonte_publica(subcodigo, mensagem, http))
        {
            eprintln!("IES: falha ao emitir JSON: {erro}");
            return ExitCode::from(1);
        }
    } else {
        eprintln!("{}", linha_humana(subcodigo, mensagem, http));
    }
    ExitCode::from(1)
}

/// Saída tipada: o campo JSON recebe a variante Serde, sem string
/// manual na camada de domínio. Só a mensagem humana usa sua projeção
/// textual, já na fronteira de apresentação.
pub(crate) fn erro_fonte_tipado<S: serde::Serialize>(
    json_mode: bool,
    causa: &S,
    mensagem: &str,
) -> ExitCode {
    if json_mode {
        if let Err(erro) =
            crate::saida::imprimir_json_compacto(&falha_fonte_publica(causa, mensagem, None))
        {
            eprintln!("IES: falha ao emitir JSON: {erro}");
            return ExitCode::from(1);
        }
    } else {
        let representacao = serde_json::to_value(causa).expect("causa serializável");
        let identificador = representacao
            .as_str()
            .expect("a causa da origem é uma variante unitária");
        eprintln!("{}", linha_humana(identificador, mensagem, None));
    }
    ExitCode::from(1)
}

/// Atalho da CLI para falha sem resposta HTTP observada.
pub(crate) fn erro_fonte(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    erro_fonte_http(json_mode, subcodigo, mensagem, None)
}

#[cfg(test)]
#[path = "saida_erro/tests.rs"]
mod tests;
