//! Identificação estrutural de controles de paginação no Controle Web.
//! Não interpreta comandos JavaScript nem emite requisições.
use crate::erro_web::ErroCaixa;

/// Em visão resumida, um paginador observado impede certificar a caixa
/// pelo total da legenda da primeira página, mesmo quando coincide.
pub(crate) fn controles_de_paginacao(html: &str) -> Result<bool, ErroCaixa> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroCaixa::HtmlInvalido)?;
    let parser = dom.parser();
    for seletor in ["a", "select"] {
        if let Some(elementos) = dom.query_selector(seletor) {
            for elemento in elementos {
                let Some(tag) = elemento.get(parser).and_then(tl::Node::as_tag) else {
                    continue;
                };
                if tag.attributes().get("id").flatten().is_some_and(|valor| {
                    let nome = valor.as_utf8_str();
                    nome.starts_with("lnkInfraProximaPagina")
                        || nome.starts_with("lnkInfraPaginaAnterior")
                        || nome.starts_with("selInfraPaginacao")
                }) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
