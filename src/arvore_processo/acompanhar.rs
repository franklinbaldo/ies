//! Link assinado de Acompanhamento Especial publicado no nó PROCESSO (#254).
//! A extração é estrutural, sem executar JavaScript nem montar URL.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

pub(crate) const ACAO: &str = "acompanhamento_gerenciar";

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAcompanharNaBarra {
    LinkDeOutroProcesso,
    BarraDeAcompanhamentoForaDaForma,
    BarraDeAcompanhamentoRepetida,
    LinkDeAcompanhamentoNaoReconhecido,
    LinksDeAcompanhamentoRepetidos,
}

pub(crate) fn ler_acompanhar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcompanharNaBarra> {
    let ancoras = ancoras_da_barra(html).map_err(|falha| match falha {
        FalhaDaBarra::Fora => ErroAcompanharNaBarra::BarraDeAcompanhamentoForaDaForma,
        FalhaDaBarra::Repetidas => ErroAcompanharNaBarra::BarraDeAcompanhamentoRepetida,
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
                    ErroAcompanharNaBarra::LinkDeAcompanhamentoNaoReconhecido
                }
                EnderecoRecusado::DeOutroProcesso => ErroAcompanharNaBarra::LinkDeOutroProcesso,
            }),
        _ => Err(ErroAcompanharNaBarra::LinksDeAcompanhamentoRepetidos),
    }
}
