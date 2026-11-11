//! Testes sintéticos: cenários dos testes do Todos @323eb01.
//! Os testes foram escritos antes do leitor Rust (RED local indisponível).
use super::tela::{
    ErroDestinoDaConsulta, ErroTelaDeRelacionamentos, ler_destino, ler_linhas, url_complementar,
};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::tela_html::HtmlServido;

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
    let lidos = ler_linhas(&HtmlServido::de_teste(&tela())).unwrap();
    assert_eq!(lidos.len(), 1);
    assert_eq!(lidos[0].id, "22");
    assert_eq!(lidos[0].nup, "12345.000002/2026-01");
    assert!(lidos[0].pode_remover);
    let sem_oferta = HtmlServido::de_teste(&tela().replace("acaoRemover('22');", "outraCoisa();"));
    assert!(!ler_linhas(&sem_oferta).unwrap()[0].pode_remover);
}

#[test]
fn nao_toma_texto_solto_como_relacao() {
    let html =
        HtmlServido::de_teste(&tela().replace("</table>", "</table><p>99999.000001/2026-01</p>"));
    assert_eq!(ler_linhas(&html).unwrap().len(), 1);
    assert_eq!(ler_linhas(&html).unwrap()[0].id, "22");
}

#[test]
fn repeticao_de_id_e_de_tabela_falha_fechado() {
    let linha = r#"<tr><td><a href="controlador.php?acao=procedimento_trabalhar&id_procedimento=22">99999</a></td></tr>"#;
    assert_eq!(
        ler_linhas(&HtmlServido::de_teste(
            &tela().replace("</table>", &format!("{linha}</table>"))
        )),
        Err(ErroTelaDeRelacionamentos::RelacionamentosRepetidosNaTela)
    );
    let duplicada = HtmlServido::de_teste(&tela().replace(
        "</form>",
        r#"<table summary="Lista de Processos Relacionados"></table></form>"#,
    ));
    assert_eq!(
        ler_linhas(&duplicada),
        Err(ErroTelaDeRelacionamentos::TabelaDeRelacionamentosAusenteOuRepetida)
    );
}

#[test]
fn sem_tabela_e_falta_da_consulta_sao_recusas_tipadas() {
    assert_eq!(
        ler_linhas(&HtmlServido::de_teste("<html></html>")),
        Err(
            ErroTelaDeRelacionamentos::FormularioDeRelacionamentosNaoLocalizado(
                ErroSubmissaoFormulario::FormularioAusenteNaTela
            )
        )
    );
    assert!(
        url_complementar(&tela())
            .unwrap()
            .contains("acao_ajax=protocolo_RI1023")
    );
    assert_eq!(url_complementar("<script></script>"), None);
}

#[test]
fn xml_complementos_identifica_destino_sem_inventar_id() {
    let xml = HtmlServido::de_teste(
        r#"<complementos><complemento nome="IdProcedimento">50698</complemento><complemento nome="ProtocoloProcedimentoFormatado">12345.000001/2026-01</complemento><complemento nome="NomeTipoProcedimento">Tipo</complemento></complementos>"#,
    );
    let d = ler_destino(&xml).unwrap();
    assert_eq!(d.id, "50698");
    assert_eq!(d.nup, "12345.000001/2026-01");
    assert_eq!(
        ler_destino(&HtmlServido::de_teste(
            "<erros><erro descricao='Não encontrado.'/></erros>"
        ))
        .err(),
        Some(ErroDestinoDaConsulta::ProcessoDeDestinoNaoEncontrado)
    );
}

#[test]
fn xml_destino_duplicado_ou_vazio_e_recusado() {
    let duplicado = HtmlServido::de_teste(
        r#"<complementos><complemento nome="IdProcedimento">11</complemento><complemento nome="IdProcedimento">12</complemento><complemento nome="ProtocoloProcedimentoFormatado">00001</complemento></complementos>"#,
    );
    assert_eq!(
        ler_destino(&duplicado).err(),
        Some(ErroDestinoDaConsulta::DestinoDaConsultaAmbiguo)
    );
    assert_eq!(
        ler_destino(&HtmlServido::de_teste("<complementos/>")).err(),
        Some(ErroDestinoDaConsulta::DestinoARelacionarNaoIdentificado)
    );
}

