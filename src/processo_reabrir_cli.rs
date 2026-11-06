//! `ies processo reabrir NUP [--confirmar]` (contracts/cli/processo-reabrir.md):
//! NUP inválido sai com 2 antes de qualquer arquivo; prévia sem escrita;
//! com `--confirmar`, uma escrita conferida por leitura; efeito incerto, 3.
//! A saída nunca traz URL, `infra_hash` ou cookie.
use crate::arvore_processo::FormaDaReabertura;
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{
    com_sessao, erro_de_uso, json_base_da_falha, json_da_falha, lista_humana,
    publicar_valor_da_falha,
};
use crate::processo_reabrir::{
    ErroReabrir, PedidoDaReabertura, ProcessoReaberto, reabrir_processo,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
use serde_json::{Value, json};
use std::process::ExitCode;

/// `Some` só para `ies processo reabrir`.
pub(crate) fn tentar_reabrir(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Reabrir { numero, confirmar }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    Some(match NumeroProcesso::novo(numero) {
        Ok(nup) => reabrir(cli, &nup, *confirmar),
        Err(erro) => erro_de_uso(cli.json, erro.subcodigo()),
    })
}

fn reabrir(cli: &Cli, nup: &NumeroProcesso, confirmar: bool) -> ExitCode {
    com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDaReabertura {
            nup,
            unidade: cli.unidade.as_deref(),
            confirmar,
        };
        match reabrir_processo(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(feito) => {
                if cli.json {
                    if let Err(erro) = crate::saida::imprimir_json(&resultado_reabertura(&feito)) {
                        eprintln!("IES: falha ao emitir JSON: {erro}");
                        return std::process::ExitCode::from(1);
                    }
                } else {
                    println!("{}", humano(&feito));
                }
                ExitCode::SUCCESS
            }
            Err(ErroReabrir::Abertura(erro)) => saida_erro::erro_fonte_http(
                cli.json,
                erro.subcodigo(),
                erro.mensagem(),
                erro.status_http(),
            ),
            Err(erro) => publicar_valor_da_falha(cli.json, &json_de_falha(&erro), erro.saida()),
        }
    })
}

/// JSON do sucesso (prévia, nada a fazer, ou reaberto e conferido).
#[expect(
    clippy::struct_excessive_bools,
    reason = "quatro campos independentes exigidos pelo contrato JSON"
)]
#[derive(serde::Serialize)]
struct ResultadoReabertura<'a> {
    aberto_em: &'a [String],
    alterado: bool,
    confirmado: bool,
    depois: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    falha_do_envio: Option<crate::saida_erro::FalhaPublica>,
    forma: Option<&'static str>,
    id_procedimento: &'a str,
    nup: &'a str,
    origem: &'static str,
    reabertura_oferecida: bool,
    reconciliado_por_leitura: bool,
    requisicoes: u8,
    unidade: &'a str,
}

fn resultado_reabertura(feito: &ProcessoReaberto) -> ResultadoReabertura<'_> {
    ResultadoReabertura {
        nup: &feito.nup,
        id_procedimento: &feito.id_procedimento,
        unidade: &feito.unidade,
        origem: "web",
        confirmado: feito.confirmado,
        alterado: feito.alterado,
        reabertura_oferecida: feito.oferecido,
        forma: feito.forma.map(FormaDaReabertura::como_texto),
        aberto_em: &feito.aberto_em,
        depois: feito.depois.as_deref(),
        reconciliado_por_leitura: feito.reconciliado_por.is_some(),
        requisicoes: feito.requisicoes,
        falha_do_envio: feito
            .reconciliado_por
            .as_ref()
            .map(crate::processo_concluir_cli::falha_do_envio_publica),
    }
}

/// Compatibilidade com os testes: o runtime usa serialização direta.
#[cfg(test)]
pub(crate) fn json_de_sucesso(feito: &ProcessoReaberto) -> Value {
    serde_json::to_value(resultado_reabertura(feito))
        .expect("a projeção de sucesso só possui valores JSON")
}

/// JSON da falha depois da abertura.
pub(crate) fn json_de_falha(erro: &ErroReabrir) -> Value {
    let mut valor = json_base_da_falha(
        erro.codigo(),
        erro.subcodigo(),
        erro.mensagem(),
        erro.status_http(),
    );
    let Some(objeto) = valor.as_object_mut() else {
        return valor;
    };
    match erro {
        ErroReabrir::SituacaoDoProcessoNaoLidaAntesDeReabrir { motivo } => {
            objeto.insert("motivo".to_owned(), json!(motivo.como_texto()));
        }
        ErroReabrir::UnidadeDaSessaoJaNaSituacao { aberto_em }
        | ErroReabrir::ReaberturaNaoPublicadaNaBarra { aberto_em } => {
            objeto.insert("aberto_em".to_owned(), json!(aberto_em));
        }
        ErroReabrir::ReaberturaNaoAplicada {
            aberto_em,
            depois,
            falha_do_envio,
        } => {
            objeto.insert("aberto_em".to_owned(), json!(aberto_em));
            objeto.insert("depois".to_owned(), json!(depois));
            if let Some(falha) = falha_do_envio {
                objeto.insert("falha_do_envio".to_owned(), json_da_falha(falha));
            }
        }
        ErroReabrir::EfeitoIncertoNaReabertura {
            aberto_em,
            falha_do_envio,
            verificacao,
        } => {
            objeto.insert("aberto_em".to_owned(), json!(aberto_em));
            objeto.insert("verificacao".to_owned(), json!(verificacao.motivo()));
            if let Some(falha) = falha_do_envio {
                objeto.insert("falha_do_envio".to_owned(), json_da_falha(falha));
            }
        }
        _ => {}
    }
    valor
}

/// Saída humana do sucesso.
pub(crate) fn humano(feito: &ProcessoReaberto) -> String {
    let refs = |v: &[String]| lista_humana(&v.iter().map(String::as_str).collect::<Vec<_>>());
    let mut linhas = vec![
        format!("Processo {} na unidade {}", feito.nup, feito.unidade),
        format!("Aberto em: {}", refs(&feito.aberto_em)),
    ];
    match (feito.oferecido, feito.confirmado, &feito.depois) {
        (false, _, _) => linhas.push(
            "Nada a fazer: o processo já está aberto nesta unidade e a barra não oferece reabrir."
                .to_owned(),
        ),
        (true, false, _) => {
            linhas.push("Ação na barra: reabrir".to_owned());
            linhas.push(format!(
                "Depois de reabrir: aberto também em {}",
                feito.unidade
            ));
            linhas.push("Para reabrir, repita com --confirmar".to_owned());
        }
        (true, true, depois) => {
            let depois = depois.as_deref().unwrap_or_default();
            linhas.push(format!("Reaberto. Aberto em, relido: {}", refs(depois)));
            if let Some(falha) = &feito.reconciliado_por {
                linhas.push(format!(
                    "A resposta não foi a esperada ({}); a leitura confirmou.",
                    falha.subcodigo()
                ));
            }
        }
    }
    linhas.push(format!("Requisições: {}", feito.requisicoes));
    linhas.join("\n")
}

#[cfg(test)]
#[path = "processo_reabrir_cli/tests.rs"]
mod tests;
