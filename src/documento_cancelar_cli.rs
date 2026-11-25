//! `ies documento cancelar DOC --motivo T [--confirmar]` (#273).
//! Números são validados antes da sessão; o motivo segue como texto opaco.
use crate::documento_cancelar::{DocumentoCancelado, PedidoDoCancelamento, cancelar_documento};
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain};

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
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDoCancelamento {
            numero,
            processo: processo.as_ref(),
            unidade: cli.unidade.as_deref(),
            motivo,
            confirmar: *confirmar,
        };
        let feito = cancelar_documento(sessao, &pedido, indice, &mut EnvioHttps)?;
        if cli.json {
            let mut valor = serde_json::to_value(resultado_cancelamento(&feito))?;
            crate::indice_local::anexar_avisos(&mut valor, indice);
            println!("{}", serde_json::to_string_pretty(&valor)?);
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
    falha_do_envio: Option<String>,
    id_documento: &'a str,
    motivo: &'a str,
    numero: &'a str,
    origem: &'static str,
    processo: ProcessoCanceladoPublico<'a>,
    reconciliado_por_leitura: bool,
    requisicoes: u8,
    unidade: &'a str,
}

fn resultado_cancelamento(feito: &DocumentoCancelado) -> ResultadoCancelamento<'_> {
    ResultadoCancelamento {
        alterado: feito.resultado.cancelado(),
        confirmado: feito.resultado.cancelado(),
        falha_do_envio: feito.resultado.falha_do_envio().map(|f| format!("{f:?}")),
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
        unidade: feito.unidade.as_str(),
    }
}

#[cfg(test)]
#[path = "documento_cancelar_cli/tests_saida.rs"]
mod tests_saida;
