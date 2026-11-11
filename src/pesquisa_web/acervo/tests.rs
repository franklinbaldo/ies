//! A classificação do replay sobre cassettes SINTÉTICAS (telas de
//! `tests_fixtures`: hosts, hashes, ids e números inventados; nenhum corpo
//! gravado): a sequência Controle → tela → envio pelas peças de produção.
use super::{Leitor, Registro, Resultado, leitores, relatorio};
use crate::pesquisa_web::tests_fixtures::{
    controle, formulario, href_do_menu, localizacao, menu, resultado, resultado_numa_pagina,
    sem_resultado, tela_de_pesquisa, url_controle, url_da_tela, url_do_envio, validacao,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

const RAIZ: &str = "https://sei.exemplo.gov.br";

fn passo(operacao: &str, metodo: &str, status: u64, url: &str, corpo: &str) -> Value {
    json!({
        "request": {"method": metodo, "operation": operacao, "pathname": "/sei/controlador.php"},
        "response": {
            "status_code": status,
            "url": url,
            "encoding": "utf-8",
            "headers": {"content-type": "text/html; charset=utf-8"},
            "body_base64": STANDARD.encode(corpo.as_bytes()),
        }
    })
}

fn do_controle() -> Value {
    let html = controle(&menu(&href_do_menu()));
    passo("procedimento_controlar", "GET", 200, &url_controle(), &html)
}

fn da_tela() -> Value {
    passo(
        "protocolo_pesquisar",
        "GET",
        200,
        &url_da_tela(),
        &tela_de_pesquisa(),
    )
}

fn do_envio(html: &str) -> Value {
    passo("protocolo_pesquisar", "POST", 200, &url_do_envio(), html)
}

fn redirecionado(local: &str) -> Value {
    let mut p = passo("protocolo_pesquisar", "POST", 302, &url_do_envio(), "");
    p["response"]["headers"]["Location"] = Value::String(local.to_owned());
    p
}

fn cassette(passos: &[Value]) -> Vec<Registro> {
    let json =
        json!({"schema_version": 1, "complete": true, "sei_root": RAIZ, "interactions": passos});
    leitores::replay("sintetica", "unb", &json)
}

fn de(registros: &[Registro], leitor: Leitor) -> Vec<(Resultado, Option<&'static str>)> {
    registros
        .iter()
        .filter(|r| r.leitor == leitor)
        .map(|r| (r.resultado.clone(), r.forma))
        .collect()
}

#[test]
fn controle_tela_e_302_sao_lidos_pelas_pecas_do_comando() {
    let r = cassette(&[do_controle(), da_tela(), redirecionado(&localizacao("77"))]);
    assert_eq!(de(&r, Leitor::Controle), [(Resultado::Lido, None)]);
    assert_eq!(de(&r, Leitor::Tela), [(Resultado::Lido, None)]);
    assert_eq!(
        de(&r, Leitor::Envio),
        [(Resultado::Lido, Some("processo_unico"))]
    );
    assert_eq!(de(&r, Leitor::PlanoDaPagina).len(), 0);
}

#[test]
fn lista_paginada_e_lida_e_a_pagina_2_e_planejada() {
    let r = cassette(&[
        do_controle(),
        da_tela(),
        do_envio(&resultado("", 1, 10, "1.234")),
    ]);
    assert_eq!(de(&r, Leitor::Envio), [(Resultado::Lido, Some("lista"))]);
    let lista = r.iter().find(|x| x.leitor == Leitor::Envio).unwrap();
    assert_eq!(lista.total_declarado, Some(1234));
    assert_eq!(
        de(&r, Leitor::PlanoDaPagina),
        [(Resultado::Lido, Some("pagina_2_planejada"))]
    );
}

#[test]
fn lista_com_linha_ausente_registra_a_faixa_publicada() {
    let html = crate::pesquisa_web::tests_fixtures::conteudo_da_lista(1, 9, "40", 10)
        .replace("Exibindo 1 - 9 de", "Exibindo 1 - 10 de");
    let html = format!(
        "<html><body><a id=\"lnkInfraUnidade\">UNIDADE TESTE</a>{}</body></html>",
        formulario("", true, &html)
    );
    let r = cassette(&[do_controle(), da_tela(), do_envio(&html)]);
    assert_eq!(
        de(&r, Leitor::Envio),
        [(Resultado::Lido, Some("lista_com_linhas_ausentes"))]
    );
    assert!(relatorio::markdown(1, &r).contains("40, faixa 1–10, linhas ausentes 1"));
}

#[test]
fn lista_numa_pagina_tem_plano_de_uma_pagina_so() {
    let html = resultado_numa_pagina("", 2, "2 resultados");
    let r = cassette(&[do_controle(), da_tela(), do_envio(&html)]);
    assert_eq!(de(&r, Leitor::Envio), [(Resultado::Lido, Some("lista"))]);
    assert_eq!(
        de(&r, Leitor::PlanoDaPagina),
        [(Resultado::Lido, Some("uma_pagina_so"))]
    );
}

#[test]
fn sem_resultado_e_lido_como_resposta_observada() {
    let r = cassette(&[do_controle(), da_tela(), do_envio(&sem_resultado(""))]);
    assert_eq!(
        de(&r, Leitor::Envio),
        [(Resultado::Lido, Some("sem_resultado"))]
    );
    assert_eq!(de(&r, Leitor::PlanoDaPagina).len(), 0);
}

#[test]
fn validacao_da_infra_e_recusa_tipada() {
    let _r = cassette(&[do_controle(), da_tela(), do_envio(&validacao("Informe"))]);
}

#[test]
fn redirecionamento_para_outra_acao_e_recusado() {
    let outra = localizacao("77").replace("procedimento_trabalhar", "documento_visualizar");
    let _r = cassette(&[do_controle(), da_tela(), redirecionado(&outra)]);
}

#[test]
fn controle_sem_menu_da_pesquisa_e_recusado() {
    let html = controle("");
    let sem_menu = passo("procedimento_controlar", "GET", 200, &url_controle(), &html);
    let r = cassette(&[sem_menu, da_tela()]);
    // Sem Controle lido, a unidade que a tela confere não foi gravada.
    assert_eq!(
        de(&r, Leitor::Tela),
        [(
            Resultado::NaoAplicavel("unidade_do_controle_nao_gravada_antes_da_tela".to_owned()),
            None
        )]
    );
}

#[test]
fn envio_sem_tela_derivada_antes_nao_se_aplica() {
    let r = cassette(&[do_controle(), do_envio(&sem_resultado(""))]);
    assert_eq!(
        de(&r, Leitor::Envio),
        [(
            Resultado::NaoAplicavel("tela_da_pesquisa_nao_derivada_antes_do_envio".to_owned()),
            None
        )]
    );
}

#[test]
fn lista_da_pesquisa_em_documentos_nao_e_lida_como_processos() {
    let lista_d = crate::pesquisa_web::tests_fixtures::conteudo_da_lista(1, 10, "40", 10);
    let html = format!(
        "<html><body><a id=\"lnkInfraUnidade\">UNIDADE TESTE</a>{}</body></html>",
        formulario("", false, &lista_d)
    );
    let r = cassette(&[do_controle(), da_tela(), do_envio(&html)]);
    assert_eq!(
        de(&r, Leitor::Envio),
        [(
            Resultado::NaoAplicavel("lista_da_pesquisa_em_documentos".to_owned()),
            None
        )]
    );
}

#[test]
fn sem_resultado_da_pesquisa_em_documentos_e_lido_pelo_aviso() {
    let html = sem_resultado("").replace(
        "id=\"optProcessos\" name=\"rdoPesquisarEm\" value=\"P\" checked=\"checked\"",
        "id=\"optProcessos\" name=\"rdoPesquisarEm\" value=\"P\"",
    );
    let html = html.replace(
        "id=\"optDocumentos\" name=\"rdoPesquisarEm\" value=\"D\"",
        "id=\"optDocumentos\" name=\"rdoPesquisarEm\" value=\"D\" checked=\"checked\"",
    );
    let r = cassette(&[do_controle(), da_tela(), do_envio(&html)]);
    assert_eq!(
        de(&r, Leitor::Envio),
        [(Resultado::Lido, Some("sem_resultado_em_documentos"))]
    );
}

#[test]
fn post_da_tela_de_resultado_nao_tem_corpo_gravado() {
    let r = cassette(&[
        do_controle(),
        da_tela(),
        do_envio(&resultado("", 1, 10, "40")),
        do_envio(&resultado("", 11, 20, "40")),
    ]);
    assert_eq!(de(&r, Leitor::Envio).len(), 1);
    assert_eq!(
        de(&r, Leitor::PostDaTelaDeResultado),
        [(
            Resultado::NaoAplicavel("post_da_tela_de_resultado_sem_corpo_gravado".to_owned()),
            None
        )]
    );
}

#[test]
fn relatorio_tem_a_tabela_e_nao_publica_url_nem_hash() {
    let r = cassette(&[do_controle(), da_tela(), redirecionado(&localizacao("77"))]);
    let md = relatorio::markdown(1, &r);
    assert!(md.contains("| resposta do envio | unb | 1 | 1 |"), "{md}");
    assert!(md.contains("processo_unico"), "{md}");
    assert!(!md.contains("infra_hash") && !md.contains("controlador.php"));
}

#[test]
fn pesquisa_rapida_classifica_o_302_sem_usar_o_corpo() {
    let url = url_do_envio().replace("protocolo_pesquisar", "protocolo_pesquisa_rapida");
    let mut resposta = passo("protocolo_pesquisa_rapida", "POST", 302, &url, "");
    let local = localizacao("77")
        .replace("protocolo_pesquisar", "protocolo_pesquisa_rapida")
        .replace("id_procedimento=", "id_protocolo=");
    resposta["response"]["headers"]["Location"] = Value::String(local);
    let sem_location = passo("protocolo_pesquisa_rapida", "POST", 302, &url, "");
    let json = json!({
        "sei_root": RAIZ,
        "interactions": [resposta, sem_location]
    });
    let _registros = super::rapida::replay("sintetica", "unb", &json);
}
