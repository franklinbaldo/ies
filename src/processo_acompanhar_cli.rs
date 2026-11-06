//! CLI de acompanhamento: prévia, inclusão e retirada com POST único e releitura.
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
                match previa(sessao, &pedido, indice, &mut EnvioHttps) {
                    Ok(plano) => publicar_previa(cli.json, &plano),
                    Err(erro) => publicar_erro(cli.json, &erro),
                }
            })
        }
        Err(erro) => erro_de_uso(cli.json, erro.subcodigo()),
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
                "ja_estava": plano.ja_estava,
                "confirmado": plano.confirmado,
                "alterado": plano.alterado,
                "origem": "web",
                "requisicoes": plano.requisicoes,
            })
        );
    } else {
        println!(
            "{} — processo {} (unidade {})",
            if plano.confirmado {
                "Confirmação"
            } else {
                "Prévia"
            },
            plano.numero,
            plano.unidade
        );
        if plano.ja_estava {
            println!("Nada a fazer: o acompanhamento solicitado já está no estado pedido.");
        } else if plano.operacao == "deixar" && plano.alterado {
            println!("Retirado acompanhamento do grupo: {}", plano.grupo);
        } else if plano.operacao == "deixar" {
            println!("Retiraria o acompanhamento do grupo: {}", plano.grupo);
        } else if plano.alterado {
            println!("Incluído acompanhamento no grupo: {}", plano.grupo);
            println!("Observação: {}", plano.observacao);
        } else {
            println!("Incluiria acompanhamento no grupo: {}", plano.grupo);
            println!("Observação: {}", plano.observacao);
        }
        if plano.alterado {
            if let Some(id) = &plano.id_de_remocao {
                if plano.operacao == "deixar" {
                    println!("ID retirado, ausente na releitura: {id}");
                } else {
                    println!("ID observado na releitura: {id}");
                }
            }
            if plano.operacao == "deixar" {
                println!("Acompanhamento retirado e confirmado por leitura.");
            } else {
                println!("Acompanhamento incluído e confirmado por leitura.");
            }
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
        "subcodigo": erro.subcodigo(),
        "erro": if incerto {
            "A alteração pode ter ocorrido, mas a releitura não comprovou o efeito. Não repita a escrita."
        } else {
            "Não foi possível conferir o Acompanhamento Especial; nenhuma escrita foi enviada."
        },
        "retentavel": false
    });
    publicar_valor_da_falha(json_mode, &valor, if incerto { 3 } else { 1 })
}
