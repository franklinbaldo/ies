//! Identificação estrutural de controles de paginação no Controle Web.
//! Não interpreta comandos JavaScript nem emite requisições.

/// Em visão resumida, um paginador observado impede certificar a caixa
/// pelo total da legenda da primeira página, mesmo quando coincide.
pub(crate) fn controles_de_paginacao(html: &crate::tela_html::HtmlServido) -> bool {
    let dom = crate::tela_html::arvore(html);
    for seletor in ["a", "select"] {
        for elemento in dom.elementos(seletor) {
            let id = elemento.atributo("id");
            if id.is_some_and(|nome| {
                nome.starts_with("lnkInfraProximaPagina")
                    || nome.starts_with("lnkInfraPaginaAnterior")
                    || nome.starts_with("selInfraPaginacao")
            }) {
                return true;
            }
        }
    }
    false
}

/// Paginador próprio de Recebidos ou Gerados (`lnk{T}ProximaPagina*`,
/// `sel{T}Paginacao*` etc.), observado em PEN1, PEN3, PEN4 e Goiás
/// (02/11/2026). `hdn{T}PaginaAtual` não conta: as telas `UnB` e `UFRJ` o publicam em
/// telas de uma página.
pub(crate) fn paginador_da_tabela(
    html: &crate::tela_html::HtmlServido,
    categoria: crate::dominio::Categoria,
) -> bool {
    let prefixo = match categoria {
        crate::dominio::Categoria::Recebidos => "Recebidos",
        crate::dominio::Categoria::Gerados => "Gerados",
    };
    paginador_com_prefixo(html, prefixo)
}

/// Paginador `lnk{prefixo}*`/`sel{prefixo}Paginacao*`; `Detalhado` na visão D
/// (`lnkDetalhadoProximaPagina*`, `selDetalhadoPaginacao*`: PEN1 e GO).
pub(crate) fn paginador_com_prefixo(html: &crate::tela_html::HtmlServido, prefixo: &str) -> bool {
    let dom = crate::tela_html::arvore(html);
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
        for elemento in dom.elementos(seletor) {
            let id = elemento.atributo("id");
            let resto = id.and_then(|nome| {
                nome.strip_prefix(if seletor == "a" { "lnk" } else { "sel" })?
                    .strip_prefix(prefixo)
            });
            if resto.is_some_and(|r| marcas.iter().any(|m| r.starts_with(m))) {
                return true;
            }
        }
    }
    false
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
            assert!(
                paginador_da_tabela(
                    &crate::tela_html::HtmlServido::de_teste(controle),
                    Categoria::Recebidos
                ),
                "{controle}"
            );
            assert!(
                !paginador_da_tabela(
                    &crate::tela_html::HtmlServido::de_teste(controle),
                    Categoria::Gerados
                ),
                "{controle}"
            );
        }
    }

    #[test]
    fn campo_de_pagina_atual_nao_indica_paginacao() {
        // UnB e UFRJ publicam o campo em telas de uma página.
        let html = crate::tela_html::HtmlServido::de_teste(
            "<input type='hidden' id='hdnGeradosPaginaAtual' value='0'>\
                    <div id='divGeradosAreaPaginacaoSuperior'></div>",
        );
        assert!(!paginador_da_tabela(&html, Categoria::Gerados));
    }

    #[test]
    fn booleano_antes_do_id_nao_esconde_o_paginador() {
        for html in [
            "<a hidden id='lnkInfraProximaPaginaSuperior'>»</a>",
            "<select disabled id='selInfraPaginacaoSuperior'></select>",
        ] {
            assert!(
                controles_de_paginacao(&crate::tela_html::HtmlServido::de_teste(html)),
                "{html}"
            );
        }
        assert!(!controles_de_paginacao(
            &crate::tela_html::HtmlServido::de_teste("<a hidden id='x'>»</a>")
        ));
    }
}
