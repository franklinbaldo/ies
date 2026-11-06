//! Testes de transporte para escrita confirmada (--confirmar) e releitura.
use super::{FalhaRelacionar, Pedido, prever};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::tests_apoio::{
    ID, Pedido as PedidoHttp, Resposta, UNIDADE, abertura, ambiente, arvore, ok, r302, rodar_com,
};
use crate::propriedades_processo::tests_apoio::acao_da_barra;

const NUP_DESTINO: &str = "12345.000002/2026-01";
const ID_DESTINO: &str = "22";

fn link_relacionar_barra() -> String {
    acao_da_barra(&format!(
        "controlador.php?acao=procedimento_relacionar&id_procedimento={ID}&infra_hash={}",
        "a".repeat(64)
    ))
}

fn tela_relacionar_vazia() -> String {
    let hash = "a".repeat(64);
    format!(
        r#"<html><script>
objAjax = new infraAjaxComplementar('txtProtocolo','controlador_ajax.php?acao_ajax=protocolo_RI1023&infra_hash={hash}');
function acaoRemover(id){{
  document.getElementById('frmProcedimentoListarRelacionamentos').action='controlador.php?acao=procedimento_excluir_relacionamento&id_procedimento={ID}&infra_hash={hash}';
  document.getElementById('frmProcedimentoListarRelacionamentos').submit();
}}
</script><form id="frmProcedimentoListarRelacionamentos" method="post" action="controlador.php?acao=procedimento_relacionar&id_procedimento={ID}&infra_hash={hash}">
<input name="hdnInfraTipoPagina" type="hidden" value="2" />
<input type="text" id="txtProtocolo" name="txtProtocolo" value="" />
<input type="text" id="txtIdentificacaoProtocolo" name="txtIdentificacaoProtocolo" value="" />
<input type="hidden" id="hdnIdProtocolo" name="hdnIdProtocolo" value="" />
<input type="hidden" id="hdnInfraItemId" name="hdnInfraItemId" value="" />
<button type="submit" name="sbmAdicionar" id="sbmAdicionar" value="Adicionar">Adicionar</button>
<table summary="Lista de Processos Relacionados">
<tr><th>Processo</th></tr>
</table></form></html>"#
    )
}

fn tela_relacionar_com_destino() -> String {
    let hash = "a".repeat(64);
    format!(
        r##"<html><script>
objAjax = new infraAjaxComplementar('txtProtocolo','controlador_ajax.php?acao_ajax=protocolo_RI1023&infra_hash={hash}');
function acaoRemover(id){{
  document.getElementById('frmProcedimentoListarRelacionamentos').action='controlador.php?acao=procedimento_excluir_relacionamento&id_procedimento={ID}&infra_hash={hash}';
  document.getElementById('frmProcedimentoListarRelacionamentos').submit();
}}
</script><form id="frmProcedimentoListarRelacionamentos" method="post" action="controlador.php?acao=procedimento_relacionar&id_procedimento={ID}&infra_hash={hash}">
<input name="hdnInfraTipoPagina" type="hidden" value="2" />
<input type="text" id="txtProtocolo" name="txtProtocolo" value="" />
<input type="text" id="txtIdentificacaoProtocolo" name="txtIdentificacaoProtocolo" value="" />
<input type="hidden" id="hdnIdProtocolo" name="hdnIdProtocolo" value="" />
<input type="hidden" id="hdnInfraItemId" name="hdnInfraItemId" value="" />
<button type="submit" name="sbmAdicionar" id="sbmAdicionar" value="Adicionar">Adicionar</button>
<table summary="Lista de Processos Relacionados">
<tr><th>Processo</th></tr>
<tr><td><a class="protocoloAberto" href="controlador.php?acao=procedimento_trabalhar&id_procedimento={ID_DESTINO}&infra_hash={hash}">{NUP_DESTINO}</a></td>
<td><a href="#ID-{ID_DESTINO}" onclick="acaoRemover('{ID_DESTINO}');">Remover</a></td></tr>
</table></form></html>"##
    )
}

fn xml_ajax_destino() -> String {
    format!(
        r#"<complementos><complemento nome="IdProcedimento">{ID_DESTINO}</complemento><complemento nome="ProtocoloProcedimentoFormatado">{NUP_DESTINO}</complemento><complemento nome="NomeTipoProcedimento">Processo de Teste</complemento></complementos>"#
    )
}

