//! Exportação da lista D: página observada, total e faixa.
use super::*;
use crate::caixa_detalhada::ler_paginas;
use crate::tela_html::HtmlServido;
use std::fs;
use std::path::PathBuf;

fn destino(nome: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ies-{nome}-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn detalhada(legenda: &str, classe: &str) -> String {
    format!(
        "<html><a id='lnkInfraUnidade'>PPI</a>\
        <input id='hdnTipoVisualizacao' value='D'>\
        <table id='tblProcessosDetalhado'>\
        <caption>{legenda}</caption>\
        <tr><th>Processo</th><th>Tipo</th></tr>\
        <tr id='P42'><td><a class='{classe}'\
        href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=42'>\
        000042/2026</a></td><td>Ofício</td></tr></table></html>"
    )
}

fn exportar_detalhada(html: &str, nome: &str) -> String {
    let leitura = ler_paginas(&[HtmlServido::de_teste(html)], Some("PPI")).unwrap();
    let caminho = destino(nome);
    let indice = IndiceLocal::desligado();
    mostrar_caixa_detalhada(&leitura, "web", true, Some(&caminho), (None, &indice)).unwrap();
    let texto = fs::read_to_string(&caminho).unwrap();
    fs::remove_file(&caminho).unwrap();
    texto
}

#[test]
fn detalhada_parcial_exporta_faixa_observada() {
    let texto = exportar_detalhada(
        &detalhada("Processos (3 registros - 1 a 1):", "processoVisualizado"),
        "detalhada-faixa",
    );
    let valor: serde_json::Value = serde_json::from_str(&texto).unwrap();
    // A visão D é uma lista própria do SEI, na mesma forma das outras.
    assert_eq!(valor.as_array().map(Vec::len), Some(1));
    // D é modo de exibição, não categoria: a chave não existe.
    assert!(valor[0].get("categoria").is_none());
    assert_eq!(valor[0]["unidade"], "PPI");
    assert_eq!(valor[0]["origem"], "web");
    assert_eq!(valor[0]["total_declarado"], 3);
    assert_eq!(
        valor[0]["faixa_observada"],
        serde_json::json!({"inicio": 1, "fim": 1})
    );
    assert!(valor[0].get("visualizacao").is_none());
    assert!(valor[0].get("recebidos").is_none());
}

#[test]
fn exportacao_detalhada_e_lista_unica_sem_categoria() {
    let texto = exportar_detalhada(
        &detalhada("Processos (1 registros - 1 a 1):", "processoVisualizado"),
        "detalhada-lista-unica",
    );
    let valor: serde_json::Value = serde_json::from_str(&texto).unwrap();
    let processos = valor[0]["processos"].as_array().unwrap();
    assert_eq!(processos.len(), 1);
    assert_eq!(processos[0]["numero"], "000042/2026");
    assert_eq!(processos[0]["visualizado"], true);
    assert_eq!(processos[0]["sigiloso"], false);
    assert_eq!(valor[0]["unidade"], "PPI");
}

#[test]
fn link_sem_classe_de_estado_recusa_a_exportacao() {
    let html = HtmlServido::de_teste(&detalhada(
        "Processos (1 registros - 1 a 1):",
        "outraClasse",
    ));
    assert_eq!(
        ler_paginas(&[html], Some("PPI")).map(|l| l.processos.len()),
        Err(crate::caixa_detalhada::ErroDetalhada::EstadoDoLinkDoProcessoAusente)
    );
}
