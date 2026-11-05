//! Testes sintéticos: cenários dos testes do Todos @323eb01.
//! Os testes foram escritos antes do leitor Rust (RED local indisponível).
use super::tela::{ler_destino, ler_linhas, url_complementar};

fn tela() -> String {
    let hash = "a".repeat(64);
    format!(
        r##"<html><script>
objAjax = new infraAjaxComplementar('txtProtocolo','controlador_ajax.php?acao_ajax=protocolo_RI1023&infra_hash={hash}');
</script><form id="frmProcedimentoListarRelacionamentos" method="post" action="controlador.php?acao=procedimento_relacionar&infra_hash={hash}">
<input name="hdnInfraTipoPagina" type="hidden" value="2" />
<table summary="Lista de Processos Relacionados">
<tr><th>Processo</th></tr>
<tr><td><a class="protocoloAberto" href="controlador.php?acao=procedimento_trabalhar&id_procedimento=22&infra_hash={hash}">12345.000002/2026-01</a></td>
<td><a href="#ID-22" onclick="acaoRemover('22');">Remover</a></td></tr>
</table></form></html>"##
    )
}

#[test]
fn le_linha_e_remove_somente_se_funcao_na_mesma_linha() {
    let lidos = ler_linhas(&tela()).unwrap();
    assert_eq!(lidos.len(), 1);
    assert_eq!(lidos[0].id, "22");
    assert_eq!(lidos[0].nup, "12345.000002/2026-01");
    assert!(lidos[0].pode_remover);
    let sem_oferta = tela().replace("acaoRemover('22');", "outraCoisa();");
    assert!(!ler_linhas(&sem_oferta).unwrap()[0].pode_remover);
}

#[test]
fn nao_toma_texto_solto_como_relacao() {
    let html = tela().replace("</table>", "</table><p>99999.000001/2026-01</p>");
    assert_eq!(ler_linhas(&html).unwrap().len(), 1);
    assert_eq!(ler_linhas(&html).unwrap()[0].id, "22");
}

#[test]
fn repeticao_de_id_e_de_tabela_falha_fechado() {
    let linha = r#"<tr><td><a href="controlador.php?acao=procedimento_trabalhar&id_procedimento=22">99999</a></td></tr>"#;
    assert_eq!(
        ler_linhas(&tela().replace("</table>", &format!("{linha}</table>"))),
        Err("relacionamentos_repetidos_na_tela")
    );
    let duplicada = tela().replace(
        "</form>",
        r#"<table summary="Lista de Processos Relacionados"></table></form>"#,
    );
    assert_eq!(
        ler_linhas(&duplicada),
        Err("tabela_de_relacionamentos_ausente_ou_repetida")
    );
}

#[test]
fn sem_tabela_e_falta_da_consulta_sao_recusas_tipadas() {
    assert_eq!(
        ler_linhas("<html></html>"),
        Err("formulario_de_relacionamentos_ausente_ou_repetido")
    );
    assert!(
        url_complementar(&tela())
            .unwrap()
            .contains("acao_ajax=protocolo_RI1023")
    );
    assert_eq!(
        url_complementar("<script></script>"),
        Err("consulta_complementar_de_relacionamento_nao_declarada")
    );
}

#[test]
fn xml_complementos_identifica_destino_sem_inventar_id() {
    let xml = r#"<complementos><complemento nome="IdProcedimento">50698</complemento><complemento nome="ProtocoloProcedimentoFormatado">12345.000001/2026-01</complemento><complemento nome="NomeTipoProcedimento">Tipo</complemento></complementos>"#;
    let d = ler_destino(xml).unwrap();
    assert_eq!(d.id, "50698");
    assert_eq!(d.nup, "12345.000001/2026-01");
    assert_eq!(
        ler_destino("<erros><erro descricao='Não encontrado.'/></erros>").err(),
        Some("processo_de_destino_nao_encontrado")
    );
}

#[test]
fn xml_destino_duplicado_ou_vazio_e_recusado() {
    let duplicado = r#"<complementos><complemento nome="IdProcedimento">11</complemento><complemento nome="IdProcedimento">12</complemento><complemento nome="ProtocoloProcedimentoFormatado">00001</complemento></complementos>"#;
    assert_eq!(
        ler_destino(duplicado).err(),
        Some("destino_da_consulta_ambiguo")
    );
    assert_eq!(
        ler_destino("<complementos/>").err(),
        Some("destino_a_relacionar_nao_identificado")
    );
}

#[test]
fn o_numero_de_controle_nao_e_reconhecido_como_id_de_processo() {
    let html = tela().replace("id_procedimento=22", "id_procedimento=22X");
    assert!(matches!(ler_linhas(&html).unwrap().as_slice(), []));
}
