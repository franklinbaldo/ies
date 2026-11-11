//! Link observado da ação `procedimento_anexar` na barra do PROCESSO.
//! Endereço assinado validado pela mesma peça das outras escritas.
use super::barra_do_processo::{acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::acao_do_sei::AcaoDoSei;
use crate::instalacao::Instalacao;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAnexarNaBarra {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    EnderecoDeOutroProcesso,
    EnderecoDeAnexarNaoReconhecido,
    LinksDeAnexarRepetidosNaBarra,
}
pub(crate) fn ler_anexar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAnexarNaBarra> {
    let ancoras = ancoras_da_barra(html)?;
    let links: Vec<_> = ancoras
        .iter()
        .filter_map(|ancora| ancora.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(AcaoDoSei::ProcedimentoAnexar))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => conferir_endereco_relativo(instalacao, href, AcaoDoSei::ProcedimentoAnexar, id)
            .map(|url| Some(EnderecoDeEscrita::novo(url)))
            .map_err(|recusa| match recusa {
                EnderecoRecusado::NaoReconhecido => {
                    ErroAnexarNaBarra::EnderecoDeAnexarNaoReconhecido
                }
                EnderecoRecusado::DeOutroProcesso => ErroAnexarNaBarra::EnderecoDeOutroProcesso,
            }),
        _ => Err(ErroAnexarNaBarra::LinksDeAnexarRepetidosNaBarra),
    }
}
