//! Link assinado de Acompanhamento Especial publicado no nó PROCESSO (#254).
//! A extração é estrutural, sem executar JavaScript nem montar URL.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::erro_html::ErroHtml;
use crate::instalacao::Instalacao;

pub(crate) const ACAO: &str = "acompanhamento_gerenciar";

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroAcompanharNaBarra {
    LinkDeOutroProcesso,
    #[causa(transparente)]
    Html(ErroHtml),
}

pub(crate) fn ler_acompanhar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcompanharNaBarra> {
    use ErroAcompanharNaBarra as E;
    let ancoras = ancoras_da_barra(html).map_err(|falha| match falha {
        FalhaDaBarra::Fora => E::Html(ErroHtml::valor("barra_de_acompanhamento_fora_da_forma")),
        FalhaDaBarra::Repetidas => E::Html(ErroHtml::repetido("barra_de_acompanhamento_repetida")),
    })?;
    let links: Vec<&str> = ancoras
        .iter()
        .filter_map(|ancora| ancora.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(ACAO))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => conferir_endereco_relativo(instalacao, href, ACAO, id)
            .map(|url| Some(EnderecoDeEscrita::novo(url)))
            .map_err(|recusa| match recusa {
                EnderecoRecusado::NaoReconhecido => {
                    E::Html(ErroHtml::valor("link_de_acompanhamento_nao_reconhecido"))
                }
                EnderecoRecusado::DeOutroProcesso => E::LinkDeOutroProcesso,
            }),
        _ => Err(ErroHtml::repetido("links_de_acompanhamento_repetidos").into()),
    }
}
