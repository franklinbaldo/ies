//! Link assinado de `procedimento_relacionar` na barra do processo.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::erro_html::ErroHtml;
use crate::instalacao::Instalacao;

pub(crate) const ACAO_DE_RELACIONAR: &str = "procedimento_relacionar";

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroRelacionarNaBarra {
    EnderecoDeOutroProcesso,
    Html(#[from] ErroHtml),
}
pub(crate) fn ler_relacionar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroRelacionarNaBarra> {
    let ancoras = ancoras_da_barra(html).map_err(|erro| match erro {
        FalhaDaBarra::Fora => ErroRelacionarNaBarra::Html(ErroHtml::valor(
            "barra_do_processo_fora_da_forma_observada",
        )),
        FalhaDaBarra::Repetidas => ErroRelacionarNaBarra::Html(ErroHtml::repetido(
            "barras_do_processo_repetidas_na_arvore",
        )),
    })?;
    let links: Vec<_> = ancoras
        .iter()
        .filter_map(|a| a.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(ACAO_DE_RELACIONAR))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => conferir_endereco_relativo(instalacao, href, ACAO_DE_RELACIONAR, id)
            .map(|url| Some(EnderecoDeEscrita::novo(url)))
            .map_err(|recusa| match recusa {
                EnderecoRecusado::NaoReconhecido => ErroRelacionarNaBarra::Html(ErroHtml::valor(
                    "endereco_de_relacionar_nao_reconhecido",
                )),
                EnderecoRecusado::DeOutroProcesso => ErroRelacionarNaBarra::EnderecoDeOutroProcesso,
            }),
        _ => Err(ErroHtml::repetido("links_de_relacionar_repetidos_na_barra").into()),
    }
}
