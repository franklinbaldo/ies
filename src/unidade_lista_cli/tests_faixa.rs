//! Exportação da lista D e da caixa do replay: página observada, total e faixa.
use super::*;
use crate::caixa_detalhada::ler_paginas;
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
    let leitura = ler_paginas(&[html.to_owned()], Some("PPI")).unwrap();
    let caminho = destino(nome);
    assert_eq!(
        mostrar_caixa_detalhada(&leitura, "web", true, Some(&caminho)),
        ExitCode::SUCCESS
    );
    let texto = fs::read_to_string(&caminho).unwrap();
    fs::remove_file(&caminho).unwrap();
    texto
}

#[test]
fn detalhada_parcial_exporta_faixa_observada() {
    let texto = exportar_detalhada(
        &detalhada("(3 registros - 1 a 1):", "processoVisualizado"),
        "detalhada-faixa",
    );
    let valor: serde_json::Value = serde_json::from_str(&texto).unwrap();
    // A visão D é uma lista própria do SEI, na mesma forma das outras.
    assert_eq!(valor.as_array().map(Vec::len), Some(1));
    // D é modo de exibição, não categoria: a chave existe e vale null.
    assert_eq!(
        valor[0].as_object().unwrap().get("categoria"),
        Some(&serde_json::Value::Null)
    );
    assert_eq!(valor[0]["unidade"], "PPI");
    assert_eq!(valor[0]["origem"], "web");
    assert_eq!(valor[0]["total_declarado"], 3);
    assert_eq!(valor[0]["faixa_observada"]["inicio"], 1);
    assert_eq!(valor[0]["faixa_observada"]["fim"], 1);
    assert!(valor[0].get("visualizacao").is_none());
    assert!(valor[0].get("recebidos").is_none());
}

#[test]
fn exportacao_detalhada_e_reimportada_sem_categoria() {
    let texto = exportar_detalhada(
        &detalhada("(1 registros - 1 a 1):", "outraClasse"),
        "detalhada-reimportada",
    );
    let observada = crate::caixa_local::interpretar(&texto, Some("PPI")).unwrap();
    let crate::caixa_local::ListasLocais::SemCategoria(processos) = &observada.listas else {
        panic!("a exportação D não tem recebidos/gerados");
    };
    assert_eq!(processos.len(), 1);
    assert_eq!(processos[0].numero, "000042/2026");
    assert_eq!(processos[0].visualizado, None);
    assert_eq!(observada.unidade.as_deref(), Some("PPI"));
}

fn controle(legenda_recebidos: &str, ids: &[u32]) -> String {
    let linhas = ids.iter().fold(String::new(), |mut linhas, id| {
        use std::fmt::Write as _;
        write!(
            linhas,
            "<tr id='P{id}'><td><a class='processoVisualizado'>NUP-{id}</a></td></tr>"
        )
        .unwrap();
        linhas
    });
    format!(
        "<a id='lnkInfraUnidade'>PPI</a>\
         <input id='hdnTipoVisualizacao' value='R'>\
         <input id='hdnRecebidosHashCriterios' value='aaaa'>\
         <input id='hdnGeradosHashCriterios' value='bbbb'>\
         <table id='tblProcessosRecebidos'><caption>{legenda_recebidos}</caption>{linhas}</table>\
         <table id='tblProcessosGerados'><caption>0 registros:</caption></table>"
    )
}

#[test]
fn exportacao_do_replay_e_reimportada_pela_caixa_local() {
    let caminho = destino("caixa-replay");
    let html = controle("(2 registros - 1 a 2):", &[1, 2]);
    let codigo = mostrar_caixa(&[html], "web", Some("PPI"), true, true, Some(&caminho));
    assert_eq!(codigo, ExitCode::SUCCESS);
    let texto = fs::read_to_string(&caminho).unwrap();
    fs::remove_file(&caminho).unwrap();
    let observada = crate::caixa_local::interpretar(&texto, Some("PPI")).unwrap();
    assert_eq!(observada.caixa().recebidos.processos.len(), 2);
    assert_eq!(observada.caixa().recebidos.processos[1].numero, "NUP-2");
}

#[test]
fn replay_com_todas_as_paginas_nao_exporta_falso_total() {
    let caminho = destino("caixa-replay-parcial");
    let html = controle("(4 registros - 1 a 2):", &[1, 2]);
    let codigo = mostrar_caixa(&[html], "web", Some("PPI"), true, true, Some(&caminho));
    assert_eq!(codigo, ExitCode::from(1));
    assert!(!caminho.exists());
}
