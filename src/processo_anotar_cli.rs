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
    Some(com_sessao(
        cli,
        |sessao, indice| match processo_anotar::prever(
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
                use crate::erro_publico::Causa;
                let causa = crate::erro_publico::caminho_de(&erro);
                let mensagem = "o SEI não publicou a anotação deste processo para esta unidade; nenhuma escrita foi realizada";
                if cli.json {
                    let mut saida = json!({
                        "codigo": "fonte_invalida",
                        "causa": causa,
                        "erro": mensagem,
                        "retentavel": false
                    });
                    if let Some(http) = erro.status_http() {
                        if http.status != 200 {
                            saida["status_http"] = json!(http.status);
                        }
                        if let Some(mensagem_do_servidor) = &http.mensagem_do_servidor {
                            saida["mensagem_do_servidor"] = json!(mensagem_do_servidor);
                        }
                    }
                    println!("{saida}");
                } else {
                    eprintln!("IES: {mensagem} [{}]", causa.join("/"));
                }
                ExitCode::from(1)
            }
        },
    ))
}