fn rodar_relacionar(
    com_nup: &str,
    remover: bool,
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<super::Previa, FalhaRelacionar>, Vec<PedidoHttp>) {
    let amb = ambiente();
    let nup_com = NumeroProcesso::novo(com_nup).unwrap();
    rodar_com(&amb, respostas, |sessao, nup, indice, envio| {
        let pedido = Pedido {
            nup,
            com: &nup_com,
            unidade: None,
            remover,
            confirmar,
        };
        prever(sessao, &pedido, indice, envio)
    })
}

#[test]
fn relacionar_confirmar_adiciona_com_sucesso_7_requisicoes_a_frio() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[link_relacionar_barra()]));
    respostas.push(ok(tela_relacionar_vazia())); // 1: GET tela relacionar
    respostas.push(ok(xml_ajax_destino())); // 2: POST consulta AJAX
    respostas.push(r302("controlador.php?acao=procedimento_relacionar")); // 3: POST escrita
    respostas.push(ok(tela_relacionar_com_destino())); // 4: GET releitura de verificação

    let (res, pedidos) = rodar_relacionar(NUP_DESTINO, false, true, respostas);
    let previa = res.expect("adicionar relacionamento com confirmação deve suceder");
    assert!(previa.confirmado);
    assert!(previa.alterado);
    assert!(!previa.sem_efeito);
    assert_eq!(previa.requisicoes, 7);
    assert_eq!(pedidos.len(), 7);
    assert_eq!(pedidos[5].metodo, "POST");
    assert_eq!(pedidos[6].metodo, "GET");
}

#[test]
fn relacionar_remover_confirmar_sucesso_6_requisicoes_a_frio() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[link_relacionar_barra()]));
    respostas.push(ok(tela_relacionar_com_destino())); // 1: GET tela relacionar (com destino)
    respostas.push(r302("controlador.php?acao=procedimento_relacionar")); // 2: POST excluir
    respostas.push(ok(tela_relacionar_vazia())); // 3: GET releitura (agora sem destino)

    let (res, pedidos) = rodar_relacionar(NUP_DESTINO, true, true, respostas);
    let previa = res.expect("remover relacionamento com confirmação deve suceder");
    assert!(previa.confirmado);
    assert!(previa.alterado);
    assert!(!previa.sem_efeito);
    assert_eq!(previa.requisicoes, 6);
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos[4].metodo, "POST");
    assert_eq!(pedidos[5].metodo, "GET");
}

#[test]
fn relacionar_confirmar_sem_efeito_se_ja_relacionado_sem_post() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[link_relacionar_barra()]));
    respostas.push(ok(tela_relacionar_com_destino())); // 1: GET tela relacionar
    respostas.push(ok(xml_ajax_destino())); // 2: POST consulta AJAX

    let (res, pedidos) = rodar_relacionar(NUP_DESTINO, false, true, respostas);
    let previa = res.expect("relação já existente com confirmação sai com sucesso sem efeito");
    assert!(!previa.confirmado);
    assert!(!previa.alterado);
    assert!(previa.sem_efeito);
    assert_eq!(previa.requisicoes, 5);
    assert_eq!(pedidos.len(), 5);
}

#[test]
fn relacionar_remover_confirmar_sem_efeito_se_nao_relacionado_sem_post() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[link_relacionar_barra()]));
    respostas.push(ok(tela_relacionar_vazia())); // 1: GET tela relacionar

    let (res, pedidos) = rodar_relacionar(NUP_DESTINO, true, true, respostas);
    let previa = res.expect("remoção sem relação com confirmação sai com sucesso sem efeito");
    assert!(!previa.confirmado);
    assert!(!previa.alterado);
    assert!(previa.sem_efeito);
    assert_eq!(previa.requisicoes, 4);
    assert_eq!(pedidos.len(), 4);
}

#[test]
fn relacionar_confirmar_efeito_incerto_se_releitura_nao_reflete() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[link_relacionar_barra()]));
    respostas.push(ok(tela_relacionar_vazia())); // 1: GET tela relacionar
    respostas.push(ok(xml_ajax_destino())); // 2: POST consulta AJAX
    respostas.push(r302("controlador.php?acao=procedimento_relacionar")); // 3: POST escrita
    respostas.push(ok(tela_relacionar_vazia())); // 4: GET releitura (mas AINDA vazia!)

    let (res, pedidos) = rodar_relacionar(NUP_DESTINO, false, true, respostas);
    assert!(matches!(res, Err(FalhaRelacionar::EfeitoIncerto)));
    assert_eq!(pedidos.len(), 7);
}
