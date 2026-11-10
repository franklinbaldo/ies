//! Casos de Todos src/todos/anexacao.py @323eb01 (formulário,
//! complemento `protocolo_RI1023`, verificação e lista de anexados).
use super::*;

fn html() -> String {
    let hash = "a".repeat(64);
    format!(
        r#"<html><form id="frmProcedimentoAnexar" method="post" action="controlador.php?acao=procedimento_anexar&id_procedimento=11&infra_hash={hash}">
<input name="txtProtocolo" type="text" value="">
<input name="txtIdentificacaoProtocolo" type="text" value="">
<input name="hdnIdProtocolo" type="hidden" value="">
<input name="hdnAnexacao" type="hidden" value="">
<input name="sbmAnexar" type="submit" value="Anexar">
<table summary="Lista de Processos Anexados">
<tr><th>Processo</th></tr><tr><td><input type="checkbox" value="22"></td></tr>
</table></form></html>"#
    )
}
fn complemento() -> &'static str {
    r#"<complementos>
<complemento nome="IdProcedimento">42</complemento>
<complemento nome="ProtocoloProcedimentoFormatado">12345.000002/2026-01</complemento>
<complemento nome="NomeTipoProcedimento">Administrativo</complemento>
</complementos>"#
}
fn destino() -> Destino {
    Destino {
        id: "42".into(),
        nup: "12345.000002/2026-01".into(),
        tipo: "Administrativo".into(),
    }
}
#[test]
fn controle_e_lista_da_tela_sao_reconhecidos() {
    assert_eq!(ler_tela(&html()).unwrap().anexados, vec!["22"]);
}
#[test]
fn campos_duplicados_nao_podem_ser_escolhidos_arbitrariamente() {
    let repetida = html().replace(
        "</form>",
        "<input name=\"hdnIdProtocolo\" type=\"hidden\"></form>",
    );
    assert_eq!(
        ler_tela(&repetida),
        Err("controles_da_anexacao_ausentes_ou_repetidos")
    );
}
#[test]
fn lista_duplicada_ou_anexado_duplicado_nao_e_aceito() {
    let tabela_repetida = html().replace(
        "</form>",
        r#"<table summary="Lista de Processos Anexados"></table></form>"#,
    );
    assert_eq!(
        ler_tela(&tabela_repetida),
        Err("listas_de_anexados_repetidas")
    );
    let id_repetido = html().replace(
        "</table>",
        r#"<tr><td><input type="checkbox" value="22"></td></tr></table>"#,
    );
    assert_eq!(ler_tela(&id_repetido), Err("ids_de_anexados_repetidos"));
}
#[test]
fn complemento_identifica_numero_id_e_tipo_sem_adivinhar() {
    assert_eq!(ler_destino(complemento()).unwrap(), destino());
}
#[test]
fn complemento_ambiguo_nao_e_aceito() {
    let alterado = complemento().replace(
        "</complementos>",
        "<complemento nome=\"IdProcedimento\">43</complemento></complementos>",
    );
    assert_eq!(ler_destino(&alterado), Err("identidade_de_anexado_ambigua"));
    assert_eq!(
        ler_destino("<complementos/>"),
        Err("processo_a_anexar_nao_identificado")
    );
}
#[test]
fn verificacao_da_anexacao_exige_s_ou_n_unico() {
    assert_eq!(
        ler_confirmacao(
            r#"<complementos><complemento nome="Anexacao">S</complemento></complementos>"#
        ),
        Ok(true)
    );
    assert_eq!(
        ler_confirmacao(
            r#"<complementos><complemento nome="Anexacao">N</complemento></complementos>"#
        ),
        Ok(false)
    );
    assert_eq!(
        ler_confirmacao("<complementos/>"),
        Err("anexacao_nao_verificada")
    );
    assert_eq!(
        ler_confirmacao(
            r#"<complementos><complemento nome="Anexacao">S</complemento><complemento nome="Anexacao">N</complemento></complementos>"#
        ),
        Err("anexacao_nao_verificada")
    );
}
#[test]
fn recusa_o_proprio_principal_antes_da_escrita() {
    assert_eq!(
        decidir("12345.000002/2026-01", "11", &destino(), &[]),
        Err("anexacao_ao_mesmo_processo")
    );
    assert_eq!(
        decidir("12345.000001/2026-01", "42", &destino(), &[]),
        Err("anexacao_ao_mesmo_processo")
    );
}
#[test]
fn existente_e_recusado_e_novo_e_aceito() {
    assert_eq!(
        decidir("12345.000001/2026-01", "11", &destino(), &["42".into()]),
        Err("processo_ja_anexado")
    );
    assert_eq!(
        decidir("12345.000001/2026-01", "11", &destino(), &[]),
        Ok(true)
    );
}

#[test]
fn tabela_de_anexados_pode_ser_irma_do_formulario() {
    // Forma do oráculo Todos: a lista é procurada na tela, não apenas no form.
    let alterado = html()
        .replace(
            "<table summary=\"Lista de Processos Anexados\">",
            "</form><table summary=\"Lista de Processos Anexados\">",
        )
        .replace("</table></form>", "</table>");
    assert_eq!(ler_tela(&alterado).unwrap().anexados, vec!["22"]);
}

#[test]
fn consulta_ajax_exige_literal_unico_e_endereco_relativo() {
    use super::fluxo::url_da_consulta;
    let script = "new infraAjaxComplementar('txtProtocolo','controlador_ajax.php?acao_ajax=protocolo_RI1023&infra_hash=abc');";
    let prefixo = "new infraAjaxComplementar('txtProtocolo','";
    assert!(
        url_da_consulta(script, prefixo)
            .unwrap()
            .starts_with("controlador_ajax.php?")
    );
    assert_eq!(
        url_da_consulta(&format!("{script}{script}"), prefixo),
        Err("consultas_de_anexacao_repetidas")
    );
    assert_eq!(
        url_da_consulta("", prefixo),
        Err("consulta_de_anexacao_nao_publicada")
    );
    assert_eq!(
        url_da_consulta(
            "new infraAjaxComplementar('txtProtocolo','https://outro.test/')",
            prefixo
        ),
        Err("consulta_de_anexacao_fora_da_forma_observada")
    );
}
#[test]
fn botao_submit_da_unb_conta_como_controle_de_anexar() {
    // UnB 4.0.9 ao vivo (10/11/2026): <button type="submit" name="sbmAnexar">.
    let tela = html().replace(
        r#"<input name="sbmAnexar" type="submit" value="Anexar">"#,
        r#"<button type="submit" name="sbmAnexar" id="sbmAnexar" value="Anexar">Anexar</button>"#,
    );
    assert_eq!(ler_tela(&tela).unwrap().anexados, vec!["22"]);
}
