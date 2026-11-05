//! Ações de sobrestar e retomar publicadas na barra do nó PROCESSO (#269).
//! Só URLs servidas, com ID e `infra_hash` únicos; nunca executar JavaScript.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::{EnderecoDeEscrita, tela_do_no_da_raiz};
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

const ACAO_SOBRESTAR: &str = "procedimento_sobrestar";
const ONCLICK_RETOMAR: &str = "removerSobrestamentoProcesso();";

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroAcaoSobrestar {
    #[causa(nome = "endereco_do_sobrestamento_de_outro_processo")]
    OutroProcesso,
    #[causa(interna)]
    Html(crate::erro_html::ErroHtml),
}

impl ErroAcaoSobrestar {}

fn mapear_barra(_: FalhaDaBarra) -> ErroAcaoSobrestar {
    ErroAcaoSobrestar::Html(crate::erro_html::ErroHtml::valor(
        "barra_do_sobrestamento_nao_interpretada",
        "o valor não está na forma observada",
    ))
}

/// Link para o formulário de sobrestamento: não é uma escrita.
pub(crate) fn ler_sobrestar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar> {
    let ancoras = ancoras_da_barra(html).map_err(mapear_barra)?;
    let links: Vec<&str> = ancoras
        .iter()
        .filter_map(|a| a.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(ACAO_SOBRESTAR))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [link] => conferir_endereco_relativo(instalacao, link, ACAO_SOBRESTAR, id)
            .map(EnderecoDeEscrita::novo)
            .map(Some)
            .map_err(|erro| match erro {
                EnderecoRecusado::NaoReconhecido => {
                    ErroAcaoSobrestar::Html(crate::erro_html::ErroHtml::valor(
                        "endereco_do_sobrestamento_nao_reconhecido",
                        "o valor não está na forma observada",
                    ))
                }
                EnderecoRecusado::DeOutroProcesso => ErroAcaoSobrestar::OutroProcesso,
            }),
        _ => Err(ErroAcaoSobrestar::Html(
            crate::erro_html::ErroHtml::repetido(
                "acoes_de_sobrestamento_repetidas_na_barra",
                "o elemento aparece mais de uma vez",
            ),
        )),
    }
}

/// Retomar é uma função no HTML servido da raiz; sua URL só pode ser
/// procurada na tela do nó, depois de uma confirmação explícita.
pub(crate) fn ler_retomar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar> {
    let ancoras = ancoras_da_barra(html).map_err(mapear_barra)?;
    match ancoras
        .iter()
        .filter(|a| a.onclick.as_deref() == Some(ONCLICK_RETOMAR))
        .count()
    {
        0 => Ok(None),
        1 => tela_do_no_da_raiz(instalacao, html, id)
            .map(Some)
            .ok_or(ErroAcaoSobrestar::Html(crate::erro_html::ErroHtml::valor(
                "tela_do_no_da_retomada_nao_reconhecida",
                "o valor não está na forma observada",
            ))),
        _ => Err(ErroAcaoSobrestar::Html(
            crate::erro_html::ErroHtml::repetido(
                "acoes_de_sobrestamento_repetidas_na_barra",
                "o elemento aparece mais de uma vez",
            ),
        )),
    }
}
