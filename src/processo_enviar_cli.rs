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

/// O clap valida e apara cada ocorrência de `--para` (saída 2 se inválida).
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
    para: &[String],
    manter_aberto: bool,
    remover_anotacoes: bool,
    confirmar: bool,
) -> anyhow::Result<()> {
    let (primeiro, outros) = para.split_first().ok_or(SiglaDeDestinoInvalida)?;
    let outros: Vec<&str> = outros.iter().map(String::as_str).collect();
    com_sessao(cli, |sessao, indice| {
        if confirmar && !outros.is_empty() {
            crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoEnviarMultiplos,
                sessao.origem,
            )?;
        }
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
            para: primeiro,
            outros: &outros,
            manter_aberto,
            remover_anotacoes,
            confirmar,
        };
        let feito = enviar_processo(sessao, &pedido, indice, &mut EnvioHttps).map_err(|erro| {
            let sugestoes = if outros.is_empty() {
                sugestoes_guardadas(&erro, sessao, indice, primeiro)
            } else {
                Vec::new()
            };
            let erro = anyhow::Error::from(erro);
            if sugestoes.is_empty() {
                erro
            } else {
                erro.context(SugestoesGuardadas(sugestoes))
            }
        })?;
        if cli.json {
            let mut valor = serde_json::to_value(resultado_envio(&feito))?;
            crate::indice_local::anexar_avisos(&mut valor, indice);
            println!("{}", serde_json::to_string_pretty(&valor)?);
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
    // Sugestões são melhor esforço: sem unidade legível, não sugere.
    let ativa = crate::indice_local::unidade_dos_links(sessao.url_do_controle).ok();
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
    #[serde(skip_serializing_if = "Option::is_none")]
    depois: Option<&'a [String]>,
    depois_esperado: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    destino: Option<DestinoPublico<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    destinos: Option<Vec<DestinoPublico<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    falha_do_envio: Option<String>,
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

fn destino_publico(destino: &crate::processo_enviar::Destino) -> DestinoPublico<'_> {
    DestinoPublico {
        sigla: &destino.sigla,
        descricao: &destino.descricao,
        id_unidade: &destino.id_unidade,
    }
}

fn resultado_envio(feito: &ProcessoEnviado) -> ResultadoEnvio<'_> {
    ResultadoEnvio {
        nup: &feito.nup,
        id_procedimento: &feito.id_procedimento,
        unidade: &feito.unidade,
        origem: "web",
        confirmado: feito.confirmado(),
        alterado: feito.alterado(),
        manter_aberto: feito.manter_aberto,
        remover_anotacoes: feito.remover_anotacoes().then_some(true),
        anotacao_antes: feito.anotacao_antes(),
        anotacao_depois: feito.anotacao_depois(),
        metodo_remocao: feito.metodo_remocao(),
        destino: match feito.destinos.as_slice() {
            [destino] => Some(destino_publico(destino)),
            _ => None,
        },
        destinos: (feito.destinos.len() > 1)
            .then(|| feito.destinos.iter().map(destino_publico).collect()),
        aberto_em: &feito.aberto_em,
        depois_esperado: feito.depois_esperado(),
        depois: feito.depois(),
        reconciliado_por_leitura: feito.reconciliado_por().is_some(),
        requisicoes: feito.requisicoes,
        falha_do_envio: feito.reconciliado_por().map(|f| format!("{f:?}")),
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
