//! Leitura do formulário e das seções do editor clássico.
use crate::documento_versoes::VersaoDocumento;
use crate::tela_html::{HtmlServido, arvore};

pub(super) fn versao_do_editor(html: &HtmlServido) -> Option<VersaoDocumento> {
    let dom = arvore(html);
    let form = crate::formulario_web::localizar_formulario(&dom, "frmEditor").ok()?;
    let oculto = |nome| {
        let controles: Vec<_> = form
            .elementos("input")
            .filter(|e| {
                e.atributo("name") == Some(nome)
                    && e.atributo("type") == Some("hidden")
                    && !e.presente("disabled")
            })
            .collect();
        let [controle] = controles.as_slice() else {
            return None;
        };
        controle.atributo("value")
    };
    if oculto("hdnIgnorarNovaVersao")? != "N" {
        return None;
    }
    VersaoDocumento::interpretar(oculto("hdnVersao")?).ok()
}

/// Lê o texto cru escapado pelo SEI, pois o DOM normaliza CR e CRLF do textarea.
pub(super) fn secao(html: &HtmlServido, nome: &str) -> Option<String> {
    let aspas_duplas = format!("name=\"{nome}\"");
    let aspas_simples = format!("name='{nome}'");
    let mut secoes = html.split("<textarea").skip(1).filter_map(|trecho| {
        let (atributos, resto) = trecho.split_once('>')?;
        if !atributos.contains(&aspas_duplas) && !atributos.contains(&aspas_simples) {
            return None;
        }
        resto
            .split_once("</textarea>")
            .map(|(texto, _)| html_escape::decode_html_entities(texto).into_owned())
    });
    let unica = secoes.next()?;
    secoes.next().is_none().then_some(unica)
}

pub(super) fn principal_publicado(html: &HtmlServido) -> Option<String> {
    let dom = arvore(html);
    let mut gravaveis = Vec::new();
    for script in dom.elementos("script").filter(|s| !s.presente("src")) {
        let texto = script.texto();
        for chamada in texto.split("CKEDITOR.replace(").skip(1) {
            let (nome, config) = chamada.split_once(',')?;
            let nome = nome.trim().trim_matches(['\'', '"']);
            let (_, readonly) = config.split_once("\"readOnly\":")?;
            if readonly.trim_start().starts_with("false") {
                gravaveis.push(nome.to_owned());
            }
        }
    }
    let [nome] = gravaveis.as_slice() else {
        return None;
    };
    Some(nome.clone())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn rele_o_textarea_preservando_caracteres_de_controle() {
        let html = HtmlServido::de_teste(
            "<textarea name='txaEditor_370'>&lt;p&gt;linha\r\r\nmais\r</textarea>",
        );
        let lida = secao(&html, "txaEditor_370").unwrap();
        assert_eq!(lida, "<p>linha\r\r\nmais\r");
        // O formulário clássico recebe a seção opaca, com LF canônico.
        let enviada = crate::documento_salvar_cli::html_para_envio(&lida);
        assert_eq!(enviada, "<p>linha\nmais\n");
        assert_eq!(lida, "<p>linha\r\r\nmais\r");
    }
}
