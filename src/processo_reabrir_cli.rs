//! `ies processo reabrir NUP [--confirmar]` (contracts/cli/processo-reabrir.md):
//! NUP inválido sai com 2 antes de qualquer arquivo; prévia sem escrita;
//! com `--confirmar`, uma escrita conferida por leitura; efeito incerto, 3.
//! A saída nunca traz URL, `infra_hash` ou cookie.
use crate::arvore_processo::FormaDaReabertura;
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{com_sessao, lista_humana};
use crate::processo_reabrir::{
    DesfechoDaReabertura, PedidoDaReabertura, ProcessoReaberto, reabrir_processo,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
#[cfg(test)]
use serde_json::Value;

/// `Some` só para `ies processo reabrir`.
pub(crate) fn tentar_reabrir(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Reabrir { numero, confirmar }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = numero.clone();
    Some(reabrir(cli, &nup, *confirmar))
}

fn reabrir(cli: &Cli, nup: &NumeroProcesso, confirmar: bool) -> anyhow::Result<()> {
    com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDaReabertura {
            nup,
            unidade: cli.unidade.as_deref(),
            confirmar,
        };
        let feito = reabrir_processo(sessao, &pedido, indice, &mut EnvioHttps)?;
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&resultado_reabertura(&feito))?
            );
        } else {
            println!("{}", humano(&feito));
        }
        Ok(())
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
    falha_do_envio: Option<String>,
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
        confirmado: feito.confirmado(),
        alterado: feito.alterado(),
        reabertura_oferecida: feito.oferecido(),
        forma: feito.forma().map(FormaDaReabertura::como_texto),
        aberto_em: &feito.aberto_em,
        depois: feito.depois(),
        reconciliado_por_leitura: feito.reconciliado_por().is_some(),
        requisicoes: feito.requisicoes,
        falha_do_envio: feito.reconciliado_por().map(|f| format!("{f:?}")),
    }
}

/// Compatibilidade com os testes: o runtime usa serialização direta.
#[cfg(test)]
pub(crate) fn json_de_sucesso(feito: &ProcessoReaberto) -> Value {
    serde_json::to_value(resultado_reabertura(feito))
        .expect("a projeção de sucesso só possui valores JSON")
}

/// Saída humana do sucesso.
pub(crate) fn humano(feito: &ProcessoReaberto) -> String {
    let refs = |v: &[String]| lista_humana(&v.iter().map(String::as_str).collect::<Vec<_>>());
    let mut linhas = vec![
        format!("Processo {} na unidade {}", feito.nup, feito.unidade),
        format!("Aberto em: {}", refs(&feito.aberto_em)),
    ];
    match &feito.desfecho {
        DesfechoDaReabertura::JaAbertoNaUnidade { .. } => linhas.push(
            "Nada a fazer: o processo já está aberto nesta unidade e a barra não oferece reabrir."
                .to_owned(),
        ),
        DesfechoDaReabertura::Previa => {
            linhas.push("Ação na barra: reabrir".to_owned());
            linhas.push(format!(
                "Depois de reabrir: aberto também em {}",
                feito.unidade
            ));
            linhas.push("Para reabrir, repita com --confirmar".to_owned());
        }
        DesfechoDaReabertura::Reaberto {
            depois,
            reconciliado_por,
            ..
        } => {
            linhas.push(format!("Reaberto. Aberto em, relido: {}", refs(depois)));
            if let Some(falha) = reconciliado_por {
                linhas.push(format!(
                    "A resposta não foi a esperada ({falha:?}); a leitura confirmou."
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
