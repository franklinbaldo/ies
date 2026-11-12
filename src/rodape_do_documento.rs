//! O rodapé que o `documento_visualizar` serve abaixo do conteúdo (Refs
//! #88; contracts/cli/documento-obter-versoes.md): um
//! `<div unselectable="on">` com `Criado por <a>U</a>, versão N por <a>U</a>
//! em DATA.`, gerado pelo SEI (não é texto editável). Visto em todas as 96
//! respostas `documento_visualizar` do acervo (`UnB`, PEN1 a PEN4) e na
//! PEN4 5.0.4 ao vivo, inclusive na versão N lida pela lista (`versão N`).
//! Só a forma exata conta: nenhum ou mais de um rodapé é "não servido".
use crate::documento_versoes::VersaoDocumento;
use crate::html_atributos::atributos;

/// A versão que o rodapé declara para o conteúdo servido; `None` se não há
/// exatamente um rodapé na forma observada.
pub(crate) fn versao_do_rodape(html: &crate::tela_html::HtmlServido) -> Option<VersaoDocumento> {
    let dom = crate::tela_html::interpretar(html);
    let parser = dom.parser();
    let mut achadas = Vec::new();
    for no in dom.query_selector("div")?.filter_map(|h| h.get(parser)) {
        let Some(tag) = no.as_tag() else {
            continue;
        };
        let lidos = atributos(tag).ok()?;
        if lidos.valor("unselectable") != Some("on") {
            continue;
        }
        let texto = html_escape::decode_html_entities(&no.inner_text(parser)).into_owned();
        let texto = texto.split_whitespace().collect::<Vec<_>>().join(" ");
        if let Some(versao) = versao_da_linha(&texto) {
            achadas.push(versao);
        }
    }
    match achadas.as_slice() {
        [uma] => Some(*uma),
        _ => None,
    }
}

/// Código CRC mostrado no bloco de autenticidade de documento assinado.
/// Informativo: não é recalculado nem participa da identidade do cache.
pub(crate) fn crc_do_rodape(html: &crate::tela_html::HtmlServido) -> Option<String> {
    let dom = crate::tela_html::interpretar(html);
    let parser = dom.parser();
    let mut achados = Vec::new();
    for no in dom.query_selector("p, div")?.filter_map(|h| h.get(parser)) {
        let texto = html_escape::decode_html_entities(&no.inner_text(parser)).into_owned();
        let texto = texto.split_whitespace().collect::<Vec<_>>().join(" ");
        let minusculo = texto.to_lowercase();
        if !minusculo.contains("autenticidade") || !minusculo.contains("código verificador") {
            continue;
        }
        let Some(inicio) = minusculo.find("código crc") else {
            continue;
        };
        let depois = texto.get(inicio + "código crc".len()..)?.trim_start();
        let depois = depois.trim_start_matches([':', '-', ' ', '\t']);
        let codigo: String = depois.chars().take_while(char::is_ascii_hexdigit).collect();
        if (6..=16).contains(&codigo.len()) {
            achados.push(codigo.to_ascii_uppercase());
        }
    }
    achados.sort();
    achados.dedup();
    match achados.as_slice() {
        [unico] => Some(unico.clone()),
        _ => None,
    }
}

/// `Criado por X, versão N por Y em DATA.` → N.
fn versao_da_linha(texto: &str) -> Option<VersaoDocumento> {
    let resto = texto.strip_prefix("Criado por ")?;
    let mut partes = resto.split(", versão ");
    let (_, depois) = (partes.next()?, partes.next()?);
    if partes.next().is_some() {
        return None;
    }
    let (numero, resto) = depois.split_once(" por ")?;
    let (_, data) = resto.rsplit_once(" em ")?;
    if data.is_empty() || !data.ends_with('.') {
        return None;
    }
    VersaoDocumento::interpretar(numero).ok()
}

#[cfg(test)]
pub(crate) mod testes {
    use super::*;

    /// O rodapé na forma servida pela PEN4 5.0.4 (04/11/2026).
    pub(crate) fn rodape(versao: &str) -> String {
        format!(
            "<div unselectable=\"on\" style=\"user-select:none;\">\n<hr style=\"border:1px solid #c0c0c0;\" />Criado por <a onclick=\"alert('Usuário Teste')\" title=\"Usuário Teste\">USUARIO</a>, versão {versao} por <a onclick=\"alert('Usuário Teste')\" title=\"Usuário Teste\">USUARIO</a> em 20/10/2026 14:07:04.\n</div>"
        )
    }

    fn pagina(corpo: &str) -> String {
        format!(
            "<html><head><title>SEI/X - 0007 - Despacho</title></head><body><p>Texto, versão 9 por alguém.</p>{corpo}</body></html>"
        )
    }

    #[test]
    fn crc_assinado_so_do_bloco_de_autenticidade() {
        let assinado = crate::tela_html::HtmlServido::de_teste(
            "<html><body><p>A autenticidade deste documento pode ser conferida informando o código verificador 123456 e o código CRC 1a2B3c4D.</p></body></html>",
        );
        assert_eq!(crc_do_rodape(&assinado).as_deref(), Some("1A2B3C4D"));
        assert_eq!(
            crc_do_rodape(&crate::tela_html::HtmlServido::de_teste(
                "<html><body><p>código CRC 1a2B3c4D</p></body></html>"
            )),
            None
        );
    }

    #[test]
    fn rodape_servido_declara_a_versao() {
        assert_eq!(
            versao_do_rodape(&crate::tela_html::HtmlServido::de_teste(&pagina(&rodape(
                "2"
            ))))
            .map(VersaoDocumento::numero),
            Some(2)
        );
        let com_entidade = rodape("3").replace("versão", "vers&atilde;o");
        assert_eq!(
            versao_do_rodape(&crate::tela_html::HtmlServido::de_teste(&pagina(
                &com_entidade
            )))
            .map(VersaoDocumento::numero),
            Some(3)
        );
    }

    #[test]
    fn sem_rodape_repetido_ou_fora_da_forma_nao_ha_versao() {
        assert_eq!(
            versao_do_rodape(&crate::tela_html::HtmlServido::de_teste(&pagina(""))),
            None
        );
        let dois = format!("{}{}", rodape("2"), rodape("3"));
        assert_eq!(
            versao_do_rodape(&crate::tela_html::HtmlServido::de_teste(&pagina(&dois))),
            None
        );
        for fora in ["0", "2a", ""] {
            assert_eq!(
                versao_do_rodape(&crate::tela_html::HtmlServido::de_teste(&pagina(&rodape(
                    fora
                )))),
                None,
                "{fora}"
            );
        }
        let selecionavel = rodape("2").replace("unselectable=\"on\"", "class=\"x\"");
        assert_eq!(
            versao_do_rodape(&crate::tela_html::HtmlServido::de_teste(&pagina(
                &selecionavel
            ))),
            None
        );
        let sem_data = rodape("2").replace(" em 20/10/2026 14:07:04.", ".");
        assert_eq!(
            versao_do_rodape(&crate::tela_html::HtmlServido::de_teste(&pagina(&sem_data))),
            None
        );
    }
}
