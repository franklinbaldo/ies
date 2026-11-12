//! Transitório até a F4 (#676): o `localizar_formulario` sobre o `tl`, para
//! os leitores que ainda não migraram para a árvore html5ever
//! (`pesquisa_web`, `processo_criar::interessados`, `sessao_login::tela`).
use super::ErroSubmissaoFormulario;
use crate::html_atributos::valor_do_atributo;

/// Exatamente um `<form>` com o `id` ou `name` da operação.
pub(crate) fn localizar_formulario_tl<'d>(
    dom: &'d tl::VDom<'d>,
    identificador: &str,
) -> Result<&'d tl::HTMLTag<'d>, ErroSubmissaoFormulario> {
    let parser = dom.parser();
    let mut achados = Vec::new();
    for form in dom
        .query_selector("form")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
    {
        for chave in ["id", "name"] {
            if valor_do_atributo(form, chave)?.is_some_and(|v| v == identificador) {
                achados.push(form);
                break;
            }
        }
    }
    match achados.as_slice() {
        [] => Err(ErroSubmissaoFormulario::FormularioAusenteNaTela),
        [form] => Ok(form),
        _ => Err(ErroSubmissaoFormulario::FormulariosComMesmoIdentificadorNaTela),
    }
}
