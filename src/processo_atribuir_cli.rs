//! Prévia e confirmação Web de atribuição por login exato.
//! O envio usa um único POST e confere o efeito com releitura independente.
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
        Ok(nup) => com_sessao(cli, |sessao, indice| {
            let pedido = PedidoDaAtribuicao {
                nup: &nup,
                unidade: cli.unidade.as_deref(),
                usuario: if *remover { None } else { usuario.as_deref() },
                confirmar: *confirmar,
            };
            match executar_previa(sessao, &pedido, indice, &mut EnvioHttps) {
                Ok(feito) => publicar_resultado(cli.json, &feito),
                Err(erro) => publicar_falha(cli.json, &erro),
            }
        }),
        Err(erro) => erro_de_uso(cli.json, erro.subcodigo()),
    })
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

fn publicar_resultado(json_mode: bool, feito: &ProcessoAtribuido) -> ExitCode {
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
        let situacao = if feito.confirmado {
            "Atribuição confirmada"
        } else {
            "Prévia da atribuição"
        };
        println!(
            "{situacao}: processo {}, unidade {}.",
            feito.nup, feito.unidade
        );
        match (&feito.usuario_pedido, feito.confirmado) {
            (Some(login), true) => println!("Atribuído a {login}; efeito conferido por releitura."),
            (Some(login), false) => println!("Atribuir a {login}; opção verificada na tela."),
            (None, true) => println!("Atribuição removida; efeito conferido por releitura."),
            (None, false) => println!("Remover a atribuição observada."),
        }
        println!("Requisições: {}", feito.requisicoes);
    }
    ExitCode::SUCCESS
}
