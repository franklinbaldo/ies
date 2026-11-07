//! RED sintético #246: script executável adicional invalida prova de Salvar.
//! Cenários adversariais, não observação de uma instalação do SEI.
use super::*;

#[test]
fn script_ativo_adicional_pode_invalidar_cadeia_antes_do_post() {
    let original = cadastro();
    for (variante, script) in [
        (
            "application/javascript redefine submeter",
            r#"<script type="application/javascript">function submeter() {}</script>"#,
        ),
        (
            "application/javascript redefine confirmarDados",
            r#"<script type="application/javascript">function confirmarDados() {}</script>"#,
        ),
        (
            "externo com codigo nao comprovado",
            r#"<script src="/static/submissao.js"></script>"#,
        ),
        (
            "module pode alterar globais",
            r#"<script type="module">window.submeter = () => {};</script>"#,
        ),
    ] {
        let html = original.replace("</script>", &format!("</script>{script}"));
        let (resultado, pedidos) = executar_com_cadastro_sintetico(html);
        assert_eq!(
            resultado.err().map(|erro| erro.subcodigo()),
            Some("submissao_do_documento_nao_observada"),
            "{variante}: cadeia nao comprovada"
        );
        assert_eq!(
            pedidos,
            pedidos_antes_da_escrita(),
            "{variante}: 5 GET de previa, 0 POST de criacao"
        );
    }
}

#[test]
fn script_ativo_application_javascript_e_json_inerte_preservam_prova() {
    let original = cadastro();
    let ativo = original.replace("<script>", "<script type=\"application/javascript\">");
    assert!(
        cadastro_publica_script_de_criacao(&ativo),
        "MIME JavaScript executavel deve ser examinado, nao descartado"
    );
    let json = original.replace(
        "</script>",
        "</script><script type=\"application/json\">{\"texto\":\"function submeter() {}\"}</script>",
    );
    assert!(
        cadastro_publica_script_de_criacao(&json),
        "JSON nao executavel nao redefine os callbacks"
    );
}
