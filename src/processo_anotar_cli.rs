//! BDD-ANOT-01: o comando anotar apenas prevê a gravação, sem POST.
use crate::processo::NumeroProcesso;
use crate::processo_anotar::{self, Erro};
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_anotar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Anotar { numero, texto }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = match NumeroProcesso::novo(numero) {
        Ok(numero) => numero,
        Err(erro) => return Some(erro_de_uso(cli.json, erro.subcodigo())),
    };
    if texto.trim().is_empty() {
        if cli.json {
            println!(
                "{}",
                json!({
                    "codigo": "entrada_invalida",
                    "erro": "o texto da anotação não pode ser vazio",
                    "retentavel": false
                })
            );
        } else {
            eprintln!("IES: o texto da anotação não pode ser vazio");
        }
        return Some(ExitCode::from(2));
    }
    Some(com_sessao(cli, |sessao, indice| {
        match processo_anotar::prever(
            sessao,
            &nup,
            texto,
            cli.unidade.as_deref(),
            indice,
            &mut EnvioHttps,
        ) {
            Ok(preview) => {
                if cli.json {
                    println!(
                        "{}",
                        json!({
                            "nup": preview.nup,
                            "id_procedimento": preview.id_procedimento,
                            "unidade": preview.unidade,
                            "texto": preview.texto,
                            "anotacao_antes": preview.anotacao_antes,
                            "confirmado": false,
                            "alterado": false,
                            "origem": "web",
                            "requisicoes": preview.requisicoes,
                        })
                    );
                } else {
                    println!(
                        "Prévia: anotar no processo {} ({}): {:?}. Anotação atual: {:?}. Nenhuma escrita realizada. Requisições: {}.",
                        preview.nup,
                        preview.unidade,
                        preview.texto,
                        preview.anotacao_antes,
                        preview.requisicoes
                    );
                }
                ExitCode::SUCCESS
            }
            Err(Erro::Abertura(erro)) => saida_erro::erro_fonte_http(
                cli.json,
                erro.subcodigo(),
                erro.mensagem(),
                erro.status_http(),
            ),
            Err(erro) => {
                let subcodigo = erro.subcodigo();
                if cli.json {
                    println!(
                        "{}",
                        json!({
                            "codigo": "fonte_invalida",
                            "subcodigo": subcodigo,
                            "erro": "o SEI não publicou a anotação deste processo para esta unidade; nenhuma escrita foi realizada",
                            "retentavel": false
                        })
                    );
                } else {
                    eprintln!("IES: anotação não disponível para prévia [{subcodigo}]");
                }
                ExitCode::from(1)
            }
        }
    }))
}
