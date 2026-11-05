//! Link assinado de `procedimento_relacionar` na barra do processo.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

pub(crate) const ACAO_DE_RELACIONAR: &str = "procedimento_relacionar";

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroRelacionarNaBarra {
    #[causa(nome = "endereco_de_relacionar_de_outro_processo")]
    EnderecoDeOutroProcesso,
    #[causa(interna)]
    Html(crate::erro_html::ErroHtml),
}
impl ErroRelacionarNaBarra {
    pub(crate) fn subcodigo(self) -> &'static str {
        crate::erro_publico::folha(&self)
    }
}
pub(crate) fn ler_relacionar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroRelacionarNaBarra> {
    use ErroRelacionarNaBarra as E;
    let ancoras = ancoras_da_barra(html).map_err(|erro| match erro {
        FalhaDaBarra::Fora => E::Html(crate::erro_html::ErroHtml::valor(
            "barra_do_processo_fora_da_forma_observada",
            "o valor não está na forma observada",
        )),
        FalhaDaBarra::Repetidas => E::Html(crate::erro_html::ErroHtml::repetido(
            "barras_do_processo_repetidas_na_arvore",
            "o elemento aparece mais de uma vez",
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
                EnderecoRecusado::NaoReconhecido => E::Html(crate::erro_html::ErroHtml::valor(
                    "endereco_de_relacionar_nao_reconhecido",
                    "o valor não está na forma observada",
                )),
                EnderecoRecusado::DeOutroProcesso => E::EnderecoDeOutroProcesso,
            }),
        _ => Err(E::Html(crate::erro_html::ErroHtml::repetido(
            "links_de_relacionar_repetidos_na_barra",
            "o elemento aparece mais de uma vez",
        ))),
    }
}
