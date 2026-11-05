//! Link assinado de Acompanhamento Especial publicado no nó PROCESSO (#254).
//! A extração é estrutural, sem executar JavaScript nem montar URL.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

pub(crate) const ACAO: &str = "acompanhamento_gerenciar";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroAcompanharNaBarra {
    BarraForaDaForma,
    BarraRepetida,
    LinksRepetidos,
    LinkNaoReconhecido,
    LinkDeOutroProcesso,
}

impl ErroAcompanharNaBarra {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::BarraForaDaForma => "barra_de_acompanhamento_fora_da_forma",
            Self::BarraRepetida => "barra_de_acompanhamento_repetida",
            Self::LinksRepetidos => "links_de_acompanhamento_repetidos",
            Self::LinkNaoReconhecido => "link_de_acompanhamento_nao_reconhecido",
            Self::LinkDeOutroProcesso => "link_de_acompanhamento_de_outro_processo",
        }
    }
}

pub(crate) fn ler_acompanhar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcompanharNaBarra> {
    use ErroAcompanharNaBarra as E;
    let ancoras = ancoras_da_barra(html).map_err(|falha| match falha {
        FalhaDaBarra::Fora => E::BarraForaDaForma,
        FalhaDaBarra::Repetidas => E::BarraRepetida,
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
                EnderecoRecusado::NaoReconhecido => E::LinkNaoReconhecido,
                EnderecoRecusado::DeOutroProcesso => E::LinkDeOutroProcesso,
            }),
        _ => Err(E::LinksRepetidos),
    }
}
