//! RED/GREEN da mensagem de validação da Infra. Fixtures sintéticas na
//! estrutura observada (textarea oculto e script de `alert`).
use super::mensagem_de_validacao_da_infra as validacao;
use crate::tela_html::HtmlServido;
use crate::transporte::saneamento::sanear;
use crate::transporte::texto_de_recusa::texto_do_html;

/// Como `corpo_de_recusa` expõe: texto da recusa, saneado.
fn extraido(html: &str) -> Option<String> {
    texto_do_html(&HtmlServido::de_teste(html)).and_then(|t| sanear(&t))
}

/// Página sintética com a validação no formato observado.
pub(crate) fn com_validacao(texto: &str) -> String {
    format!(
        "<html><head><title>SEI</title></head><body><form id='frmX'></form>\
         <textarea id=\"txaInfraValidacao\" name=\"txaInfraValidacao\" \
         style=\"display:none\">{texto}</textarea>\
         <script type=\"text/javascript\"><!--//--><![CDATA[//><!--\n\
         self.setTimeout('alert(document.getElementById(\\'txaInfraValidacao\\').value)',300);\n\
         //--><!]]></script></body></html>"
    )
}

#[test]
fn texto_do_textarea_oculto_e_a_mensagem() {
    assert_eq!(
        validacao(&HtmlServido::de_teste(&com_validacao(
            "Usuário ou Senha Inválida."
        )))
        .as_deref(),
        Some("Usuário ou Senha Inválida.")
    );
}

#[test]
fn entidades_decodificadas_e_texto_saneado() {
    let html = HtmlServido::de_teste(&com_validacao(
        "Usu&aacute;rio &#105;nv&aacute;lido\n  em &lt;x&gt; &amp; y.",
    ));
    assert_eq!(
        validacao(&html).as_deref(),
        Some("Usuário inválido em <x> & y.")
    );
    let html = HtmlServido::de_teste(&com_validacao(
        "Ver https://sei.exemplo.invalid/a?infra_hash=abc",
    ));
    let texto = validacao(&html).unwrap();
    assert!(!texto.contains("infra_hash=abc"), "{texto}");
    let longa = validacao(&HtmlServido::de_teste(&com_validacao(&"a".repeat(2_000)))).unwrap();
    assert!(longa.chars().count() <= crate::transporte::saneamento::MAX_CARACTERES + 1);
}

#[test]
fn sem_textarea_ou_com_textarea_vazio_nao_ha_mensagem() {
    assert_eq!(
        validacao(&HtmlServido::de_teste(
            "<html><body><p>ok</p></body></html>"
        )),
        None
    );
    assert_eq!(validacao(&HtmlServido::de_teste(&com_validacao(""))), None);
    assert_eq!(
        validacao(&HtmlServido::de_teste(&com_validacao(" \n\t "))),
        None
    );
    // Só o id conta; `name` sozinho ou outro elemento com o texto, não.
    assert_eq!(
        validacao(&HtmlServido::de_teste(
            "<textarea name='txaInfraValidacao'>Erro</textarea>"
        )),
        None
    );
}

#[test]
fn validacao_vem_antes_da_excecao_no_texto_da_recusa() {
    let html = com_validacao("Senha expirada.").replace(
        "<form id='frmX'></form>",
        "<div id='divInfraExcecao'>Outra coisa</div>",
    );
    assert_eq!(extraido(&html).as_deref(), Some("Senha expirada."));
    // Textarea vazio: a exceção continua valendo.
    let html = com_validacao("").replace(
        "<form id='frmX'></form>",
        "<div id='divInfraExcecao'>Falha.</div>",
    );
    assert_eq!(extraido(&html).as_deref(), Some("Falha."));
}
