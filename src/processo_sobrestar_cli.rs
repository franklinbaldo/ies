//! Despacho da prévia de sobrestar/retomar; nenhuma credencial ou URL
//! assinada é publicada. A confirmação permanece fechada sem rede (#269).
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::com_sessao;
use crate::processo_sobrestar::{Acao, ErroSobrestamento, preparar};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_operar(cli: &Cli) -> Option<ExitCode> {
    match &cli.command {
        Domain::Processo(Processo {
            acao:
                Some(ProcessoAcao::Sobrestar {
                    numero,
                    motivo,
                    confirmar,
                }),
            ..
        }) => Some(operar(
            cli,
            numero,
            Some(motivo),
            *confirmar,
            Acao::Sobrestar,
        )),
        Domain::Processo(Processo {
            acao: Some(ProcessoAcao::Retomar { numero, confirmar }),
            ..
        }) => Some(operar(cli, numero, None, *confirmar, Acao::Retomar)),
        _ => None,
    }
}

fn erro_de_uso(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": subcodigo,
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]");
    }
    ExitCode::from(2)
}

fn operar(cli: &Cli, numero: &str, motivo: Option<&str>, confirmar: bool, acao: Acao) -> ExitCode {
    let nup = match NumeroProcesso::novo(numero) {
        Ok(nup) => nup,
        Err(erro) => return erro_de_uso(cli.json, erro.subcodigo(), "número de processo inválido"),
    };
    if matches!(acao, Acao::Sobrestar) && motivo.is_none_or(|t| t.trim().is_empty()) {
        return erro_de_uso(
            cli.json,
            "motivo_do_sobrestamento_vazio",
            "informe um motivo não vazio",
        );
    }
    // Nunca chamar a Web, nem mesmo ler a sessão, antes de existir
    // escrita verificada e teste de transporte para a operação.
    if confirmar {
        let e = ErroSobrestamento::ConfirmacaoAindaBloqueada;
        return saida_erro::erro_fonte_http(cli.json, e.subcodigo(), e.mensagem(), e.status_http());
    }
    com_sessao(cli, |sessao, indice| {
        match preparar(
            sessao,
            &nup,
            cli.unidade.as_deref(),
            acao,
            indice,
            &mut EnvioHttps,
        ) {
            Ok(previa) => {
                if cli.json {
                    println!(
                        "{}",
                        json!({
                            "nup": previa.nup,
                            "id_procedimento": previa.id_procedimento,
                            "unidade": previa.unidade,
                            "acao": previa.acao.como_texto(),
                            "motivo": motivo.map(str::trim),
                            "confirmado": false,
                            "alterado": false,
                            "origem": "web",
                            "requisicoes": previa.requisicoes
                        })
                    );
                } else {
                    println!(
                        "Prévia: {} o processo {} na unidade {}. Requisições: {}. Nenhuma escrita enviada.",
                        previa.acao.como_texto(),
                        previa.nup,
                        previa.unidade,
                        previa.requisicoes
                    );
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                saida_erro::erro_fonte_http(cli.json, e.subcodigo(), e.mensagem(), e.status_http())
            }
        }
    })
}
