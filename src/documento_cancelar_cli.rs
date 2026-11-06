//! `ies documento cancelar DOC --motivo T [--confirmar]` (#273).
//! Motivo e números são validados antes de abrir arquivo privado ou rede.
use crate::documento::NumeroDocumento;
use crate::documento_cancelar::{
    DocumentoCancelado, ErroCancelar, FalhaDoCancelamento, PedidoDoCancelamento,
    cancelar_documento, conferir_motivo,
};
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{com_sessao, json_base_da_falha, publicar_valor_da_falha};
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
    let numero = match NumeroDocumento::novo(numero) {
        Ok(n) => n,
        Err(e) => {
            return Some(uso_invalido(
                cli.json,
                e.subcodigo(),
                "número de documento inválido",
            ));
        }
    };
    let processo = match processo.as_deref().map(NumeroProcesso::novo) {
        None => None,
        Some(Ok(n)) => Some(n),
        Some(Err(e)) => {
            return Some(uso_invalido(
                cli.json,
                e.subcodigo(),
                "número de processo inválido",
            ));
        }
    };
    let motivo = match conferir_motivo(motivo) {
        Ok(m) => m,
        Err(sub) => return Some(uso_invalido(cli.json, sub, "motivo inválido")),
    };
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDoCancelamento {
            numero: &numero,
            processo: processo.as_ref(),
            unidade: cli.unidade.as_deref(),
            motivo: &motivo,
            confirmar: *confirmar,
        };
        match cancelar_documento(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(feito) => {
                if cli.json {
                    if let Err(erro) = crate::saida::imprimir_json(&json_do_sucesso(&feito)) {
                        eprintln!("IES: falha ao emitir JSON: {erro}");
                        return std::process::ExitCode::from(1);
                    };
                } else if feito.alterado {
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
            Err(e) => publicar_valor_da_falha(cli.json, &json_da_falha(&e), e.saida()),
        }
    }))
}

fn uso_invalido(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({"codigo":"entrada_invalida","subcodigo":subcodigo,"erro":mensagem,"retentavel":false})
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]");
    }
    ExitCode::from(2)
}

fn json_da_falha_do_envio(f: &FalhaDoCancelamento) -> Value {
    let mut valor = json!({"subcodigo":f.subcodigo()});
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
    let mut valor = json_base_da_falha(e.codigo(), e.subcodigo(), e.mensagem(), e.status_http());
    if let Some(obj) = valor.as_object_mut() {
        match e {
            ErroCancelar::CancelamentoNaoAplicado {
                falha_do_envio: Some(f),
            } => {
                obj.insert("falha_do_envio".to_owned(), json_da_falha_do_envio(f));
            }
            ErroCancelar::EfeitoIncerto {
                falha_do_envio,
                verificacao,
            } => {
                obj.insert("verificacao".to_owned(), json!(verificacao.motivo()));
                if let Some(f) = falha_do_envio {
                    obj.insert("falha_do_envio".to_owned(), json_da_falha_do_envio(f));
                }
            }
            _ => {}
        }
    }
    valor
}

fn json_do_sucesso(feito: &DocumentoCancelado) -> Value {
    let mut valor = json!({
        "numero": feito.numero, "id_documento":feito.id_documento,
        "processo":{"nup":feito.processo.nup,"id":feito.processo.id},
        "unidade":feito.unidade,
        "motivo":feito.motivo,
        "origem":"web", "confirmado":feito.confirmado,"alterado":feito.alterado,
        "requisicoes":feito.requisicoes,
        "reconciliado_por_leitura":feito.reconciliado_por.is_some()
    });
    if let (Some(f), Some(obj)) = (&feito.reconciliado_por, valor.as_object_mut()) {
        obj.insert("falha_do_envio".to_owned(), json_da_falha_do_envio(f));
    }
    valor
}
