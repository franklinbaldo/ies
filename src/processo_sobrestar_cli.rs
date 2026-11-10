//! Despacho da prévia e confirmação de sobrestar/retomar (#269).
//! Nenhuma credencial nem URL assinada é publicada; efeito incerto sai 3.
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::com_sessao;
use crate::processo_sobrestar::{Acao, ErroSobrestamento, PedidoDaMudanca, executar};
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

fn erro_de_uso(json_mode: bool, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(2)
}

fn operar(
    cli: &Cli,
    nup: &NumeroProcesso,
    motivo: Option<&str>,
    confirmar: bool,
    acao: Acao,
) -> ExitCode {
    if matches!(acao, Acao::Sobrestar) && motivo.is_none_or(|t| t.trim().is_empty()) {
        return erro_de_uso(cli.json, "informe um motivo não vazio");
    }
    com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDaMudanca {
            nup,
            unidade: cli.unidade.as_deref(),
            acao,
            motivo,
            confirmar,
        };
        match executar(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(previa) => {
                if cli.json {
                    println!(
                        "{}",
                        json!({
                            "nup": previa.nup,
                            "id_procedimento": previa.id_procedimento,
                            "unidade": previa.unidade,
                            "acao": previa.acao.como_texto(),
                            "motivo": motivo,
                            "confirmado": previa.estado.aplicada(),
                            "alterado": previa.estado.aplicada(),
                            "reconciliado_por_leitura": previa.estado.aplicada(),
                            "origem": "web",
                            "requisicoes": previa.requisicoes
                        })
                    );
                } else if previa.estado.aplicada() {
                    println!(
                        "Processo {}: {} aplicado e conferido por leitura. Requisições: {}.",
                        previa.nup,
                        previa.acao.como_texto(),
                        previa.requisicoes
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
                if matches!(e, ErroSobrestamento::NaoConfirmado) {
                    if cli.json {
                        println!(
                            "{}",
                            json!({
                                "codigo": "nao_confirmado",
                                "erro": e.mensagem(),
                                "retentavel": false,
                                "origem": "web"
                            })
                        );
                    } else {
                        eprintln!("IES: {} ({})", e.mensagem(), e.mensagem());
                    }
                    ExitCode::from(1)
                } else {
                    saida_erro::erro_fonte_http(cli.json, e.mensagem(), e.status_http())
                }
            }
        }
    })
}
