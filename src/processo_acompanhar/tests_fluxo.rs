//! Transporte sintético do cadastro: GETs, POST único e leitura de confirmação.
//! Caso de Todos `tests/test_processo_acompanhar_010.py` @323eb01.
use super::*;
use crate::processo_concluir::tests_apoio::{
    ID, UNIDADE, abertura, absoluta, ambiente, arvore, get, ok, post, r302, rodar_com,
};
use crate::propriedades_processo::tests_apoio::acao_da_barra;
use crate::transporte::FalhaTransporte;

fn gerenciar() -> String {
    format!(
        "controlador.php?acao=acompanhamento_gerenciar&id_procedimento={ID}&infra_hash={}",
        "a".repeat(64)
    )
}
fn cadastrar() -> String {
    format!(
        "controlador.php?acao=acompanhamento_cadastrar&id_procedimento={ID}&infra_hash={}",
        "b".repeat(64)
    )
}
fn action() -> String {
    format!(
        "controlador.php?acao=acompanhamento_cadastrar&id_procedimento={ID}&infra_hash={}",
        "c".repeat(64)
    )
}
fn formulario() -> String {
    format!(
        "<form id=\"frmAcompanhamentoCadastro\" method=\"post\" action=\"{}\">\
         <input type=\"hidden\" name=\"hdnIdProtocolo\" value=\"{ID}\">\
         <select name=\"selGrupoAcompanhamento\">\
         <option value=\"null\">&nbsp;</option><option value=\"19\">G</option></select>\
         <textarea name=\"txaObservacao\"></textarea>\
         <button type=\"submit\" name=\"sbmCadastrarAcompanhamento\" value=\"Salvar\">Salvar</button>\
         </form>",
        action().replace('&', "&amp;")
    )
}
fn lista(grupo: &str) -> String {
    format!(
        "<table summary=\"Tabela de Acompanhamentos.\">\
         <tr><th></th><th>Grupo</th><th>Observação</th>\
         <th>Usuário</th><th>Data</th><th>Ações</th></tr>\
         <tr><td><input type=\"checkbox\" value=\"831\"></td>\
         <td>{grupo}</td><td>Prazo.</td><td>aluno</td><td>05/11/2026</td><td>ação</td></tr></table>"
    )
}
fn preparar() -> Vec<crate::processo_concluir::tests_apoio::Resposta> {
    let html = arvore(&[UNIDADE], &[acao_da_barra(&gerenciar())]);
    let mut respostas = abertura(html);
    respostas.extend([r302(&cadastrar()), ok(formulario())]);
    respostas
}
fn executar(
    respostas: Vec<crate::processo_concluir::tests_apoio::Resposta>,
    confirmar: bool,
) -> (
    Result<Previa, ErroPrevia>,
    Vec<crate::processo_concluir::tests_apoio::Pedido>,
) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
        let pedido = Pedido {
            nup,
            unidade: None,
            grupo: Some("G"),
            observacao: Some("Prazo."),
            deixar: false,
            confirmar,
        };
        previa(sessao, &pedido, indice, roteiro)
    })
}
fn requests_post() -> Vec<crate::processo_concluir::tests_apoio::Pedido> {
    let mut pedidos = crate::processo_concluir::tests_apoio::pedidos_da_abertura();
    pedidos.extend([
        get(&absoluta(&gerenciar())),
        get(&absoluta(&cadastrar())),
        post(
            &absoluta(&action()),
            "hdnIdProtocolo=10&selGrupoAcompanhamento=19&txaObservacao=Prazo.&sbmCadastrarAcompanhamento=Salvar",
        ),
        get(&absoluta(&gerenciar())),
    ]);
    pedidos
}

#[test]
fn previa_deriva_formulario_sem_post_com_orcamento_exato() {
    let (resultado, pedidos) = executar(preparar(), false);
    let plano = resultado.unwrap();
    assert!(!plano.estado.confirmado() && !plano.estado.alterado() && !plano.estado.ja_estava());
    assert_eq!(plano.requisicoes, 5);
    assert_eq!(pedidos, &requests_post()[..5]);
}

#[test]
fn confirmar_post_unico_e_reler_lista_mesmo_processo() {
    let mut respostas = preparar();
    respostas.extend([r302(&gerenciar()), ok(lista("G"))]);
    let (resultado, pedidos) = executar(respostas, true);
    let feito = resultado.unwrap();
    assert!(feito.estado.confirmado() && feito.estado.alterado() && !feito.estado.ja_estava());
    assert_eq!(feito.id_de_remocao.as_deref(), Some("831"));
    assert_eq!(feito.requisicoes, 7);
    assert_eq!(pedidos, requests_post());
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

#[test]
fn falha_de_rede_no_post_nao_repete_e_reconcilia_pela_leitura() {
    let mut respostas = preparar();
    respostas.extend([Err(FalhaTransporte::Rede), ok(lista("G"))]);
    let (resultado, pedidos) = executar(respostas, true);
    assert!(resultado.unwrap().estado.alterado());
    assert_eq!(pedidos, requests_post());
}

#[test]
fn falha_na_releitura_depois_do_post_e_efeito_incerto_sem_retry() {
    let mut respostas = preparar();
    respostas.extend([r302(&gerenciar()), Err(FalhaTransporte::Rede)]);
    let (resultado, pedidos) = executar(respostas, true);
    assert!(matches!(resultado, Err(ErroPrevia::EfeitoIncerto)));
    assert_eq!(pedidos, requests_post());
}

#[test]
fn lista_apos_post_sem_linha_nova_e_efeito_incerto() {
    let mut respostas = preparar();
    respostas.extend([r302(&gerenciar()), ok(lista("Outro"))]);
    let (resultado, pedidos) = executar(respostas, true);
    assert!(matches!(resultado, Err(ErroPrevia::EfeitoIncerto)));
    assert_eq!(pedidos, requests_post());
}
