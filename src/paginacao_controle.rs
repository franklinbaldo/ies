//! Identificação estrutural de controles de paginação no Controle Web.
//! Não interpreta comandos JavaScript nem emite requisições.
use crate::erro_web::ErroCaixa;
use crate::html_atributos::valor_do_atributo;

/// Em visão resumida, um paginador observado impede certificar a caixa
/// pelo total da legenda da primeira página, mesmo quando coincide.
pub(crate) fn controles_de_paginacao(html: &str) -> Result<bool, ErroCaixa> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroCaixa::HtmlDaCaixaInvalido)?;
    let parser = dom.parser();
    for seletor in ["a", "select"] {
        if let Some(elementos) = dom.query_selector(seletor) {
            for elemento in elementos {
                let Some(tag) = elemento.get(parser).and_then(tl::Node::as_tag) else {
                    continue;
                };
                let id =
                    valor_do_atributo(tag, "id").map_err(ErroCaixa::AtributoDaCaixaIlegivel)?;
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
/// `sel{T}Paginacao*` etc.), observado em PEN1, PEN3, PEN4 e Goiás
/// (02/11/2026). `hdn{T}PaginaAtual` não conta: as telas `UnB` e `UFRJ` o publicam em
/// telas de uma página.
pub(crate) fn paginador_da_tabela(
    html: &str,
    categoria: crate::dominio::Categoria,
) -> Result<bool, ErroCaixa> {
    let prefixo = match categoria {
        crate::dominio::Categoria::Recebidos => "Recebidos",
        crate::dominio::Categoria::Gerados => "Gerados",
    };
    paginador_com_prefixo(html, prefixo)
}

/// Paginador `lnk{prefixo}*`/`sel{prefixo}Paginacao*`; `Detalhado` na visão D
/// (`lnkDetalhadoProximaPagina*`, `selDetalhadoPaginacao*`: PEN1 e GO).
pub(crate) fn paginador_com_prefixo(html: &str, prefixo: &str) -> Result<bool, ErroCaixa> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroCaixa::HtmlDaCaixaInvalido)?;
    let parser = dom.parser();
    for (seletor, marcas) in [
        (
            "a",
            &[
                "ProximaPagina",
                "PaginaAnterior",
                "PrimeiraPagina",
                "UltimaPagina",
            ][..],
        ),
        ("select", &["Paginacao"][..]),
    ] {
        for elemento in dom.query_selector(seletor).into_iter().flatten() {
            let Some(tag) = elemento.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            let id = valor_do_atributo(tag, "id").map_err(ErroCaixa::AtributoDaCaixaIlegivel)?;
            let resto = id.as_deref().and_then(|nome| {
                nome.strip_prefix(if seletor == "a" { "lnk" } else { "sel" })?
                    .strip_prefix(prefixo)
            });
            if resto.is_some_and(|r| marcas.iter().any(|m| r.starts_with(m))) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::{controles_de_paginacao, paginador_da_tabela};
    use crate::dominio::Categoria;

    #[test]
    fn paginador_da_tabela_e_reconhecido_pelo_prefixo_da_propria_tabela() {
        for controle in [
            "<a id='lnkRecebidosProximaPaginaSuperior'>»</a>",
            "<a hidden id='lnkRecebidosUltimaPaginaInferior'>»|</a>",
            "<a id='lnkRecebidosPaginaAnteriorSuperior'>«</a>",
            "<a id='lnkRecebidosPrimeiraPaginaSuperior'>|«</a>",
            "<select id='selRecebidosPaginacaoInferior'><option>1</option></select>",
        ] {
            assert_eq!(
                paginador_da_tabela(controle, Categoria::Recebidos),
                Ok(true),
                "{controle}"
            );
            assert_eq!(
                paginador_da_tabela(controle, Categoria::Gerados),
                Ok(false),
                "{controle}"
            );
        }
    }

    #[test]
    fn campo_de_pagina_atual_nao_indica_paginacao() {
        // UnB e UFRJ publicam o campo em telas de uma página.
        let html = "<input type='hidden' id='hdnGeradosPaginaAtual' value='0'>\
                    <div id='divGeradosAreaPaginacaoSuperior'></div>";
        assert_eq!(paginador_da_tabela(html, Categoria::Gerados), Ok(false));
    }

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
