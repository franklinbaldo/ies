//! Borda CLI de `ies processo acompanhar NUP`: prévia sem POST e erros tipados.
use crate::processo::NumeroProcesso;
use crate::processo_acompanhar::{ErroPrevia, Pedido, Previa, previa};
use crate::processo_concluir_cli::{com_sessao, erro_de_uso, publicar_valor_da_falha};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_acompanhar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Acompanhar {
            numero, grupo, observacao, deixar, confirmar,
        }),
        ..
    }) = &cli.command else {
        return None;
    };
    Some(match NumeroProcesso::novo(numero) {
        Ok(nup) => {
            // Não abrir credenciais nem rede quando a escrita está bloqueada.
            if *confirmar {
                return Some(publicar_erro(cli.json, &ErroPrevia::ConfirmacaoNaoImplementada));
            }
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
        println!("{}", json!({
            "numero": plano.numero,
            "unidade": plano.unidade,
            "operacao": plano.operacao,
            "grupo": plano.grupo,
            "observacao": plano.observacao,
            "id_acompanhamento": plano.id_de_remocao,
            "ja_estava": plano.ja_estava,
            "confirmado": false,
            "alterado": false,
            "origem": "web",
            "requisicoes": plano.requisicoes,
        }));
    } else {
        println!("Prévia — processo {} (unidade {})", plano.numero, plano.unidade);
        if plano.ja_estava {
            println!("Nada a fazer: o acompanhamento solicitado já está no estado pedido.");
        } else if plano.operacao == "deixar" {
            println!("Retiraria o acompanhamento do grupo: {}", plano.grupo);
        } else {
            println!("Incluíria acompanhamento no grupo: {}", plano.grupo);
            println!("Observação: {}", plano.observacao);
        }
        println!("Nenhuma escrita executada. Requisições: {}", plano.requisicoes);
    }
    ExitCode::SUCCESS
}

fn publicar_erro(json_mode: bool, erro: &ErroPrevia) -> ExitCode {
    let valor = json!({
        "codigo": "operacao_recusada",
        "subcodigo": erro.subcodigo(),
        "erro": "Não foi possível conferir a prévia do Acompanhamento Especial; nenhuma escrita foi enviada.",
        "retentavel": false
    });
    publicar_valor_da_falha(json_mode, &valor, 1)
}
