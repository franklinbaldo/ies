//! Busca por `id` e por classe sem o índice do `tl`, que é montado com a
//! chave corrompida pelo defeito do 0.7.8. A estrutura (ordem do documento,
//! descendentes, nome do elemento) continua vindo do `tl`.
use super::{ErroAtributoHtml, atributos};

/// `id` exatamente igual; repetir `id` na mesma tag é recusado.
fn tem_id(tag: &tl::HTMLTag<'_>, id: &str) -> Result<bool, ErroAtributoHtml> {
    Ok(atributos(tag)?.valor("id") == Some(id))
}

fn filtrar_por_id(
    parser: &tl::Parser<'_>,
    handles: impl Iterator<Item = tl::NodeHandle>,
    id: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    let mut achados = Vec::new();
    for handle in handles {
        if let Some(tag) = handle.get(parser).and_then(tl::Node::as_tag)
            && tem_id(tag, id)?
        {
            achados.push(handle);
        }
    }
    Ok(achados)
}

fn handle(indice: usize) -> tl::NodeHandle {
    // O `tl` endereça nós por `u32`; o limite de 8 MB por resposta mantém
    // o índice muito abaixo de `u32::MAX`.
    tl::NodeHandle::new(u32::try_from(indice).unwrap_or(u32::MAX))
}

/// Todos os elementos do documento cujo `id` é exatamente `id`, em ordem.
/// Uma tag ilegível em qualquer ponto impede afirmar unicidade: erro.
pub(crate) fn nos_com_id(
    dom: &tl::VDom<'_>,
    id: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    filtrar_por_id(dom.parser(), (0..dom.nodes().len()).map(handle), id)
}

/// Descendentes de `tag` cujo `id` é exatamente `id`, em ordem.
pub(crate) fn nos_com_id_em(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    id: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    let Some((inicio, fim)) = tag.children().boundaries(parser) else {
        return Ok(Vec::new());
    };
    filtrar_por_id(parser, (inicio..=fim).map(tl::NodeHandle::new), id)
}

/// Elementos `elemento` com a classe `classe`, no documento (`raiz` `None`)
/// ou entre os descendentes de `raiz`. A seleção por nome é a do `tl`.
pub(crate) fn nos_por_elemento_e_classe(
    dom: &tl::VDom<'_>,
    raiz: Option<&tl::HTMLTag<'_>>,
    elemento: &str,
    classe: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    let parser = dom.parser();
    let candidatos: Vec<tl::NodeHandle> = match raiz {
        Some(raiz) => raiz
            .query_selector(parser, elemento)
            .into_iter()
            .flatten()
            .collect(),
        None => dom.query_selector(elemento).into_iter().flatten().collect(),
    };
    let mut achados = Vec::new();
    for candidato in candidatos {
        let Some(tag) = candidato.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        if atributos(tag)?.tem_classe(classe) {
            achados.push(candidato);
        }
    }
    Ok(achados)
}
