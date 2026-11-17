//! Testes Web sintéticos da prévia: mesmo transporte roteirizado de
//! processo concluir; contagem exata, zero POST e decisão antes da escrita.
use super::{Acao, ErroSobrestamento, preparar};
use crate::arvore_processo::tests_concluir::arvore_para_concluir;
use crate::processo_concluir::tests_apoio::{
    ID, NUP, UNIDADE, abertura, ambiente, ancora, arvore, pedidos_da_abertura, rodar_com,
};
use crate::propriedades_processo::tests_apoio::acao_da_barra;

fn sobrestar() -> String {
    acao_da_barra(&format!(
        "controlador.php?acao=procedimento_sobrestar&id_procedimento={ID}&infra_hash={}",
        "e".repeat(64)
    ))
}
fn retomar() -> String {
    "<a href=\"#\" onclick=\"removerSobrestamentoProcesso();\">Retomar</a>".to_owned()
}
fn sobrestada(sigla: &str) -> String {
    arvore_para_concluir(
        ID,
        NUP,
        &format!("Processo sobrestado na unidade {}.<br />", ancora(sigla)),
        &[retomar()],
    )
}

#[test]
fn sobrestar_previa_lida_so_tres_gets_e_sem_post() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        abertura(arvore(&[UNIDADE], &[sobrestar()])),
        |sessao, nup, indice, envio| preparar(sessao, nup, None, Acao::Sobrestar, indice, envio),
    );
    let previa = r.unwrap();
    assert_eq!(previa.acao, Acao::Sobrestar);
    assert_eq!(previa.requisicoes, 3);
    assert_eq!(previa.unidade, UNIDADE);
    assert_eq!(pedidos, pedidos_da_abertura());
}
#[test]
fn retomar_previa_lida_estado_sobrestado_sem_post() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        abertura(sobrestada(UNIDADE)),
        |sessao, nup, indice, envio| preparar(sessao, nup, None, Acao::Retomar, indice, envio),
    );
    let previa = r.unwrap();
    assert_eq!(previa.acao, Acao::Retomar);
    assert_eq!(previa.requisicoes, 3);
    assert_eq!(pedidos, pedidos_da_abertura());
}
#[test]
fn sobrestar_ja_sobrestado_falha_fechado() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        abertura(sobrestada(UNIDADE)),
        |sessao, nup, indice, envio| preparar(sessao, nup, None, Acao::Sobrestar, indice, envio),
    );
    assert_eq!(r, Err(ErroSobrestamento::JaSobrestado));
    assert_eq!(pedidos, pedidos_da_abertura());
}
#[test]
fn retomar_sem_sobrestamento_falha_fechado() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        abertura(arvore(&[UNIDADE], &[sobrestar()])),
        |sessao, nup, indice, envio| preparar(sessao, nup, None, Acao::Retomar, indice, envio),
    );
    assert_eq!(r, Err(ErroSobrestamento::NaoSobrestado));
    assert_eq!(pedidos, pedidos_da_abertura());
}
#[test]
fn sobrestar_sem_link_e_retomar_de_outra_unidade_recusados() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        abertura(arvore(&[UNIDADE], &[])),
        |sessao, nup, indice, envio| preparar(sessao, nup, None, Acao::Sobrestar, indice, envio),
    );
    assert_eq!(r, Err(ErroSobrestamento::AcaoNaoOferecida));
    assert_eq!(pedidos, pedidos_da_abertura());
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        abertura(sobrestada("OUTRA")),
        |sessao, nup, indice, envio| preparar(sessao, nup, None, Acao::Retomar, indice, envio),
    );
    assert_eq!(r, Err(ErroSobrestamento::SobrestadoEmOutraUnidade));
    assert_eq!(pedidos, pedidos_da_abertura());
}
