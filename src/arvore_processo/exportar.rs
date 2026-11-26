//! Os links de geração publicados pela barra do processo (#889).
use super::barra_do_processo::ancoras_da_barra;
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::menciona_acao;
use crate::instalacao::Instalacao;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroExportacaoNaBarra {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    Repetida,
    Endereco(#[from] EnderecoRecusado),
}

type Link = Result<Option<EnderecoDeEscrita>, ErroExportacaoNaBarra>;

pub(crate) struct LinksExportacao {
    pub(crate) pdf: Link,
    pub(crate) zip: Link,
}

fn ler(instalacao: &Instalacao, html: &str, id: &str, acao: AcaoDoSei) -> Link {
    let ancoras = ancoras_da_barra(html)?;
    let links: Vec<_> = ancoras
        .iter()
        .filter_map(|a| a.href.as_deref())
        .filter(|href| menciona_acao(href, acao))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => Ok(Some(EnderecoDeEscrita::novo(conferir_endereco_relativo(
            instalacao, href, acao, id,
        )?))),
        _ => Err(ErroExportacaoNaBarra::Repetida),
    }
}

pub(crate) fn ler_exportacao_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> LinksExportacao {
    LinksExportacao {
        pdf: ler(instalacao, html, id, AcaoDoSei::ProcedimentoGerarPdf),
        zip: ler(instalacao, html, id, AcaoDoSei::ProcedimentoGerarZip),
    }
}
