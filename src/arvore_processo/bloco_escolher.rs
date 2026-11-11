//! Link `bloco_escolher` ("Incluir em Bloco de Assinatura") na barra
//! (`Nos[i].acoes`) de um `DOCUMENTO` (#381; contracts/cli/bloco-incluir.md).
//! Forma observada no card `bloco-incluir-anotar-retirar-unb`: o endereço
//! assinado carrega o `id_procedimento` e o `id_documento` do nó. Nenhum
//! link é montado (ADR 0003): sem âncora, a ação não é oferecida.
use super::barra_do_documento::{ErroBarraDoDocumento, barra_do_no};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo_por};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::menciona_acao;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::atributos;
use crate::instalacao::Instalacao;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroBlocoNaBarra {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    EnderecoDeOutroDocumento,
    Barra(#[from] ErroBarraDoDocumento),
    BarraDoDocumentoForaDaFormaObservada,
    EnderecoDeEscolherBlocoNaoReconhecido,
    LinksDeEscolherBlocoRepetidosNaBarra,
}

/// O endereço de `bloco_escolher` da barra do documento `id_documento`;
/// `None` quando a barra não oferece a ação. Dois links são recusa.
pub(crate) fn ler_bloco_escolher_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id_documento: &str,
    id_procedimento: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroBlocoNaBarra> {
    let fora = || ErroBlocoNaBarra::BarraDoDocumentoForaDaFormaObservada;
    let Some(texto) = barra_do_no(html, id_documento)? else {
        return Ok(None);
    };
    let dom = crate::tela_html::interpretar(&texto);
    let parser = dom.parser();
    let mut links = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = atributos(tag).map_err(ErroBlocoNaBarra::Atributo)?;
        let Some(bruto) = lidos.valor("href").map_err(ErroBlocoNaBarra::Atributo)? else {
            continue;
        };
        let href = decodificar_atributo(bruto).ok_or_else(fora)?;
        if menciona_acao(&href, AcaoDoSei::BlocoEscolher) {
            links.push(href);
        }
    }
    let href = match links.as_slice() {
        [] => return Ok(None),
        [href] => href,
        _ => return Err(ErroBlocoNaBarra::LinksDeEscolherBlocoRepetidosNaBarra),
    };
    let recusa = |r| match r {
        EnderecoRecusado::NaoReconhecido => ErroBlocoNaBarra::EnderecoDeEscolherBlocoNaoReconhecido,
        EnderecoRecusado::DeOutroProcesso => ErroBlocoNaBarra::EnderecoDeOutroDocumento,
    };
    conferir_endereco_relativo_por(
        instalacao,
        href,
        AcaoDoSei::BlocoEscolher,
        "id_procedimento",
        id_procedimento,
    )
    .map_err(recusa)?;
    conferir_endereco_relativo_por(
        instalacao,
        href,
        AcaoDoSei::BlocoEscolher,
        "id_documento",
        id_documento,
    )
    .map(|url| Some(EnderecoDeEscrita::novo(url)))
    .map_err(recusa)
}
