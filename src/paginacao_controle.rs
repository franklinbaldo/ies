//! Identificação estrutural de controles de paginação no Controle Web.
//! Não interpreta comandos JavaScript nem emite requisições.
use crate::erro_web::ErroCaixa;
use crate::html_atributos::valor_do_atributo;

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
                let id = valor_do_atributo(tag, "id").map_err(|_| ErroCaixa::HtmlInvalido)?;
                if id.is_some_and(|nome| {
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

/// Paginador próprio de Recebidos ou Gerados (`lnk{T}ProximaPagina*`,
/// `sel{T}Paginacao*` etc.). `hdn{T}PaginaAtual` não conta: aparece em telas
/// sem paginação. Shim CONTRACT.
pub(crate) fn paginador_da_tabela(
    _html: &str,
    _categoria: crate::dominio::Categoria,
) -> Result<bool, ErroCaixa> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::controles_de_paginacao;

    #[test]
    fn booleano_antes_do_id_nao_esconde_o_paginador() {
        for html in [
            "<a hidden id='lnkInfraProximaPaginaSuperior'>»</a>",
            "<select disabled id='selInfraPaginacaoSuperior'></select>",
        ] {
            assert_eq!(controles_de_paginacao(html), Ok(true), "{html}");
        }
        assert_eq!(controles_de_paginacao("<a hidden id='x'>»</a>"), Ok(false));
    }
}
