//! CLI de acompanhamento: prévia e inclusão com POST único e releitura.
use crate::processo::NumeroProcesso;
use crate::processo_acompanhar::{ErroPrevia, Pedido, Previa, previa};
use crate::processo_concluir_cli::{com_sessao, erro_de_uso, publicar_valor_da_falha};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_acompanhar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Acompanhar {
                numero,
                grupo,
                observacao,
                deixar,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    Some(match NumeroProcesso::novo(numero) {
        Ok(nup) => {
            let pedido = Pedido {
                nup: &nup,
                unidade: cli.unidade.as_deref(),
                grupo: grupo.as_deref(),
                observacao: observacao.as_deref(),
                deixar: *deixar,
                confirmar: *confirmar,
            };
            com_sessao(cli, |sessao, indice| {
                if *deixar
                    && *confirmar
                    && let Err(causa) = crate::origem_de_treinamento::exigir(
                        crate::origem_de_treinamento::Escrita::ProcessoAcompanharDeixar,
                        sessao.origem,
                    )
                {
                    return crate::origem_de_treinamento::recusa(cli.json, causa);
                }
                match previa(sessao, &pedido, indice, &mut EnvioHttps) {
                    Ok(plano) => publicar_previa(cli.json, &plano),
                    Err(erro) => publicar_erro(cli.json, &erro),
                }
            })
        }
        Err(_) => erro_de_uso(cli.json),
    })
}

fn publicar_previa(json_mode: bool, plano: &Previa) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "numero": plano.numero,
                "unidade": plano.unidade,
                "operacao": plano.operacao,
                "grupo": plano.grupo,
                "observacao": plano.observacao,
                "id_acompanhamento": plano.id_de_remocao,
                "ja_estava": plano.estado.ja_estava(),
                "confirmado": plano.estado.confirmado(),
                "alterado": plano.estado.alterado(),
                "origem": "web",
                "requisicoes": plano.requisicoes,
            })
        );
    } else {
        println!(
            "{} — processo {} (unidade {})",
            if plano.estado.confirmado() {
                "Confirmação"
            } else {
                "Prévia"
            },
            plano.numero,
            plano.unidade
        );
        if plano.estado.ja_estava() {
            println!("Nada a fazer: o acompanhamento solicitado já está no estado pedido.");
        } else if plano.operacao == "deixar" {
            println!("Retiraria o acompanhamento do grupo: {}", plano.grupo);
        } else if plano.estado.alterado() {
            if plano.operacao == "deixar" {
                println!("Retirado acompanhamento do grupo: {}", plano.grupo);
            } else {
                println!("Incluído acompanhamento no grupo: {}", plano.grupo);
                println!("Observação: {}", plano.observacao);
            }
        } else {
            println!("Incluiria acompanhamento no grupo: {}", plano.grupo);
            println!("Observação: {}", plano.observacao);
        }
        if plano.estado.alterado() {
            if let Some(id) = &plano.id_de_remocao {
                println!("ID observado na releitura: {id}");
            }
            println!(
                "Acompanhamento {} e confirmado por leitura.",
                if plano.operacao == "deixar" {
                    "retirado"
                } else {
                    "incluído"
                }
            );
        } else {
            println!("Nenhuma escrita executada.");
        }
        println!("Requisições: {}", plano.requisicoes);
    }
    ExitCode::SUCCESS
}

fn publicar_erro(json_mode: bool, erro: &ErroPrevia) -> ExitCode {
    let incerto = matches!(erro, ErroPrevia::EfeitoIncerto);
    let valor = json!({
        "codigo": if incerto { "efeito_incerto" } else { "operacao_recusada" },
        "erro": if incerto {
            "A inclusão pode ter ocorrido, mas a releitura não comprovou o efeito. Não repita a escrita."
        } else {
            "Não foi possível conferir o Acompanhamento Especial; nenhuma escrita foi enviada."
        },
        "retentavel": false
    });
    publicar_valor_da_falha(json_mode, &valor, if incerto { 3 } else { 1 })
}
