//! `ies processo reabrir NUP [--confirmar]` (contracts/cli/processo-reabrir.md):
//! NUP inválido sai com 2 antes de qualquer arquivo; prévia sem escrita;
//! com `--confirmar`, uma escrita conferida por leitura; efeito incerto, 3.
//! A saída nunca traz URL, `infra_hash` ou cookie.
use crate::arvore_processo::FormaDaReabertura;
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{
    com_sessao, erro_de_uso, falha::FalhaDaSituacao, falha_do_envio_publica, lista_humana,
};
use crate::processo_reabrir::{
    ErroReabrir, PedidoDaReabertura, ProcessoReaberto, reabrir_processo,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
#[cfg(test)]
use serde_json::Value;
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
        Err(_) => erro_de_uso(cli.json),
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
            Err(ErroReabrir::Abertura(erro)) => {
                saida_erro::erro_fonte_http(cli.json, erro.mensagem(), erro.status_http())
            }
            Err(erro) => {
                if cli.json {
                    if let Err(falha) = crate::saida::imprimir_json_compacto(&json_de_falha(&erro))
                    {
                        eprintln!("IES: falha ao emitir JSON: {falha}");
                    }
                } else {
                    for linha in linhas_da_falha(&erro) {
                        eprintln!("{linha}");
                    }
                }
                ExitCode::from(erro.saida())
            }
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
pub(crate) fn json_de_falha(erro: &ErroReabrir) -> FalhaDaSituacao<'_> {
    let mut valor = FalhaDaSituacao::nova(erro.codigo(), erro.mensagem(), erro.status_http());
    match erro {
        ErroReabrir::SituacaoDoProcessoNaoLidaAntesDeReabrir { motivo } => {
            valor.motivo = Some(motivo.como_texto());
        }
        ErroReabrir::UnidadeDaSessaoJaNaSituacao { aberto_em }
        | ErroReabrir::ReaberturaNaoPublicadaNaBarra { aberto_em } => {
            valor.aberto_em = Some(aberto_em);
        }
        ErroReabrir::ReaberturaNaoAplicada {
            aberto_em,
            depois,
            falha_do_envio,
        } => {
            valor.aberto_em = Some(aberto_em);
            valor.depois = Some(depois);
            valor.falha_do_envio = falha_do_envio.as_ref().map(falha_do_envio_publica);
        }
        ErroReabrir::EfeitoIncertoNaReabertura {
            aberto_em,
            falha_do_envio,
            verificacao,
        } => {
            valor.aberto_em = Some(aberto_em);
            valor.verificacao = Some(verificacao.motivo());
            valor.falha_do_envio = falha_do_envio.as_ref().map(falha_do_envio_publica);
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
                    "A resposta não foi a esperada ({falha:?}); a leitura confirmou."
                ));
            }
        }
    }
    linhas.push(format!("Requisições: {}", feito.requisicoes));
    linhas.join("\n")
}

/// Mensagens humanas a partir do erro de reabertura; não dependem
/// da montagem JSON nem reinterpretam subcódigos como variantes.
fn linhas_da_falha(erro: &ErroReabrir) -> Vec<String> {
    use ErroReabrir as E;

    let mut linhas = vec![format!("IES: {}", erro.mensagem())];
    let aberto_em = match erro {
        E::UnidadeDaSessaoJaNaSituacao { aberto_em }
        | E::ReaberturaNaoPublicadaNaBarra { aberto_em }
        | E::ReaberturaNaoAplicada { aberto_em, .. }
        | E::EfeitoIncertoNaReabertura { aberto_em, .. } => Some(aberto_em.as_slice()),
        _ => None,
    };
    let depois = match erro {
        E::ReaberturaNaoAplicada { depois, .. } => Some(depois.as_slice()),
        _ => None,
    };
    for (nome, unidades) in [("aberto_em", aberto_em), ("depois", depois)] {
        if let Some(unidades) = unidades {
            let siglas: Vec<_> = unidades.iter().map(String::as_str).collect();
            linhas.push(format!("IES: {nome}: {}", lista_humana(&siglas)));
        }
    }
    let falha_do_envio = match erro {
        E::ReaberturaNaoAplicada { falha_do_envio, .. }
        | E::EfeitoIncertoNaReabertura { falha_do_envio, .. } => falha_do_envio.as_ref(),
        _ => None,
    };
    if let Some(falha) = falha_do_envio {
        linhas.push(format!("IES: resposta da escrita: {falha:?}"));
    }
    if let E::EfeitoIncertoNaReabertura { verificacao, .. } = erro
        && let Some(motivo) = verificacao.motivo()
    {
        linhas.push(format!("IES: verificação: {motivo}"));
    }
    linhas
}

#[cfg(test)]
#[path = "processo_reabrir_cli/tests.rs"]
mod tests;
