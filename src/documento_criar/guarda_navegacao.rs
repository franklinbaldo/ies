//! Ações de filtro e seleção de tipo: prova de script inline ativo (#246).
//! Reusa o leitor lexical de script da guarda de criação; nunca executa JS.
use super::guarda_script::{dentro_de_template, faixas_inertes, fonte_do_script};

/// Comprova uma função de navegação apenas em script inline clássico ATIVO.
/// A antiga busca em todo o HTML aceitava textos de `template`, JSON
/// e comentários HTML como se fossem código executável da página.
/// Mantém a exigência literal de corpo e unicidade sem executar JavaScript.
pub(super) fn script_publica_funcao_de_navegacao(
    html: &str,
    assinatura: &str,
    corpo_esperado: &str,
) -> bool {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return false;
    };
    let Some(faixas) = faixas_inertes(&dom) else {
        return false;
    };
    let mut encontradas = 0_usize;
    for no in dom.query_selector("script").into_iter().flatten() {
        let Some(script) = no.get(dom.parser()).and_then(tl::Node::as_tag) else {
            return false;
        };
        if dentro_de_template(script.boundaries(dom.parser()).0, &faixas) {
            continue;
        }
        let Ok(tipo) = crate::html_atributos::valor_do_atributo(script, "type") else {
            return false;
        };
        if tipo
            .as_deref()
            .is_some_and(|tipo| !tipo.trim().eq_ignore_ascii_case("text/javascript"))
        {
            continue;
        }
        match crate::html_atributos::atributo_presente(script, "src") {
            Ok(true) => continue,
            Ok(false) => {}
            Err(_) => return false,
        }
        let Some(texto) = fonte_do_script(html, script.boundaries(dom.parser()).0) else {
            return false;
        };
        let compacto: String = texto.chars().filter(|c| !c.is_ascii_whitespace()).collect();
        let ocorrencias = compacto.matches(assinatura).count();
        if ocorrencias == 0 {
            continue;
        }
        if ocorrencias != 1 || !compacto.contains(corpo_esperado) {
            return false;
        }
        encontradas += 1;
        if encontradas > 1 {
            return false;
        }
    }
    encontradas == 1
}

