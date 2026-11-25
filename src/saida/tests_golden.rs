//! O golden da caixa resumida (`tests/fixtures/caixa-v2.json`) e as recusas
//! do leitor, sobre HTML SINTÉTICO. Portado de `tests/replay_unidade.rs`, que
//! conferia o mesmo pelo antigo `ies replay`: agora o teste chama direto o
//! leitor (`paginas::juntar`) e a projeção (`saida::json`).
use super::json;
use crate::paginas::juntar;
use crate::tela_html::HtmlServido;

const HTML: &str = r#"<html><body>
<a id="lnkInfraUnidade">TESTE</a>
<table id="tblProcessosRecebidos"><caption>3 registros:</caption>
<tr><th>Número</th></tr>
<tr id="P42"><td><a class="processoVisualizado" href="controlador.php?acao=procedimento_trabalhar&amp;infra_hash=SEGREDO">23106.000001/2026-01</a></td></tr>
<tr id="P43"><td><a class="processoNaoVisualizado" href="controlador.php?acao=procedimento_trabalhar">23106.000002/2026-02</a></td></tr>
<tr id="P44"><td><a class="processoVisualizadoSigiloso" href="controlador.php?acao=procedimento_trabalhar">001234500000015</a></td></tr>
</table>
<table id="tblProcessosGerados"><caption>0 registros:</caption></table>
</body></html>"#;

#[test]
fn contrato_json_v2_da_caixa_resumida_e_deterministico() {
    let leitura = juntar(&[HtmlServido::de_teste(HTML)], Some("TESTE")).unwrap();
    let texto = format!(
        "{}\n",
        serde_json::to_string_pretty(&json(&leitura, "unb").unwrap()).unwrap()
    );
    assert_eq!(texto, include_str!("../../tests/fixtures/caixa-v2.json"));
    assert!(!texto.contains("infra_hash") && !texto.contains("SEGREDO"));
}

#[test]
fn caixa_resumida_sao_duas_listas_sem_url_assinada_nem_completude() {
    let leitura = juntar(&[HtmlServido::de_teste(HTML)], None).unwrap();
    let valor = json(&leitura, "unb").unwrap();
    assert_eq!(valor.as_array().map(Vec::len), Some(2));
    assert_eq!(valor[0]["categoria"], "recebidos");
    assert_eq!(valor[1]["categoria"], "gerados");
    assert!(valor[0].get("completa").is_none() && valor[1].get("completa").is_none());
    assert_eq!(valor[0]["total_declarado"], 3);
    assert_eq!(valor[1]["total_declarado"], 0);
    assert_eq!(valor[0]["processos"][0]["id"], "42");
    assert_eq!(valor[0]["processos"][1]["visualizado"], false);
    assert_eq!(valor[0]["processos"][2]["sigiloso"], true);
    assert_eq!(valor[0]["processos"][2]["numero"], "001234500000015");
}

#[test]
fn unidade_solicitada_precisa_ser_confirmada_pela_tela() {
    let _erro = juntar(&[HtmlServido::de_teste(HTML)], Some("OUTRA"))
        .err()
        .unwrap();
}

#[test]
fn linha_sem_link_do_processo_tem_o_id_sem_vazar_html() {
    let html = HtmlServido::de_teste(
        "<table id='tblProcessosRecebidos'><caption>1 registros:</caption>\
        <tr id='P8'><td><a class='outraClasse' href='?infra_hash=SEGREDO'>123</a></td></tr></table>\
        <table id='tblProcessosGerados'><caption>0 registros:</caption></table>",
    );
    let erro = juntar(&[html], None).err().unwrap();
    assert_eq!(erro.id(), Some("8"));
    assert!(!erro.to_string().contains("SEGREDO"));
}
