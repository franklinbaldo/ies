//! Linha da pesquisa de processos cujo acerto é um documento (Goiás,
//! `processo-busca-go`, passos 2 e 4): o link do processo leva
//! `id_documento`, e o nome e o número SEI do documento são
//! `a.protocoloNormal` de `documento_visualizar` com o mesmo `id_documento`.
//! Só essa forma é aceita; o documento não é publicado.
use super::ErroResultadoPesquisa;
use crate::html_atributos::valor_decodificado;

/// Campos da consulta de um `href` `controlador.php?` já decodificado
/// (`valor_decodificado`): o `&amp;` do HTML chega como `&`.
fn campos(href: &str) -> Option<Vec<&str>> {
    let consulta = href.strip_prefix("controlador.php?")?;
    Some(consulta.split('&').collect())
}

/// O único valor de `nome`, decimal e não vazio.
fn unico_decimal<'a>(campos: &[&'a str], nome: &str) -> Option<&'a str> {
    let prefixo = format!("{nome}=");
    let mut valores = campos
        .iter()
        .filter_map(|c| c.strip_prefix(prefixo.as_str()));
    let valor = valores.next()?;
    let decimal = !valor.is_empty() && valor.bytes().all(|b| b.is_ascii_digit());
    (decimal && valores.next().is_none()).then_some(valor)
}

/// `id_documento` de um link que só abre o documento: uma `acao`
/// (`documento_visualizar`) e um `id_documento` decimal.
pub(super) fn documento_visualizado(href: &str) -> Option<&str> {
    let campos = campos(href)?;
    let acoes: Vec<&str> = campos
        .iter()
        .filter_map(|c| c.strip_prefix("acao="))
        .collect();
    if acoes != ["documento_visualizar"] {
        return None;
    }
    unico_decimal(&campos, "id_documento")
}

/// `id_documento` único e decimal do link do processo, se houver.
pub(super) fn documento_do_processo(href: &str) -> Option<&str> {
    unico_decimal(&campos(href)?, "id_documento")
}

/// Separa os `a.protocoloNormal` da linha em candidatos a link do processo
/// e `id_documento` dos links que só abrem um documento.
pub(super) fn separar(
    parser: &tl::Parser<'_>,
    links: Vec<tl::NodeHandle>,
) -> Result<(Vec<tl::NodeHandle>, Vec<String>), ErroResultadoPesquisa> {
    let mut candidatos = Vec::new();
    let mut documentos = Vec::new();
    for no in links {
        let href = match no.get(parser).and_then(tl::Node::as_tag) {
            Some(link) => valor_decodificado(link, "href")
                .map_err(ErroResultadoPesquisa::HrefDoLinkDaLinhaIlegivel)?,
            None => None,
        };
        match href.as_deref().and_then(documento_visualizado) {
            Some(id) => documentos.push(id.to_owned()),
            None => candidatos.push(no),
        }
    }
    Ok((candidatos, documentos))
}
