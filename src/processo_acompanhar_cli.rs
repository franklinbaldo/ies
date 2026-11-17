//! CLI de acompanhamento: prévia e inclusão com POST único e releitura.
use crate::indice_local::{self, IndiceLocal};
use crate::processo_acompanhar::{Pedido, Previa, previa};
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;

pub(crate) fn tentar_acompanhar(cli: &Cli) -> Option<anyhow::Result<()>> {
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
    let nup = numero.clone();
    Some({
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
                return Err(causa.into());
            }
            let plano = previa(sessao, &pedido, indice, &mut EnvioHttps)?;
            publicar_previa(cli.json, &plano, indice);
            Ok(())
        })
    })
}

fn publicar_previa(json_mode: bool, plano: &Previa, indice: &IndiceLocal) {
    if json_mode {
        let mut valor = json!({
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
        });
        indice_local::anexar_avisos(&mut valor, indice);
        println!("{valor}");
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
        } else if plano.estado.alterado() {
            if plano.operacao == "deixar" {
                println!("Retirado acompanhamento do grupo: {}", plano.grupo);
            } else {
                println!("Incluído acompanhamento no grupo: {}", plano.grupo);
                println!("Observação: {}", plano.observacao);
            }
        } else if plano.operacao == "deixar" {
            println!("Retiraria o acompanhamento do grupo: {}", plano.grupo);
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
}
