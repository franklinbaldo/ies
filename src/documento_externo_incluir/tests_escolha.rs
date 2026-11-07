//! BDD da etapa anterior à prévia remota, com HTML sintético baseado no
//! Todos tests/test_documento_juntar_010.py e test_documento_juntar_sei5_010.py.
use super::*;
fn lista(link: &str) -> String {
    format!(r#"<form id="frmDocumentoEscolherTipo" method="post" action="controlador.php?acao=documento_escolher_tipo">
    <input type="hidden" id="hdnIdSerie" name="hdnIdSerie" value="" />
    </form><table id="tblSeries"><tr><td><a href="controlador.php?acao=documento_gerar">Despacho</a></td></tr></table>
    {link}"#)
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
    let html = lista(r#"<a href="controlador.php?acao=documento_receber&id_procedimento=50961&infra_hash=FICTICIO">Externo</a>"#);
    assert_eq!(escolher_externo(&html),
       Ok(EscolhaExterna::Url("controlador.php?acao=documento_receber&id_procedimento=50961&infra_hash=FICTICIO".to_owned())));
}
#[test]
fn dado_sei5_quando_escolho_externo_entao_obtenho_formulario_sem_executar_script() {
    let html = lista(r##"<a href="#" onclick="escolher(-1);">Externo</a>"##);
    assert_eq!(escolher_externo(&html), Ok(EscolhaExterna::Formulario));
}
#[test]
fn dado_cadastro_antes_da_recarga_quando_peço_anexo_entao_obtenho_id_observado() {
    assert_eq!(identificar_tipo(&cadastro(), "Anexo"), Ok("293".to_owned()));
    assert_eq!(identificar_tipo(&cadastro(), "Parecer"), Err(ErroEscolhaExterna::NaoOferecido));
}
#[test]
fn dado_dois_tipos_iguais_quando_peço_anexo_entao_recuso_ambiguidade() {
    let html = cadastro().replace("<option value=\"291\">Despacho</option>", "<option value=\"291\">Anexo</option>");
    assert_eq!(identificar_tipo(&html, "Anexo"), Err(ErroEscolhaExterna::Ambiguo));
}

#[test]
fn dado_sinalizador_de_cadastro_quando_ja_grava_entao_nao_recarrega() {
    let tela = cadastro().replace("value=\"1\" />", "value=\"2\" />");
    assert_eq!(identificar_tipo(&tela, "Anexo"), Err(ErroEscolhaExterna::ForaDaForma));
}
