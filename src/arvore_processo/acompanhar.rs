//! Link assinado de Acompanhamento Especial publicado no nó PROCESSO (#254).
//! A extração é estrutural, sem executar JavaScript nem montar URL.
use super::barra_do_processo::{acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::acao_do_sei::AcaoDoSei;
use crate::instalacao::Instalacao;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAcompanharNaBarra {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    LinkDeOutroProcesso,
    LinkDeAcompanhamentoNaoReconhecido,
    LinksDeAcompanhamentoRepetidos,
}

pub(crate) fn ler_acompanhar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcompanharNaBarra> {
    let ancoras = ancoras_da_barra(html)?;
    let links: Vec<&str> = ancoras
        .iter()
        .filter_map(|ancora| ancora.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(AcaoDoSei::AcompanhamentoGerenciar))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => {
            conferir_endereco_relativo(instalacao, href, AcaoDoSei::AcompanhamentoGerenciar, id)
                .map(|url| Some(EnderecoDeEscrita::novo(url)))
                .map_err(|recusa| match recusa {
                    EnderecoRecusado::NaoReconhecido => {
                        ErroAcompanharNaBarra::LinkDeAcompanhamentoNaoReconhecido
                    }
                    EnderecoRecusado::DeOutroProcesso => ErroAcompanharNaBarra::LinkDeOutroProcesso,
                })
        }
        _ => Err(ErroAcompanharNaBarra::LinksDeAcompanhamentoRepetidos),
    }
}
