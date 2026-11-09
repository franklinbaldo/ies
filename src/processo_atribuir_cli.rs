//! Prévia Web de atribuição por login exato; escrita bloqueada até a
//! sequência POST único e releitura estar coberta por teste de transporte.
use crate::processo::NumeroProcesso;
use crate::processo_atribuir::{
    ErroProcessoAtribuir, EstadoDaAtribuicao, PedidoDaAtribuicao, ProcessoAtribuido,
    executar_previa,
};
use crate::processo_concluir_cli::{com_sessao, erro_de_uso, json_base_da_falha};
use crate::saida_erro;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_atribuir(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Atribuir {
                numero,
                usuario,
                remover,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    Some(match NumeroProcesso::novo(numero) {
        Ok(nup) => {
            if *confirmar {
                // Segurança: a escrita incompleta é recusada antes de abrir sessão ou rede.
                falhar(
                    cli.json,
                    ErroProcessoAtribuir::EscritaAindaNaoVerificada.subcodigo(),
                )
            } else {
                com_sessao(cli, |sessao, indice| {
                    let pedido = PedidoDaAtribuicao {
                        nup: &nup,
                        unidade: cli.unidade.as_deref(),
                        usuario: if *remover { None } else { usuario.as_deref() },
                        confirmar: false,
                    };
                    match executar_previa(sessao, &pedido, indice, &mut EnvioHttps) {
                        Ok(feito) => publicar_previa(cli.json, &feito),
                        Err(erro) => publicar_falha(cli.json, &erro),
                    }
                })
            }
        }
        Err(erro) => erro_de_uso(cli.json, erro.subcodigo()),
    })
}

/// A trava de escrita ainda é recusada sem sessão, preservando o contrato.
fn falhar(json_mode: bool, subcodigo: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "atribuicao_recusada",
                "subcodigo": subcodigo,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: atribuição recusada [{subcodigo}]");
    }
    ExitCode::from(1)
}

/// Toda recusa HTTP expõe status e mensagem saneada; a causa é preservada.
/// Fatos de diagnóstico da prévia são publicados sem extrapolar uma unidade
/// da árvore para outra: lista vazia = observada sem abertos; null = não lida.
pub(crate) fn json_de_falha(erro: &ErroProcessoAtribuir) -> serde_json::Value {
    let mut valor = json_base_da_falha(
        "atribuicao_recusada",
        erro.subcodigo(),
        erro.mensagem(),
        erro.status_http().cloned(),
    );
    if let Some(objeto) = valor.as_object_mut() {
        match erro {
            ErroProcessoAtribuir::SituacaoDaAtribuicaoNaoObservada {
                unidade,
                aberto_em,
                motivo,
                requisicoes,
            } => {
                objeto.insert("unidade".to_owned(), json!(unidade));
                objeto.insert("aberto_em".to_owned(), json!(aberto_em));
                objeto.insert("motivo".to_owned(), json!(motivo));
                objeto.insert("requisicoes".to_owned(), json!(requisicoes));
            }
            ErroProcessoAtribuir::EfeitoIncertoNaAtribuicao {
                requisicoes,
                atribuicao_relida,
                ..
            } => {
                objeto.insert("requisicoes".to_owned(), json!(requisicoes));
                // Estado observado após o POST, não evidência de autoria.
                objeto.insert("atribuicao_relida".to_owned(), json!(atribuicao_relida));
            }
            _ => {}
        }
    }
    valor
}

fn publicar_falha(json_mode: bool, erro: &ErroProcessoAtribuir) -> ExitCode {
    if json_mode {
        println!("{}", json_de_falha(erro));
    } else {
        eprintln!(
            "{}",
            saida_erro::linha_humana(erro.subcodigo(), erro.mensagem(), erro.status_http())
        );
    }
    ExitCode::from(erro.saida())
}

/// Traduz o estado único do domínio para os dois campos do contrato JSON legado.
pub(crate) fn json_da_previa(feito: &ProcessoAtribuido) -> serde_json::Value {
    let (confirmado, alterado) = match feito.estado {
        EstadoDaAtribuicao::Previa => (false, false),
        EstadoDaAtribuicao::AlteracaoConfirmada => (true, true),
    };
    json!({
        "nup": feito.nup,
        "id_procedimento": feito.id_procedimento,
        "unidade": feito.unidade,
        "usuario_atual": feito.usuario_atual,
        "usuario_pedido": feito.usuario_pedido,
        "remover": feito.usuario_pedido.is_none(),
        "valor_da_opcao": feito.valor_da_opcao,
        "confirmado": confirmado,
        "alterado": alterado,
        "origem": "web",
        "requisicoes": feito.requisicoes
    })
}

fn publicar_previa(json_mode: bool, feito: &ProcessoAtribuido) -> ExitCode {
    if json_mode {
        println!("{}", json_da_previa(feito));
    } else {
        println!("Prévia: processo {}, unidade {}.", feito.nup, feito.unidade);
        if feito.usuario_atual.is_empty() {
            println!("Atribuição atual: nenhuma.");
        } else {
            println!("Atribuição atual: {}.", feito.usuario_atual);
        }
        if let Some(login) = &feito.usuario_pedido {
            println!("Atribuir a {login}; opção verificada na tela.");
        } else {
            println!("Remover a atribuição observada.");
        }
        println!("Requisições: {}", feito.requisicoes);
    }
    ExitCode::SUCCESS
}
