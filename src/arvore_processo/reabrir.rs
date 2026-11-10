//! A ação "Reabrir Processo" publicada na barra do nó `PROCESSO` e a URL
//! dela na tela do nó (contracts/cli/processo-reabrir.md; observação em
//! docs/como-o-sei-funciona/reabertura-do-processo.md). A barra é a mesma
//! nas sete instalações: `<a href="#" onclick="reabrirProcesso();">`. A URL
//! está na tela do nó (`arvore_visualizar`, o `href` a3 do `Nos[i]`
//! `PROCESSO`), em uma de duas formas:
//! - 4.0 (`UnB` 4.0.9, PEN1 4.0.12): o corpo de `reabrirProcesso()` na
//!   forma de três linhas da conclusão (mesmo leitor);
//! - 4.1 e 5.0 (UFRJ, PEN2, GO, PEN3, PEN4): a linha
//!   `var linkReabrirProcesso = '…';` (a função fica num `.js` servido à
//!   parte).
//!
//! Sem regex e sem JavaScript executado: só essas formas exatas.
use super::barra_do_processo::ancoras_da_barra;
use super::concluir::{EnderecoDeEscrita, FuncaoForaDaForma, tela_do_no_da_raiz, urls_das_funcoes};
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

/// A `acao` da reabertura.
pub(crate) const ACAO_DE_REABRIR: &str = "procedimento_reabrir";
const ONCLICK: &str = "reabrirProcesso();";
const VARIAVEL: &str = "var linkReabrirProcesso";
const VARIAVEL_ATE_A_URL: &str = "var linkReabrirProcesso = '";

/// A barra oferece reabrir: a tela do nó, onde está a URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReabrirNaBarra {
    pub(crate) tela_do_no: EnderecoDeEscrita,
}

/// Recusas da leitura da ação de reabrir na barra do processo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroReabrirNaBarra {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    EnderecoDaTelaDoNoNaoReconhecido,
    FuncoesDeReabrirRepetidasNaBarra,
}

/// A ação de reabrir da barra do processo `id`; `Ok(None)` se a barra não
/// oferece reabrir.
pub(crate) fn ler_reabrir_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<ReabrirNaBarra>, ErroReabrirNaBarra> {
    let ancoras = ancoras_da_barra(html)?;
    match ancoras
        .iter()
        .filter(|a| a.onclick.as_deref() == Some(ONCLICK))
        .count()
    {
        0 => Ok(None),
        1 => tela_do_no_da_raiz(instalacao, html, id)
            .map(|tela_do_no| Some(ReabrirNaBarra { tela_do_no }))
            .ok_or(ErroReabrirNaBarra::EnderecoDaTelaDoNoNaoReconhecido),
        _ => Err(ErroReabrirNaBarra::FuncoesDeReabrirRepetidasNaBarra),
    }
}

/// Onde a tela do nó publicou a URL de reabrir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FormaDaReabertura {
    /// O corpo de `reabrirProcesso()` (SEI 4.0).
    Funcao,
    /// `var linkReabrirProcesso = '…';` (SEI 4.1 e 5.0).
    Variavel,
}

impl FormaDaReabertura {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::Funcao => "funcao",
            Self::Variavel => "variavel",
        }
    }
}

/// Recusas da leitura da URL de reabrir na tela do nó.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroEnderecoDeReabrir {
    EnderecoDaReaberturaDeOutroProcesso,
    EnderecoDaReaberturaNaoReconhecido,
    EnderecoDeReabrirAusenteNaTelaDoNo,
    EnderecoDeReabrirForaDaFormaObservada,
    EnderecosDeReabrirRepetidosNaTelaDoNo,
}

/// A URL de reabrir na tela do nó do processo `id`, conferida, e a forma
/// em que estava.
pub(crate) fn ler_endereco_de_reabrir(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<(EnderecoDeEscrita, FormaDaReabertura), ErroEnderecoDeReabrir> {
    let funcoes = urls_das_funcoes(html, "reabrirProcesso").map_err(|FuncaoForaDaForma| {
        ErroEnderecoDeReabrir::EnderecoDeReabrirForaDaFormaObservada
    })?;
    let mut variaveis = Vec::new();
    for linha in html.lines().map(str::trim) {
        if !linha.starts_with(VARIAVEL) {
            continue;
        }
        let url = linha
            .strip_prefix(VARIAVEL_ATE_A_URL)
            .and_then(|resto| resto.strip_suffix("';"))
            .ok_or(ErroEnderecoDeReabrir::EnderecoDeReabrirForaDaFormaObservada)?;
        variaveis.push(url);
    }
    let (url, forma) = match (funcoes.as_slice(), variaveis.as_slice()) {
        ([], []) => {
            return Err(ErroEnderecoDeReabrir::EnderecoDeReabrirAusenteNaTelaDoNo);
        }
        ([url], []) => (*url, FormaDaReabertura::Funcao),
        ([], [url]) => (*url, FormaDaReabertura::Variavel),
        _ => return Err(ErroEnderecoDeReabrir::EnderecosDeReabrirRepetidosNaTelaDoNo),
    };
    conferir_endereco_relativo(instalacao, url, ACAO_DE_REABRIR, id)
        .map(|url| (EnderecoDeEscrita::novo(url), forma))
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => {
                ErroEnderecoDeReabrir::EnderecoDaReaberturaNaoReconhecido
            }
            EnderecoRecusado::DeOutroProcesso => {
                ErroEnderecoDeReabrir::EnderecoDaReaberturaDeOutroProcesso
            }
        })
}
