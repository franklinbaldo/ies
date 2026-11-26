//! Diagnóstico estrutural do paginador original, sem valores ou URLs.
use super::{HtmlServido, arvore};

fn identificador(valor: &str) -> bool {
    !valor.is_empty() && valor.len() <= 80 && valor.bytes().all(|b| b.is_ascii_alphanumeric())
}

pub(super) fn observar_formularios(instalacao: &str, html: &HtmlServido) {
    let dom = arvore(html);
    let mut candidatos = 0;
    for form in dom.elementos("form") {
        let id = form
            .atributo("id")
            .filter(|id| id.starts_with("frm") && identificador(id))
            .unwrap_or("(outro)");
        let post = form
            .atributo("method")
            .is_some_and(|m| m.eq_ignore_ascii_case("post"));
        for campo in form.elementos("input") {
            if campo.atributo("type") != Some("hidden") {
                continue;
            }
            let Some(nome) = campo.atributo("name").filter(|n| {
                n.starts_with("hdn") && n.ends_with("PaginaAtual") && identificador(n)
            }) else {
                continue;
            };
            candidatos += 1;
            let zero = campo.atributo("value") == Some("0");
            println!("{instalacao}: form={id} post={post} campo={nome} indice_zero={zero}");
        }
    }
    println!("{instalacao}: campos candidatos de paginacao={candidatos}");
}
