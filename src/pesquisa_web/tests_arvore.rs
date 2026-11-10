//! RED/GREEN das linhas da `UnB` 4.0.9 (`a.arvore` com o mesmo `href` do
//! `a.protocoloNormal`, `class` repetido idêntico neste) e das linhas de
//! Goiás com acerto em documento. HTML sintético.
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
        Err(ErroResultadoPesquisa::Atributo(
            crate::html_atributos::ErroAtributoHtml::AtributoRepetidoNaTag { atributo: "class" }
        ))
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

/// Linha cujo acerto é um documento do processo, na forma de Goiás
/// (`processo-busca-go`, passos 2 e 4): o link do processo leva
/// `id_documento`, e dois `a.protocoloNormal` (nome e número SEI do
/// documento) abrem `documento_visualizar` com o mesmo `id_documento`.
fn linha_com_documento(posicao: u64, documento: &str, no_documento: &str) -> String {
    let original = linha(posicao);
    let id = 1000 + posicao;
    let original = original.replace(
        &format!("id_procedimento={id}&"),
        &format!("id_procedimento={id}&id_documento={documento}&"),
    );
    let link = format!(
        "<a onclick=\"x(this);\" target=\"_blank\" class=\"protocoloNormal\" \
         href=\"controlador.php?acao=documento_visualizar&acao_origem=protocolo_pesquisar&id_documento={no_documento}&infra_sistema=1&infra_hash={}\" title=\"Despacho\">",
        "e".repeat(64)
    );
    original.replacen(
            "</a></td><td class=\"pesquisaTituloDireita\"></td>",
            &format!("</a> ({link}Despacho</a>)</td><td class=\"pesquisaTituloDireita\">{link}0000101</a></td>"),
            1,
        )
}

#[test]
fn linha_com_acerto_em_documento_publica_so_o_processo() {
    let com_documento = linha_com_documento(2, "77", "77");
    assert_eq!(com_documento.matches("documento_visualizar").count(), 2);
    let html = tabela(&format!("{}{com_documento}{}", linha(1), linha(3)));
    let lidos = ler_resultados(&html).unwrap();
    let numeros: Vec<&str> = lidos.iter().map(|r| r.numero.as_str()).collect();
    assert_eq!(numeros, [numero(1), numero(2), numero(3)]);
}

#[test]
fn documento_da_linha_diferente_do_link_do_processo_e_recusado() {
    let html = tabela(&linha_com_documento(1, "77", "78"));
    assert_eq!(
        ler_resultados(&html),
        Err(ErroResultadoPesquisa::LinkDoDocumentoDaLinhaDiferenteDoProcesso)
    );
}

#[test]
fn documento_na_linha_sem_id_documento_no_processo_e_recusado() {
    let html = tabela(&linha_com_documento(1, "77", "77").replace(
        "&id_documento=77&infra_sistema=100000100",
        "&infra_sistema=100000100",
    ));
    assert_eq!(
        ler_resultados(&html),
        Err(ErroResultadoPesquisa::LinkDoDocumentoDaLinhaDiferenteDoProcesso)
    );
}

#[test]
fn processo_visitado_com_class_repetido_contendo_a_classe_e_lido() {
    // PEN4 5.0.4 ao vivo (04/11/2026) e PEN3 na cassette: o link do processo
    // já visitado vem com `class="protocoloNormal processoVisitado" …
    // class="protocoloNormal"`.
    let visitado = linha(1).replacen(
        "target=\"_blank\" class=\"protocoloNormal\" ",
        "target=\"_blank\" class=\"protocoloNormal processoVisitado\" ",
        1,
    );
    assert_ne!(visitado, linha(1));
    let lidos = ler_resultados(&tabela(&format!("{visitado}{}", linha(2)))).unwrap();
    assert_eq!(lidos.len(), 2);
    assert_eq!(lidos[0].numero.as_str(), numero(1));
    let sem_a_classe = linha(1).replacen(
        "target=\"_blank\" class=\"protocoloNormal\" ",
        "target=\"_blank\" class=\"processoVisitado\" ",
        1,
    );
    assert_eq!(
        ler_resultados(&tabela(&sem_a_classe)),
        Err(ErroResultadoPesquisa::Atributo(
            crate::html_atributos::ErroAtributoHtml::AtributoRepetidoNaTag { atributo: "class" }
        ))
    );
}
