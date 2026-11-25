//! Casos sintéticos derivados de Todos/src/todos/ciclo.py e dos índices
//! de cassettes processo-sobrestar-retomar (`UnB`, GO e PEN1).
//! Não há URL real, cookie real ou segredo nestas fixtures.
use super::sobrestar::{ErroAcaoSobrestar, ler_retomar_da_barra, ler_sobrestar_da_barra};
use super::tests::NUP;
use super::tests_concluir::{absoluta, arvore_para_concluir, link_da_tela_do_no};
use super::tests_tela::instalacao;
use crate::propriedades_processo::tests_apoio::acao_da_barra;

fn link(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_sobrestar&acao_origem=arvore_visualizar&id_procedimento={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        "e".repeat(64)
    )
}
fn barra(id: &str) -> String {
    acao_da_barra(&link(id))
}
fn arvore(acoes: &[String]) -> String {
    arvore_para_concluir("10", NUP, "", acoes)
}
fn retomar() -> String {
    "<a href=\"#\" onclick=\"removerSobrestamentoProcesso();\">Retomar</a>".into()
}

#[test]
fn observa_link_assinado_de_sobrestar_no_proprio_processo() {
    let html = arvore(&[barra("10")]);
    let url = ler_sobrestar_da_barra(&instalacao(), &html, "10")
        .unwrap()
        .expect("link servido");
    assert_eq!(url.as_str(), absoluta(&link("10")));
    assert!(!format!("{url:?}").contains(&"e".repeat(64)));
}
#[test]
fn nao_escolhe_entre_links_de_sobrestar_duplicados() {
    let html = arvore(&[barra("10"), barra("10")]);
    assert_eq!(
        ler_sobrestar_da_barra(&instalacao(), &html, "10"),
        Err(ErroAcaoSobrestar::AcoesDeSobrestamentoRepetidasNaBarra)
    );
}
#[test]
fn nao_aceita_link_de_outro_processo() {
    let html = arvore(&[barra("11")]);
    assert_eq!(
        ler_sobrestar_da_barra(&instalacao(), &html, "10"),
        Err(ErroAcaoSobrestar::OutroProcesso)
    );
}
#[test]
fn retomar_so_pela_funcao_servida_na_barra_e_no_mesmo_no() {
    let html = arvore(&[retomar()]);
    let url = ler_retomar_da_barra(&instalacao(), &html, "10")
        .unwrap()
        .expect("funcao servida");
    assert_eq!(url.as_str(), absoluta(&link_da_tela_do_no("10")));
}
#[test]
fn retomar_repetido_recusado_antes_de_escrever() {
    let html = arvore(&[retomar(), retomar()]);
    assert_eq!(
        ler_retomar_da_barra(&instalacao(), &html, "10"),
        Err(ErroAcaoSobrestar::AcoesDeSobrestamentoRepetidasNaBarra)
    );
}
