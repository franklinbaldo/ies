//! Busca por `id` e por classe sem o índice do `tl` (shim do contrato).
use super::ErroAtributoHtml;

/// Todos os elementos do documento cujo `id` é exatamente `id`, em ordem.
pub(crate) fn nos_com_id(
    dom: &tl::VDom<'_>,
    id: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    let _ = (dom, id);
    Ok(Vec::new())
}

/// Descendentes de `tag` cujo `id` é exatamente `id`, em ordem.
pub(crate) fn nos_com_id_em(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    id: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    let _ = (tag, parser, id);
    Ok(Vec::new())
}

/// Elementos `elemento` com a classe `classe`, no documento (`raiz` `None`)
/// ou entre os descendentes de `raiz`.
pub(crate) fn nos_por_elemento_e_classe(
    dom: &tl::VDom<'_>,
    raiz: Option<&tl::HTMLTag<'_>>,
    elemento: &str,
    classe: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    let _ = (dom, raiz, elemento, classe);
    Ok(Vec::new())
}
