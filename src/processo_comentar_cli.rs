//! Comentários no processo: prévia e confirmação condicionada à origem.
//! Um POST e releitura causal, sem retry ou escrita em produção.
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
    Some(com_sessao(cli, |sessao, indice| {
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoComentar,
                sessao.origem,
            )
        {
            return crate::origem_de_treinamento::recusa(cli.json, causa);
        }
        match processo_comentar_web::prever(
            sessao,
            &numero,
            texto,
            *confirmar,
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
                            "confirmado": preview.confirmado,
                            "alterado": preview.confirmado,
                            "origem": "web",
                            "requisicoes": preview.requisicoes,
                        })
                    );
                } else if preview.confirmado {
                    println!(
                        "Comentário confirmado por releitura no processo {}. Requisições: {}.",
                        preview.numero, preview.requisicoes
                    );
                } else {
                    println!(
                        "Prévia: comentar no processo {}: {:?}. Nenhuma escrita. Requisições: {}.",
                        preview.numero, preview.texto, preview.requisicoes
                    );
                }
                ExitCode::SUCCESS
            }
            Err(processo_comentar_web::Erro::EfeitoIncerto) => efeito_incerto(cli.json),
            Err(causa) => recusa_previa(cli.json, &causa),
        }
    }))
}

fn efeito_incerto(json_mode: bool) -> ExitCode {
    let mensagem = "O POST pode ter sido aplicado, mas o comentário não foi confirmado pela releitura; consulte ies processo comentarios antes de tentar novamente. Não repita --confirmar.";
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "efeito_incerto",
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(3)
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

fn recusa_previa(json_mode: bool, causa: &processo_comentar_web::Erro) -> ExitCode {
    const MENSAGEM: &str =
        "O formulário de comentário não foi comprovado; nenhuma escrita realizada.";
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "operacao_recusada",
                "causa": causa,
                "erro": MENSAGEM,
                "retentavel": false,
            })
        );
    } else {
        eprintln!("IES: {MENSAGEM}");
    }
    ExitCode::from(1)
}
