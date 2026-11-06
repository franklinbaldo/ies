//! Prévia Web de atribuição por login exato; escrita bloqueada até a
//! sequência POST único e releitura estar coberta por teste de transporte.
use crate::processo::NumeroProcesso;
use crate::processo_atribuir::{
    ErroProcessoAtribuir, PedidoDaAtribuicao, ProcessoAtribuido, executar_previa,
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
                falhar(cli.json, "escrita_de_atribuicao_ainda_nao_verificada")
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
fn publicar_falha(json_mode: bool, erro: &ErroProcessoAtribuir) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json_base_da_falha(
                "atribuicao_recusada",
                erro.subcodigo(),
                erro.mensagem(),
                erro.status_http().cloned()
            )
        );
    } else {
        eprintln!(
            "{}",
            saida_erro::linha_humana(erro.subcodigo(), erro.mensagem(), erro.status_http())
        );
    }
    ExitCode::from(1)
}

fn publicar_previa(json_mode: bool, feito: &ProcessoAtribuido) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "nup": feito.nup,
                "id_procedimento": feito.id_procedimento,
                "unidade": feito.unidade,
                "usuario_atual": feito.usuario_atual,
                "usuario_pedido": feito.usuario_pedido,
                "remover": feito.usuario_pedido.is_none(),
                "valor_da_opcao": feito.valor_da_opcao,
                "confirmado": feito.confirmado,
                "alterado": feito.alterado,
                "origem": "web",
                "requisicoes": feito.requisicoes
            })
        );
    } else {
        println!("Prévia: processo {}, unidade {}.", feito.nup, feito.unidade);
        if let Some(login) = &feito.usuario_pedido {
            println!("Atribuir a {login}; opção verificada na tela.");
        } else {
            println!("Remover a atribuição observada.");
        }
        println!("Requisições: {}", feito.requisicoes);
    }
    ExitCode::SUCCESS
}
