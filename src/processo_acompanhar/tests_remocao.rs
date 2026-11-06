//! RED/GREEN da retirada: forma observada na `UnB`, transporte injetado.
//! Cenários de Todos `tests/test_processo_acompanhar_010.py` @323eb01.
use super::*;
use crate::processo_concluir::tests_apoio::{
    ID, Pedido as PedidoHttp, Resposta, UNIDADE, abertura, absoluta, ambiente, arvore, get, ok,
    pedidos_da_abertura, post, r302, rodar_com,
};
use crate::propriedades_processo::tests_apoio::acao_da_barra;
use crate::transporte::FalhaTransporte;

fn gerenciar() -> String {
    format!(
        "controlador.php?acao=acompanhamento_gerenciar&id_procedimento={ID}&infra_hash={}",
        "a".repeat(64)
    )
}
fn excluir() -> String {
    format!(
        "controlador.php?acao=acompanhamento_excluir&acao_origem=acompanhamento_gerenciar&id_procedimento={ID}&infra_hash={}",
        "b".repeat(64)
    )
}
fn cadastrar() -> String {
    format!(
        "controlador.php?acao=acompanhamento_cadastrar&id_procedimento={ID}&infra_hash={}",
        "c".repeat(64)
    )
}
fn cadastro() -> String {
    format!(
        "<form id=\"frmAcompanhamentoCadastro\" method=\"post\" action=\"{}\">\
        <input type=\"hidden\" name=\"hdnIdProtocolo\" value=\"{ID}\">\
        <select name=\"selGrupoAcompanhamento\"><option value=\"null\">&nbsp;</option></select>\
        </form>",
        cadastrar().replace('&', "&amp;")
    )
}
fn lista(linhas: &str) -> String {
    format!(
        r#"<html><head><script>
        function acaoExcluir(id,desc){{
          if(confirm("Excluir?")){{
            document.getElementById('hdnInfraItemId').value=id;
            document.getElementById('frmGerenciarAcompanhamento').action='{}';
            document.getElementById('frmGerenciarAcompanhamento').submit();
          }}
        }}
        </script></head><body>
        <form id="frmGerenciarAcompanhamento" method="post" action="{}">
          <input type="hidden" name="hdnInfraItemId" id="hdnInfraItemId" value="">
          <table summary="Tabela de Acompanhamentos.">
            <tr><th></th><th>Grupo</th><th>Observação</th><th>Usuário</th><th>Data</th><th>Ações</th></tr>
            {linhas}
          </table>
        </form></body></html>"#,
        excluir(),
        gerenciar()
    )
}
fn linha(id: &str, grupo: &str) -> String {
    format!(
        r##"<tr><td><input type="checkbox" value="{id}"></td>
        <td>{grupo}</td><td>Obs.</td><td>Aluna</td><td>05/11</td>
        <td><a href="#ID-{id}" onclick="acaoExcluir('{id}','Obs.');">Excluir</a></td></tr>"##
    )
}
fn preparar() -> Vec<Resposta> {
    let arvore = arvore(&[UNIDADE], &[acao_da_barra(&gerenciar())]);
    let mut respostas = abertura(arvore);
    respostas.push(ok(lista(&linha("831", "G"))));
    respostas
}
fn executar(respostas: Vec<Resposta>) -> (Result<Previa, ErroPrevia>, Vec<PedidoHttp>) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
        previa(
            sessao,
            &Pedido {
                nup,
                unidade: None,
                grupo: Some("G"),
                observacao: None,
                deixar: true,
                confirmar: true,
            },
            indice,
            roteiro,
        )
    })
}
fn pedidos_confirmados() -> Vec<PedidoHttp> {
    let mut pedidos = pedidos_da_abertura();
    pedidos.push(get(&absoluta(&gerenciar())));
    pedidos.push(post(&absoluta(&excluir()), "hdnInfraItemId=831"));
    pedidos.push(get(&absoluta(&gerenciar())));
    pedidos
}

#[test]
fn retirada_confirmada_um_post_e_releitura_de_tabela() {
    let mut respostas = preparar();
    respostas.extend([r302(&gerenciar()), ok(lista(""))]);
    let (resultado, pedidos) = executar(respostas);
    let plano = resultado.unwrap();
    assert!(plano.confirmado && plano.alterado && !plano.ja_estava);
    assert_eq!(plano.id_de_remocao.as_deref(), Some("831"));
    assert_eq!(plano.requisicoes, 6);
    assert_eq!(pedidos, pedidos_confirmados());
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

#[test]
fn retirada_do_ultimo_confere_cadastro_apos_redirecionamento() {
    let mut respostas = preparar();
    respostas.extend([r302(&gerenciar()), r302(&cadastrar()), ok(cadastro())]);
    let (resultado, pedidos) = executar(respostas);
    let plano = resultado.unwrap();
    assert!(plano.confirmado && plano.alterado);
    assert_eq!(plano.requisicoes, 7);
    let mut esperados = pedidos_confirmados();
    esperados.push(get(&absoluta(&cadastrar())));
    assert_eq!(pedidos, esperados);
}

#[test]
fn resposta_de_post_sem_aceite_do_processo_e_incerta_sem_retry() {
    let mut respostas = preparar();
    let desvio = gerenciar().replace("id_procedimento=10", "id_procedimento=999");
    respostas.extend([r302(&desvio), ok(lista(""))]);
    let (resultado, pedidos) = executar(respostas);
    assert!(matches!(resultado, Err(ErroPrevia::EfeitoIncerto { .. })));
    assert_eq!(pedidos, pedidos_confirmados());
}

#[test]
fn falha_na_resposta_do_post_ainda_rele_com_efeito_incerto() {
    let mut respostas = preparar();
    respostas.extend([Err(FalhaTransporte::Rede), ok(lista(""))]);
    let (resultado, pedidos) = executar(respostas);
    assert!(matches!(resultado, Err(ErroPrevia::EfeitoIncerto { .. })));
    assert_eq!(pedidos, pedidos_confirmados());
}

#[test]
fn sem_prova_na_releitura_da_retirada_e_incerto() {
    let mut respostas = preparar();
    respostas.extend([r302(&gerenciar()), ok(lista(&linha("831", "G")))]);
    let (resultado, pedidos) = executar(respostas);
    assert!(matches!(resultado, Err(ErroPrevia::EfeitoIncerto { .. })));
    assert_eq!(pedidos, pedidos_confirmados());
}

#[test]
fn cadastro_apos_retirar_ultimo_precisa_publicar_processo() {
    let mut respostas = preparar();
    respostas.extend([
        r302(&gerenciar()),
        r302(&cadastrar()),
        ok("<html>cortado</html>".to_owned()),
    ]);
    let (resultado, pedidos) = executar(respostas);
    assert!(matches!(resultado, Err(ErroPrevia::EfeitoIncerto { .. })));
    let mut esperados = pedidos_confirmados();
    esperados.push(get(&absoluta(&cadastrar())));
    assert_eq!(pedidos, esperados);
}
