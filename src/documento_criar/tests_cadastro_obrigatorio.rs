//! Controles obrigatórios e desabilitados do cadastro de documento interno:
//! o POST só sai com o que a tela exige atendido. Parte de `tests`.
use super::*;
use crate::tela_html::HtmlServido;

#[test]
fn cadastro_recusa_controle_obrigatorio_nao_suportado() {
    // HTML5 required: não enviar um POST ignorando um campo que a página exige.
    let html = HtmlServido::de_teste(
        r#"<form id="frmDocumentoCadastro" method="post">
        <label id="lblSerieTitulo">Despacho</label>
        <input type="radio" id="optPublico" name="rdoNivelAcesso" value="0">
        <input type="text" name="txtPrazoNovo" required>
        </form>"#,
    );
    assert_eq!(
        conferir_cadastro(
            &html,
            "Despacho",
            None,
            Some(crate::dominio::NivelDeAcesso::Publico)
        ),
        Err(super::cadastro::ErroCadastroDoDocumento::CampoObrigatorioNaoAtendido)
    );
}

#[test]
fn cadastro_recusa_descricao_obrigatoria_nao_preenchida() {
    let html = HtmlServido::de_teste(
        r#"<form id="frmDocumentoCadastro" method="post">
        <label id="lblSerieTitulo">Despacho</label>
        <input type="radio" id="optPublico" name="rdoNivelAcesso" value="0">
        <input type="text" id="txtDescricao" name="txtDescricao" required>
        </form>"#,
    );
    assert_eq!(
        conferir_cadastro(
            &html,
            "Despacho",
            None,
            Some(crate::dominio::NivelDeAcesso::Publico)
        ),
        Err(super::cadastro::ErroCadastroDoDocumento::CampoObrigatorioNaoAtendido)
    );
}

#[test]
fn cadastro_recusa_selecao_obrigatoria_nao_suportada() {
    let html = HtmlServido::de_teste(
        r#"<form id="frmDocumentoCadastro" method="post">
        <label id="lblSerieTitulo">Despacho</label>
        <input type="radio" id="optPublico" name="rdoNivelAcesso" value="0">
        <select name="selNovaExigencia" required><option value="">Selecione</option></select>
        </form>"#,
    );
    assert_eq!(
        conferir_cadastro(
            &html,
            "Despacho",
            None,
            Some(crate::dominio::NivelDeAcesso::Publico)
        ),
        Err(super::cadastro::ErroCadastroDoDocumento::CampoObrigatorioNaoAtendido)
    );
}

#[test]
fn cadastro_aceita_controles_obrigatorios_atendidos_e_ignora_desabilitado() {
    let html = HtmlServido::de_teste(
        r#"<form id="frmDocumentoCadastro" method="post">
        <label id="lblSerieTitulo">Despacho</label>
        <input type="radio" id="optPublico" name="rdoNivelAcesso" value="0" required>
        <input type="text" id="txtDescricao" name="txtDescricao" required>
        <input type="text" name="txtCampoNaoUsado" required disabled>
        <input type="hidden" name="hdnIdSerie" value="291" required>
        </form>"#,
    );
    assert_eq!(
        conferir_cadastro(
            &html,
            "Despacho",
            Some("Ensaio"),
            Some(crate::dominio::NivelDeAcesso::Publico)
        ),
        Ok(())
    );
}

#[test]
fn cadastro_recusa_nivel_com_disabled_booleano() {
    // `disabled` sem valor é booleano HTML; valor_do_atributo não o representa.
    let html = HtmlServido::de_teste(
        r#"<form id="frmDocumentoCadastro" method="post">
        <label id="lblSerieTitulo">Despacho</label>
        <input type="radio" id="optPublico" name="rdoNivelAcesso" value="0" disabled>
        </form>"#,
    );
    assert_eq!(
        conferir_cadastro(
            &html,
            "Despacho",
            None,
            Some(crate::dominio::NivelDeAcesso::Publico)
        ),
        Err(super::cadastro::ErroCadastroDoDocumento::NivelNaoOferecido)
    );
}

#[test]
fn cadastro_recusa_descricao_com_disabled_booleano() {
    let html = HtmlServido::de_teste(
        r#"<form id="frmDocumentoCadastro" method="post">
        <label id="lblSerieTitulo">Despacho</label>
        <input type="radio" id="optPublico" name="rdoNivelAcesso" value="0">
        <input type="text" id="txtDescricao" name="txtDescricao" disabled>
        </form>"#,
    );
    assert_eq!(
        conferir_cadastro(
            &html,
            "Despacho",
            Some("texto"),
            Some(crate::dominio::NivelDeAcesso::Publico)
        ),
        Err(super::cadastro::ErroCadastroDoDocumento::DescricaoNaoOferecida)
    );
}
