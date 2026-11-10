//! `ies documento cancelar DOC --motivo T [--confirmar]` (#273).
//! Números são validados antes da sessão; o motivo segue como texto opaco.
use crate::documento_cancelar::{
    DocumentoCancelado, ErroCancelar, FalhaDoCancelamento, PedidoDoCancelamento, cancelar_documento,
};
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{com_sessao, json_base_da_falha};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain};
use serde_json::{Value, json};
use std::process::ExitCode;

/// Só responde quando o comando é documento cancelar.
pub(crate) fn tentar_cancelar(cli: &Cli) -> Option<ExitCode> {
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
            return Some(uso_invalido(cli.json, "número de processo inválido"));
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
        match cancelar_documento(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(feito) => {
                if cli.json {
                    if let Err(erro) = crate::saida::imprimir_json(&resultado_cancelamento(&feito))
                    {
                        eprintln!("IES: falha ao emitir JSON: {erro}");
                        return std::process::ExitCode::from(1);
                    }
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
                ExitCode::SUCCESS
            }
            Err(erro) => {
                if cli.json {
                    println!("{}", json_da_falha(&erro));
                } else {
                    for linha in linhas_da_falha(&erro) {
                        eprintln!("{linha}");
                    }
                }
                ExitCode::from(1)
            }
        }
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

fn json_da_falha_do_envio(f: &FalhaDoCancelamento) -> Value {
    let mut valor = json!({});
    if let Some(http) = f.status_http()
        && let Some(obj) = valor.as_object_mut()
    {
        if http.status != 200 {
            obj.insert("status_http".to_owned(), json!(http.status));
        }
        if let Some(m) = http.mensagem_do_servidor {
            obj.insert("mensagem_do_servidor".to_owned(), json!(m));
        }
    }
    valor
}

fn json_da_falha(e: &ErroCancelar) -> Value {
    let mut valor = json_base_da_falha(e.codigo(), e.mensagem(), e.status_http());
    if let Some(obj) = valor.as_object_mut()
        && let ErroCancelar::CancelamentoNaoAplicado {
            falha_do_envio: Some(f),
        }
        | ErroCancelar::NaoConfirmado {
            falha_do_envio: Some(f),
            ..
        } = e
    {
        obj.insert("falha_do_envio".to_owned(), json_da_falha_do_envio(f));
    }
    valor
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

// Referência de contrato somente nos testes; produção emite Serialize diretamente.
#[cfg(test)]
fn json_do_sucesso(feito: &DocumentoCancelado) -> Value {
    let mut valor = json!({
        "numero": feito.numero, "id_documento":feito.id_documento,
        "processo":{"nup":feito.processo.nup,"id":feito.processo.id},
        "unidade":feito.unidade,
        "motivo":feito.motivo,
        "origem":"web", "confirmado":feito.resultado.cancelado(),"alterado":feito.resultado.cancelado(),
        "requisicoes":feito.requisicoes,
        "reconciliado_por_leitura":feito.resultado.cancelado()
    });
    if let (Some(f), Some(obj)) = (feito.resultado.falha_do_envio(), valor.as_object_mut()) {
        obj.insert("falha_do_envio".to_owned(), json_da_falha_do_envio(f));
    }
    valor
}

/// Mensagens humanas do cancelamento derivadas da falha tipada, sem
/// construir JSON ou reclassificar sua causa.
fn linhas_da_falha(erro: &ErroCancelar) -> Vec<String> {
    let mut linhas = vec![format!("IES: {}", erro.mensagem())];
    let falha_do_envio = match erro {
        ErroCancelar::CancelamentoNaoAplicado { falha_do_envio }
        | ErroCancelar::NaoConfirmado { falha_do_envio, .. } => falha_do_envio.as_ref(),
        _ => None,
    };
    if let Some(falha) = falha_do_envio {
        linhas.push(format!("IES: resposta da escrita: {falha:?}"));
    }
    linhas
}

#[cfg(test)]
#[path = "documento_cancelar_cli/tests_saida.rs"]
mod tests_saida;
