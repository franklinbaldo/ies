//! CLI do registro de andamento: valida entrada antes da sessão, nunca retry.
use crate::processo::NumeroProcesso;
use crate::processo_andamento_registrar::{self, Erro, texto_valido};
use crate::processo_concluir_cli::{com_sessao, json_base_da_falha, publicar_valor_da_falha};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_registrar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Andamento {
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
        Ok(n) => n,
        Err(e) => {
            return Some(erro_de_uso(
                cli.json,
                e.subcodigo(),
                "Número de processo inválido",
            ));
        }
    };
    if !texto_valido(texto) {
        return Some(erro_de_uso(
            cli.json,
            "texto_do_andamento_vazio_ou_maior_que_500_caracteres",
            "Texto do andamento deve conter entre 1 e 500 caracteres",
        ));
    }
    // Escrita disponível exclusivamente para ensaio local debug.
    // Release e qualquer outra origem recusam antes de carregar a sessão.
    if *confirmar && (!cfg!(debug_assertions) || !origem_local_sintetica()) {
        return Some(escrita_fechada(cli.json));
    }
    Some(com_sessao(cli, |sessao, indice| {
        if *confirmar && sessao.origem != "https://sei.test/sei" {
            return escrita_fechada(cli.json);
        }
        match processo_andamento_registrar::registrar(
            sessao,
            &numero,
            texto,
            *confirmar,
            cli.unidade.as_deref(),
            indice,
            &mut EnvioHttps,
        ) {
            Ok(r) => {
                if cli.json {
                    println!(
                        "{}",
                        json!({
                            "numero": r.numero,
                            "texto": r.texto,
                            "confirmado": r.confirmado,
                            "alterado": r.total_depois.is_some(),
                            "origem": "web",
                            "total_antes": r.total_antes,
                            "total_depois": r.total_depois,
                            "requisicoes": r.requisicoes,
                            "resposta_inesperada": r.resposta_inesperada,
                        })
                    );
                } else if r.confirmado {
                    println!(
                        "Andamento registrado e conferido no processo {}. Requisições: {}",
                        r.numero, r.requisicoes
                    );
                } else {
                    println!(
                        "Prévia: registrar no processo {} o andamento: {:?}. Nenhuma escrita. Requisições: {}. Para gravar, use --confirmar.",
                        r.numero, r.texto, r.requisicoes
                    );
                }
                ExitCode::SUCCESS
            }
            Err(Erro::Abertura(e)) => {
                saida_erro::erro_fonte_http(cli.json, e.subcodigo(), e.mensagem(), e.status_http())
            }
            Err(e) => publicar_valor_da_falha(
                cli.json,
                &json_base_da_falha(
                    if e.saida() == 3 {
                        "efeito_incerto"
                    } else {
                        "fonte_invalida"
                    },
                    e.subcodigo(),
                    e.mensagem(),
                    None,
                ),
                e.saida(),
            ),
        }
    }))
}

fn erro_de_uso(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida", "subcodigo": subcodigo,
                "erro": mensagem, "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]");
    }
    ExitCode::from(2)
}

/// O marcador de origem local não equivale a uma autorização de escrita
/// em qualquer SEI: a sessão é conferida novamente após ser carregada.
fn origem_local_sintetica() -> bool {
    std::env::var_os("IES_CONFIG_FILE").is_none()
        && std::env::var("IES_ORIGEM_FILE")
            .ok()
            .and_then(|caminho| std::fs::read_to_string(caminho).ok())
            .is_some_and(|origem| origem.trim() == "https://sei.test/sei")
}

fn escrita_fechada(json_mode: bool) -> ExitCode {
    let erro = Erro::EscritaAindaFechada;
    publicar_valor_da_falha(
        json_mode,
        &json_base_da_falha("fonte_invalida", erro.subcodigo(), erro.mensagem(), None),
        erro.saida(),
    )
}
