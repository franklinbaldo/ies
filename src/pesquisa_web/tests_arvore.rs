//! RED/GREEN das linhas da UnB 4.0.9: `a.arvore` com o mesmo `href` do
//! `a.protocoloNormal`, e `class` repetido idêntico neste. HTML sintético.
use super::*;
use crate::pesquisa_web::tests_fixtures::{linha, numero};

fn tabela(linhas: &str) -> String {
    format!("<html><table class=\"pesquisaResultado\">{linhas}</table></html>")
}

#[test]
fn linha_com_arvore_de_mesmo_href_e_class_repetido_identico_e_aceita() {
    let html = tabela(&format!("{}{}", linha(1), linha(2)));
    let lidos = ler_resultados(&html).unwrap();
    assert_eq!(lidos.len(), 2);
    assert_eq!(lidos[0].numero.as_str(), numero(1));
    assert_eq!(lidos[1].numero.as_str(), numero(2));
}

#[test]
fn arvore_com_href_diferente_do_link_do_processo_e_recusada() {
    let original = linha(1);
    let (antes, depois) = original.split_once("class=\"arvore\">").unwrap();
    let antes = antes.replacen("id_procedimento=1001", "id_procedimento=9999", 1);
    let html = tabela(&format!("{antes}class=\"arvore\">{depois}"));
    assert_eq!(
        ler_resultados(&html),
        Err(ErroResultadoPesquisa::LinkDaArvoreComHrefDiferenteDoLinkDoProcesso)
    );
}

#[test]
fn duas_arvores_na_mesma_linha_sao_recusadas() {
    let original = linha(1);
    let inicio = original.find("<a onclick").unwrap();
    let fim = original.find("</a>").unwrap() + "</a>".len();
    let arvore = &original[inicio..fim];
    let html = tabela(&original.replacen(arvore, &format!("{arvore}{arvore}"), 1));
    assert_eq!(
        ler_resultados(&html),
        Err(ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDaArvore)
    );
}

#[test]
fn class_repetido_com_valores_diferentes_continua_recusado() {
    let html = tabela(&linha(1).replacen(
        "title=\"Tipo\" class=\"protocoloNormal\"",
        "title=\"Tipo\" class=\"outra\"",
        1,
    ));
    assert_eq!(
        ler_resultados(&html),
        Err(ErroResultadoPesquisa::HtmlDePesquisaNaoInterpretado)
    );
}

#[test]
fn link_de_trabalho_fora_da_arvore_e_do_protocolo_continua_recusado() {
    // A tolerância é só para a árvore da própria linha: outro link de
    // trabalho na tabela ainda some da projeção e falha fechado.
    let extra = "<tr><td><a href=\"controlador.php?acao=procedimento_trabalhar&id_procedimento=5\">x</a></td></tr>";
    let html = tabela(&format!("{}{extra}", linha(1)));
    assert_eq!(
        ler_resultados(&html),
        Err(ErroResultadoPesquisa::LinkDeProcessoSemClasseReconhecida)
    );
}
