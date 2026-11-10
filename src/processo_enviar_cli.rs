//! `ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]`
//! (contracts/cli/processo-enviar.md): uso inválido sai com 2 antes de
//! qualquer arquivo; prévia sem POST de escrita; com `--confirmar`, um POST
//! conferido por leitura; falha não confirmada, saída 1. A saída nunca traz URL,
//! `infra_hash` ou cookie.
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{com_sessao, lista_humana};
use crate::processo_enviar::{ErroEnviar, PedidoDoEnvio, ProcessoEnviado, enviar_processo};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
#[cfg(test)]
use serde_json::Value;

mod humano;
pub(crate) use humano::humano;

/// `Some` só para `ies processo enviar`.
pub(crate) fn tentar_enviar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Enviar {
                numero,
                para,
                manter_aberto,
                remover_anotacoes,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = numero.clone();
    Some(enviar(
        cli,
        &nup,
        para,
        *manter_aberto,
        *remover_anotacoes,
        *confirmar,
    ))
}

/// Sigla de `--para` vazia, com controle ou com mais de 100 caracteres.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct SiglaDeDestinoInvalida;

/// A sigla de `--para`, aparada; o clap recusa a inválida (saída 2) e
/// também `--para` repetido.
pub(crate) fn sigla_de_destino(para: &str) -> Result<String, SiglaDeDestinoInvalida> {
    let sigla = para.trim();
    if sigla.is_empty() || sigla.chars().count() > 100 || sigla.chars().any(char::is_control) {
        return Err(SiglaDeDestinoInvalida);
    }
    Ok(sigla.to_owned())
}

fn enviar(
    cli: &Cli,
    nup: &NumeroProcesso,
    para: &str,
    manter_aberto: bool,
    remover_anotacoes: bool,
    confirmar: bool,
) -> anyhow::Result<()> {
    com_sessao(cli, |sessao, indice| {
        // Aprovada nas sete instalações (10/11/2026): grava também em produção.
        if remover_anotacoes
            && confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoEnviarRemovendoAnotacoes,
                sessao.origem,
            )
        {
            return Err(causa.into());
        }
        let pedido = PedidoDoEnvio {
            nup,
            unidade: cli.unidade.as_deref(),
            para,
            manter_aberto,
            remover_anotacoes,
            confirmar,
        };
        let feito = enviar_processo(sessao, &pedido, indice, &mut EnvioHttps).map_err(|erro| {
            let sugestoes = sugestoes_guardadas(&erro, sessao, indice, para);
            let erro = anyhow::Error::from(erro);
            if sugestoes.is_empty() {
                erro
            } else {
                erro.context(SugestoesGuardadas(sugestoes))
            }
        })?;
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&resultado_envio(&feito))?
            );
        } else {
            println!("{}", humano(&feito));
        }
        Ok(())
    })
}

/// Siglas do índice local parecidas com o destino recusado. Não são oferta
/// do SEI nem autorizam escrita.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct SugestoesGuardadas(pub(crate) Vec<String>);

/// Sem rede; só para destino não oferecido pelo SEI.
fn sugestoes_guardadas(
    erro: &ErroEnviar,
    sessao: &crate::processo_obter::SessaoDoProcesso<'_>,
    indice: &crate::indice_local::IndiceLocal,
    para: &str,
) -> Vec<String> {
    if !matches!(erro, ErroEnviar::DestinoNaoOferecidoPeloSei { .. }) {
        return Vec::new();
    }
    let instalacao = crate::instalacao::Instalacao::interpretar(sessao.origem);
    let ativa = crate::indice_local::unidade_dos_links(sessao.url_do_controle);
    match (instalacao, ativa) {
        (Some(instalacao), Some(ativa)) => {
            indice.sugestoes_de_unidades_guardadas(&instalacao.url_base(), &ativa, para)
        }
        _ => Vec::new(),
    }
}

/// JSON do sucesso (prévia, ou enviado e conferido).
/// O contrato externo de destino continua isolado do tipo de domínio.
#[derive(serde::Serialize)]
struct DestinoPublico<'a> {
    descricao: &'a str,
    id_unidade: &'a str,
    sigla: &'a str,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "quatro campos independentes exigidos pelo contrato JSON"
)]
#[derive(serde::Serialize)]
struct ResultadoEnvio<'a> {
    aberto_em: &'a [String],
    alterado: bool,
    confirmado: bool,
    depois: Option<&'a [String]>,
    depois_esperado: Vec<String>,
    destino: DestinoPublico<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    falha_do_envio: Option<crate::saida::FalhaPublica>,
    id_procedimento: &'a str,
    manter_aberto: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    remover_anotacoes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    anotacao_antes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    anotacao_depois: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metodo_remocao: Option<crate::processo_enviar::MetodoRemocao>,
    nup: &'a str,
    origem: &'static str,
    reconciliado_por_leitura: bool,
    requisicoes: u8,
    unidade: &'a str,
}

fn resultado_envio(feito: &ProcessoEnviado) -> ResultadoEnvio<'_> {
    ResultadoEnvio {
        nup: &feito.nup,
        id_procedimento: &feito.id_procedimento,
        unidade: &feito.unidade,
        origem: "web",
        confirmado: feito.confirmado,
        alterado: feito.alterado,
        manter_aberto: feito.manter_aberto,
        remover_anotacoes: feito.remover_anotacoes.then_some(true),
        anotacao_antes: feito.anotacao_antes,
        anotacao_depois: feito.anotacao_depois,
        metodo_remocao: feito.metodo_remocao,
        destino: DestinoPublico {
            sigla: &feito.destino.sigla,
            descricao: &feito.destino.descricao,
            id_unidade: &feito.destino.id_unidade,
        },
        aberto_em: &feito.aberto_em,
        depois_esperado: feito.depois_esperado(),
        depois: feito.depois.as_deref(),
        reconciliado_por_leitura: feito.reconciliado_por.is_some(),
        requisicoes: feito.requisicoes,
        falha_do_envio: feito
            .reconciliado_por
            .as_ref()
            .map(crate::processo_concluir_cli::falha_do_envio_publica),
    }
}

/// Compatibilidade com testes do contrato, sem Value no fluxo de produção.
#[cfg(test)]
pub(crate) fn json_de_sucesso(feito: &ProcessoEnviado) -> Value {
    serde_json::to_value(resultado_envio(feito))
        .expect("a projeção tipada do envio só possui valores JSON")
}

#[cfg(test)]
#[path = "processo_enviar_cli/tests.rs"]
mod tests;
