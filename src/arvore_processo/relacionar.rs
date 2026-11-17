//! Link assinado de `procedimento_relacionar` na barra do processo.
use super::barra_do_processo::ancoras_da_barra;
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::menciona_acao;
use crate::instalacao::Instalacao;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroRelacionarNaBarra {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    EnderecoDeOutroProcesso,
    EnderecoDeRelacionarNaoReconhecido,
    LinksDeRelacionarRepetidosNaBarra,
}
pub(crate) fn ler_relacionar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroRelacionarNaBarra> {
    let ancoras = ancoras_da_barra(html)?;
    let links: Vec<_> = ancoras
        .iter()
        .filter_map(|a| a.href.as_deref())
        .filter(|href| menciona_acao(href, AcaoDoSei::ProcedimentoRelacionar))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => {
            conferir_endereco_relativo(instalacao, href, AcaoDoSei::ProcedimentoRelacionar, id)
                .map(|url| Some(EnderecoDeEscrita::novo(url)))
                .map_err(|recusa| match recusa {
                    EnderecoRecusado::NaoReconhecido => {
                        ErroRelacionarNaBarra::EnderecoDeRelacionarNaoReconhecido
                    }
                    EnderecoRecusado::DeOutroProcesso => {
                        ErroRelacionarNaBarra::EnderecoDeOutroProcesso
                    }
                })
        }
        _ => Err(ErroRelacionarNaBarra::LinksDeRelacionarRepetidosNaBarra),
    }
}