#[test]
fn o_numero_de_controle_nao_e_reconhecido_como_id_de_processo() {
    let html = HtmlServido::de_teste(&tela().replace("id_procedimento=22", "id_procedimento=22X"));
    assert!(matches!(ler_linhas(&html).unwrap().as_slice(), []));
}

fn tela_com_remocao_servida() -> String {
    // Estrutura literal do acaoRemover da página PEN1 em Todos @323eb01.
    let hash = "a".repeat(64);
    format!(
        r#"<html><head><script>
function acaoRemover(id){{
  if (confirm("Confirma exclusão do relacionamento?")){{
    document.getElementById('hdnInfraItemId').value=id;
    document.getElementById('frmProcedimentoListarRelacionamentos').action='controlador.php?acao=procedimento_excluir_relacionamento&acao_origem=procedimento_relacionar&arvore=1&id_procedimento=50693&infra_hash={hash}';
    document.getElementById('frmProcedimentoListarRelacionamentos').submit();
  }}
}}
</script></head></html>"#
    )
}

/// Dado o handler ativo observado, quando preparo a remoção,
/// então uso exclusivamente o destino de seu .action, não texto solto.
#[test]
fn extrai_action_da_funcao_de_remocao() {
    let url =
        super::tela::url_exclusao(&HtmlServido::de_teste(&tela_com_remocao_servida())).unwrap();
    assert_eq!(
        url,
        format!(
            "controlador.php?acao=procedimento_excluir_relacionamento&acao_origem=procedimento_relacionar&arvore=1&id_procedimento=50693&infra_hash={}",
            "a".repeat(64)
        )
    );
}

/// Dado JS sem acaoRemover ativo, quando há action mencionada fora dele,
/// então a CLI não promove o literal a permissão de remoção.
#[test]
fn action_solto_comentado_ou_em_template_nao_autoriza_remocao() {
    let servido = tela_com_remocao_servida();
    let apenas_outra_funcao =
        HtmlServido::de_teste(&servido.replace("function acaoRemover(id)", "function outra(id)"));
    assert!(super::tela::url_exclusao(&apenas_outra_funcao).is_none());

    let em_template = HtmlServido::de_teste(&format!(
        "<html><head><template>{servido}</template></head></html>"
    ));
    assert!(super::tela::url_exclusao(&em_template).is_none());

    let em_comentario =
        servido.replace("function acaoRemover(id){", "/*\nfunction acaoRemover(id){");
    assert!(super::tela::url_exclusao(&HtmlServido::de_teste(&em_comentario)).is_none());
}

/// Dado que falta o campo ou o submit do handler, ou há função duplicada,
/// quando preparo a escrita, então a origem não está demonstrada.
#[test]
fn remocao_sem_passos_ou_com_handler_duplicado_e_recusada() {
    let servido = tela_com_remocao_servida();
    assert!(
        super::tela::url_exclusao(&HtmlServido::de_teste(
            &servido.replace("getElementById('hdnInfraItemId').value=id;", "void(0);")
        ))
        .is_none()
    );
    assert!(
        super::tela::url_exclusao(&HtmlServido::de_teste(&servido.replace(
            "getElementById('frmProcedimentoListarRelacionamentos').submit();",
            "void(0);",
        )))
        .is_none()
    );
    let duplicada = HtmlServido::de_teste(
        &servido.replace("</script>", "function acaoRemover(id){}\n</script>"),
    );
    assert!(super::tela::url_exclusao(&duplicada).is_none());
}

/// Dado o script do SEI 4 da `UnB` envolto em `<!--//--><![CDATA[//><!--`
/// (live de 10/11/2026), quando preparo a remoção, então a função é achada.
#[test]
fn extrai_action_de_script_envolto_em_comentario_cdata_da_unb() {
    let envolto = HtmlServido::de_teste(&tela_com_remocao_servida()
        .replace("<script>\n", "<script type=\"text/javascript\" charset=\"iso-8859-1\" >\n<!--//--><![CDATA[//><!--\n")
        .replace("</script>", "//--><!]]>\n</script>"));
    let url = super::tela::url_exclusao(&envolto).unwrap();
    assert!(url.starts_with("controlador.php?acao=procedimento_excluir_relacionamento&"));
}
