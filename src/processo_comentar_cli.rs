//! Comentários no processo: prévia real e confirmação ainda fechada.
//! A prévia não deriva nem envia POST; origem de produção nunca é escrita.
use crate::processo::NumeroProcesso;
use crate::processo_comentar_web;
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_comentar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Comentar {
                numero,
                texto,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let numero = match NumeroProcesso::novo(numero) {
        Ok(numero) => numero,
        Err(e) => return Some(erro_de_uso(cli.json, e.subcodigo())),
    };
    if texto.trim().is_empty() || texto.contains('\0') {
        return Some(recusa(cli.json, "Informe o texto do comentário."));
    }
    if *confirmar {
        return Some(recusa(
            cli.json,
            "A confirmação de comentário ainda não foi comprovada. Nenhuma requisição foi feita.",
        ));
    }
    Some(com_sessao(
        cli,
        |sessao, indice| match processo_comentar_web::prever(
            sessao,
            &numero,
            texto.trim(),
            cli.unidade.as_deref(),
            indice,
            &mut EnvioHttps,
        ) {
            Ok(preview) => {
                if cli.json {
                    println!(
                        "{}",
                        json!({
                            "nup": preview.numero,
                            "id_procedimento": preview.id_procedimento,
                            "texto": preview.texto,
                            "confirmado": false,
                            "alterado": false,
                            "origem": "web",
                            "requisicoes": preview.requisicoes,
                        })
                    );
                } else {
                    println!(
                        "Prévia: comentar no processo {}: {:?}. Nenhuma escrita. Requisições: {}.",
                        preview.numero, preview.texto, preview.requisicoes
                    );
                }
                ExitCode::SUCCESS
            }
            Err(_erro) => recusa(
                cli.json,
                "O formulário de comentário não foi comprovado; nenhuma escrita realizada.",
            ),
        },
    ))
}

fn recusa(json_mode: bool, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "operacao_recusada",
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(1)
}
