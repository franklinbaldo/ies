//! BDD da etapa anterior à prévia remota, com HTML sintético baseado no
//! Todos `tests/test_documento_juntar_010.py` e `test_documento_juntar_sei5_010.py`.
use super::*;
use crate::tela_html::HtmlServido;
fn lista(link: &str) -> String {
    format!(
        r#"<form id="frmDocumentoEscolherTipo" method="post" action="controlador.php?acao=documento_escolher_tipo">
    <input type="hidden" id="hdnIdSerie" name="hdnIdSerie" value="" />
    </form><table id="tblSeries"><tr><td><a href="controlador.php?acao=documento_gerar">Despacho</a></td></tr></table>
    {link}"#
    )
}
fn cadastro() -> String {
    r#"<form id="frmDocumentoCadastro" method="post" action="controlador.php?acao=documento_receber">
       <select id="selSerie" name="selSerie">
       <option value="null"> </option><option value="293">Anexo</option>
       <option value="291">Despacho</option>
       </select><input type="hidden" name="hdnIdSerie" id="hdnIdSerie" value="" />
       <input type="hidden" name="hdnFlagDocumentoCadastro" id="hdnFlagDocumentoCadastro" value="1" />
       </form>"#.to_owned()
}
#[test]
fn dado_sei4_quando_escolho_externo_entao_obtenho_link_publicado() {
    let html = HtmlServido::de_teste(&lista(
        r#"<a href="controlador.php?acao=documento_receber&id_procedimento=50961&infra_hash=FICTICIO">Externo</a>"#,
    ));
    assert_eq!(
        escolher_externo(&html),
        Ok(EscolhaExterna::Url(
            "controlador.php?acao=documento_receber&id_procedimento=50961&infra_hash=FICTICIO"
                .to_owned()
        ))
    );
}
#[test]
fn dado_sei5_quando_escolho_externo_entao_obtenho_formulario_sem_executar_script() {
    let html = HtmlServido::de_teste(&lista(
        r##"<a href="#" onclick="escolher(-1);">Externo</a>"##,
    ));
    assert_eq!(escolher_externo(&html), Ok(EscolhaExterna::Formulario));
}
#[test]
fn dado_cadastro_antes_da_recarga_quando_peço_anexo_entao_obtenho_id_observado() {
    assert_eq!(
        identificar_tipo(&HtmlServido::de_teste(&cadastro()), "Anexo"),
        Ok("293".to_owned())
    );
    assert_eq!(
        identificar_tipo(&HtmlServido::de_teste(&cadastro()), "Parecer"),
        Err(ErroEscolhaExterna::NaoOferecido)
    );
}
#[test]
fn dado_dois_tipos_iguais_quando_peço_anexo_entao_recuso_ambiguidade() {
    let html = HtmlServido::de_teste(&cadastro().replace(
        "<option value=\"291\">Despacho</option>",
        "<option value=\"291\">Anexo</option>",
    ));
    assert_eq!(
        identificar_tipo(&html, "Anexo"),
        Err(ErroEscolhaExterna::Ambiguo)
    );
}

#[test]
fn dado_sinalizador_de_cadastro_quando_ja_grava_entao_nao_recarrega() {
    let tela = HtmlServido::de_teste(&cadastro().replace("value=\"1\" />", "value=\"2\" />"));
    assert_eq!(
        identificar_tipo(&tela, "Anexo"),
        Err(ErroEscolhaExterna::ForaDaForma)
    );
}

#[test]
fn dado_acao_publicada_quando_rotulo_muda_entao_nao_classifica_por_texto() {
    // Dado link SEI 4 com rótulo de apresentação divergente;
    // Quando escolho documento externo; Então a ação prevalece sobre o texto.
    let link = "controlador.php?acao=documento_receber&id_procedimento=50961&infra_hash=FICTICIO";
    let html = HtmlServido::de_teste(&lista(&format!(
        r#"<a href="{link}">Documento recebido</a>"#
    )));
    assert_eq!(
        escolher_externo(&html),
        Ok(EscolhaExterna::Url(link.to_owned()))
    );
}

#[test]
fn dado_template_inerte_quando_publica_acao_entao_nao_aceita_link_falso() {
    // Dado link que só existe em template; Quando escolho; Então não navego.
    let html = HtmlServido::de_teste(&lista(
        r#"<template><a href="controlador.php?acao=documento_receber&id_procedimento=50961&infra_hash=FICTICIO">Externo</a></template>"#,
    ));
    assert_eq!(
        escolher_externo(&html),
        Err(ErroEscolhaExterna::NaoOferecido)
    );
}

#[test]
fn dado_opcao_vazia_com_nbsp_da_unb_quando_peço_anexo_entao_obtenho_id_observado() {
    // UnB SEI 4.0.9 (cassettes documento-mover-cancelar-unb e
    // cadastro-listas-unb): a opção vazia é `&nbsp;`. Ensaio live de
    // 09/11/2026 recusava a tela inteira (tipo_externo_nao_oferecido).
    let html = cadastro().replace(
        r#"<option value="null"> </option>"#,
        r#"<option value="null" selected="selected">&nbsp;</option>"#,
    );
    assert_eq!(
        identificar_tipo(&HtmlServido::de_teste(&html), "Anexo"),
        Ok("293".to_owned())
    );
    let acentuado = HtmlServido::de_teste(&html.replace(">Despacho<", ">Of&iacute;cio<"));
    assert_eq!(identificar_tipo(&acentuado, "Ofício"), Ok("291".to_owned()));
}
