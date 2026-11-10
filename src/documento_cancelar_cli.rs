//! `ies documento cancelar DOC --motivo T [--confirmar]` (#273).
//! Números são validados antes da sessão; o motivo segue como texto opaco.
use crate::documento_cancelar::{DocumentoCancelado, PedidoDoCancelamento, cancelar_documento};
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain};
use serde_json::json;
use std::process::ExitCode;

/// Só responde quando o comando é documento cancelar.
pub(crate) fn tentar_cancelar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Documento(Documento {
        acao:
            Some(DocumentoAcao::Cancelar {
                numero,
                motivo,
                confirmar,
            }),
        processo,
        ..
    }) = &cli.command
    else {
        return None;
    };
    let numero = numero.clone();
    let processo = match processo.as_deref().map(NumeroProcesso::novo) {
        None => None,
        Some(Ok(n)) => Some(n),
        Some(Err(_)) => {
            return Some(crate::legado(uso_invalido(
                cli.json,
                "número de processo inválido",
            )));
        }
    };
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDoCancelamento {
            numero: &numero,
            processo: processo.as_ref(),
            unidade: cli.unidade.as_deref(),
            motivo,
            confirmar: *confirmar,
        };
        let feito = cancelar_documento(sessao, &pedido, indice, &mut EnvioHttps)?;
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&resultado_cancelamento(&feito))?
            );
        } else if feito.resultado.cancelado() {
            println!(
                "Documento {} cancelado e conferido pela árvore. Requisições: {}.",
                feito.numero, feito.requisicoes
            );
        } else {
            println!(
                "Prévia: cancelar documento {} no processo {}. Motivo: {}. Não foi enviado POST. Repita com --confirmar apenas em treinamento.",
                feito.numero, feito.processo.nup, feito.motivo
            );
        }
        Ok(())
    }))
}

fn uso_invalido(json_mode: bool, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({"codigo":"entrada_invalida","erro":mensagem,"retentavel":false})
        );
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(2)
}

/// Exposição pública sem os demais campos internos de `ProcessoNaArvore`.
#[derive(serde::Serialize)]
struct ProcessoCanceladoPublico<'a> {
    id: &'a str,
    nup: &'a str,
}

#[derive(serde::Serialize)]
struct ResultadoCancelamento<'a> {
    alterado: bool,
    confirmado: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    falha_do_envio: Option<crate::saida_erro::FalhaPublica>,
    id_documento: &'a str,
    motivo: &'a str,
    numero: &'a str,
    origem: &'static str,
    processo: ProcessoCanceladoPublico<'a>,
    reconciliado_por_leitura: bool,
    requisicoes: u8,
    unidade: Option<&'a str>,
}

fn resultado_cancelamento(feito: &DocumentoCancelado) -> ResultadoCancelamento<'_> {
    ResultadoCancelamento {
        alterado: feito.resultado.cancelado(),
        confirmado: feito.resultado.cancelado(),
        falha_do_envio: feito
            .resultado
            .falha_do_envio()
            .map(|f| crate::saida_erro::FalhaPublica::nova(f.status_http())),
        id_documento: &feito.id_documento,
        motivo: &feito.motivo,
        numero: &feito.numero,
        origem: "web",
        processo: ProcessoCanceladoPublico {
            id: &feito.processo.id,
            nup: &feito.processo.nup,
        },
        reconciliado_por_leitura: feito.resultado.cancelado(),
        requisicoes: feito.requisicoes,
        unidade: feito.unidade.as_deref(),
    }
}

#[cfg(test)]
#[path = "documento_cancelar_cli/tests_saida.rs"]
mod tests_saida;
