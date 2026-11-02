//! RED/GREEN da caixa Web paginada: página observada, total e faixa.
use super::*;
use std::fs;

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
         <a id='lnkRecebidosProximaPaginaSuperior'>»</a>\
         <table id='tblProcessosRecebidos'><caption>{legenda_recebidos}</caption>{linhas}</table>\
         <table id='tblProcessosGerados'><caption>0 registros:</caption></table>"
    )
}

fn exportar(html: &str) -> (ExitCode, Option<serde_json::Value>) {
    let caminho = destino("caixa-web-faixa");
    let codigo = mostrar_caixa_web(&[html.to_owned()], "web", Some("PPI"), true, Some(&caminho));
    let valor = fs::read(&caminho)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap());
    let _ = fs::remove_file(&caminho);
    (codigo, valor)
}

#[test]
fn caixa_paginada_devolve_a_pagina_observada_com_total_e_faixa() {
    let (codigo, valor) = exportar(&controle(
        "Processos recebidos (4 registros - 1 a 2):",
        &[1, 2],
    ));
    assert_eq!(codigo, ExitCode::SUCCESS);
    let valor = valor.unwrap();
    assert_eq!(valor["recebidos"]["total_declarado"], 4);
    assert_eq!(valor["recebidos"]["faixa_observada"]["inicio"], 1);
    assert_eq!(valor["recebidos"]["faixa_observada"]["fim"], 2);
    assert_eq!(valor["recebidos"]["processos"].as_array().unwrap().len(), 2);
    assert!(valor["gerados"]["faixa_observada"].is_null());
    assert!(valor["recebidos"].get("completa").is_none());
    assert!(valor.get("completude").is_none());
}

#[test]
fn pagina_lembrada_pelo_servidor_aparece_com_a_propria_faixa() {
    let (codigo, valor) = exportar(&controle("(4 registros - 3 a 4):", &[3, 4]));
    assert_eq!(codigo, ExitCode::SUCCESS);
    let valor = valor.unwrap();
    assert_eq!(valor["recebidos"]["faixa_observada"]["inicio"], 3);
    assert_eq!(valor["recebidos"]["faixa_observada"]["fim"], 4);
}

#[test]
fn legenda_sem_faixa_com_total_maior_que_as_linhas_continua_recusada() {
    let sem_paginador = controle("5 registros:", &[1, 2])
        .replace("<a id='lnkRecebidosProximaPaginaSuperior'>»</a>", "");
    let (codigo, valor) = exportar(&sem_paginador);
    assert_eq!(codigo, ExitCode::from(1));
    assert!(valor.is_none());
}

#[test]
fn paginador_da_tabela_sem_intervalo_na_legenda_continua_recusado() {
    let (codigo, valor) = exportar(&controle("2 registros:", &[1, 2]));
    assert_eq!(codigo, ExitCode::from(1));
    assert!(valor.is_none());
}

#[test]
fn detalhada_parcial_exporta_faixa_observada() {
    let html = "<html><a id='lnkInfraUnidade'>PPI</a>\
        <input id='hdnTipoVisualizacao' value='D'>\
        <table id='tblProcessosDetalhado'>\
        <caption>(3 registros - 1 a 1):</caption>\
        <tr><th>Processo</th><th>Tipo</th></tr>\
        <tr id='P42'><td><a class='processoVisualizado'\
        href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=42'>\
        000042/2026</a></td><td>Ofício</td></tr></table></html>";
    let caminho = destino("detalhada-faixa");
    assert_eq!(
        mostrar_caixa_detalhada(&[html.to_owned()], "web", Some("PPI"), true, Some(&caminho)),
        ExitCode::SUCCESS
    );
    let valor: serde_json::Value = serde_json::from_slice(&fs::read(&caminho).unwrap()).unwrap();
    fs::remove_file(&caminho).unwrap();
    assert_eq!(valor["total_declarado"], 3);
    assert_eq!(valor["faixa_observada"]["inicio"], 1);
    assert_eq!(valor["faixa_observada"]["fim"], 1);
}
