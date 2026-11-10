//! A classificação do replay do Controle sobre cassettes SINTÉTICAS (telas
//! R e D de `visao_do_controle::tests`, hosts, ids e hashes inventados;
//! nenhum corpo gravado): leitores de produção, as três classes, a página
//! 2 planejada, a continuidade entre páginas e os casos negativos.
use super::super::tests::{TelaD, TelaR};
use super::continuidade::{Relacao, pares};
use super::{Leitor, Lista, PaginaLida, Registro, Replay, Resultado, leitores, relatorio};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

const RAIZ: &str = "https://sei.treinamento.unb.br";

fn passo(metodo: &str, status: u64, pathname: &str, corpo: &str) -> Value {
    json!({
        "request": {"method": metodo, "operation": "procedimento_controlar", "pathname": pathname},
        "response": {
            "status_code": status,
            "url": format!("{RAIZ}{pathname}?acao=procedimento_controlar"),
            "encoding": "utf-8",
            "headers": {
                "content-type": "text/html; charset=utf-8",
                "location": format!("{RAIZ}/sei/controlador.php?acao=procedimento_controlar&msg=x"),
            },
            "body_base64": STANDARD.encode(corpo.as_bytes()),
        }
    })
}

fn get(corpo: &str) -> Value {
    passo("GET", 200, "/sei/controlador.php", corpo)
}

fn cassette(passos: &[Value]) -> Replay {
    let json =
        json!({"schema_version": 1, "complete": true, "sei_root": RAIZ, "interactions": passos});
    leitores::replay("sintetica-unb", "unb", &json)
}

fn de(replay: &Replay, leitor: Leitor) -> Vec<&Registro> {
    replay
        .registros
        .iter()
        .filter(|r| r.leitor == leitor)
        .collect()
}

fn resultado(replay: &Replay, leitor: Leitor) -> Vec<&Resultado> {
    de(replay, leitor)
        .into_iter()
        .map(|r| &r.resultado)
        .collect()
}

fn r_paginada() -> TelaR {
    TelaR::nova((25, (1..=10).collect()), (2, vec![91, 92]))
}

#[test]
fn tela_r_paginada_publica_total_faixa_e_planeja_a_pagina_seguinte() {
    let replay = cassette(&[get(&r_paginada().html())]);
    let recebidos = de(&replay, Leitor::Recebidos)[0];
    assert_eq!(recebidos.resultado, Resultado::Lido);
    assert_eq!(recebidos.contagem("total_declarado"), Some(25));
    assert_eq!(recebidos.contagem("faixa_fim"), Some(10));
    assert!(recebidos.formas.contains(&"paginada".to_owned()));
    let pagina = de(&replay, Leitor::PaginaRecebidos)[0];
    assert_eq!(pagina.resultado, Resultado::Lido);
    assert_eq!(pagina.contagem("esperada_inicio"), Some(11));
    assert_eq!(pagina.contagem("esperada_fim"), Some(20));
    assert_eq!(pagina.contagem("paginas"), Some(3));
    assert!(pagina.formas.contains(&"lnkProximaPagina".to_owned()));
    let gerados = de(&replay, Leitor::PaginaGerados)[0];
    assert_eq!(gerados.contagem("paginas"), Some(1));
    assert!(gerados.formas.contains(&"sem_paginador".to_owned()));
    assert_eq!(
        resultado(&replay, Leitor::Detalhada),
        [&Resultado::NaoAplicavel(
            "tela_na_visao_r_o_comando_troca_para_d"
        )]
    );
    assert_eq!(replay.paginas.len(), 2);
}

#[test]
fn tela_d_le_colunas_e_planeja_a_pagina_seguinte() {
    let replay = cassette(&[get(&TelaD::nova(25, 0, 10, (1..=10).collect()).html())]);
    let d = de(&replay, Leitor::Detalhada)[0];
    assert_eq!(d.resultado, Resultado::Lido);
    assert_eq!(d.contagem("linhas"), Some(10));
    assert_eq!(d.contagem("celulas_com_marcador"), Some(1));
    assert!(d.formas.contains(&"coluna_marcadores".to_owned()));
    let p = de(&replay, Leitor::PaginaDetalhada)[0];
    assert_eq!(p.resultado, Resultado::Lido);
    assert_eq!(p.contagem("esperada_inicio"), Some(11));
    assert_eq!(p.contagem("paginas"), Some(3));
    assert_eq!(
        resultado(&replay, Leitor::Recebidos),
        [&Resultado::NaoAplicavel(
            "tela_na_visao_d_o_comando_troca_para_r"
        )]
    );
}

#[test]
fn redirecionamento_e_tela_de_login_sao_recusas_do_transporte() {
    let login = "<form id='frmLogin' action='login.php'><input id='txtUsuario'></form>";
    let mut sip = passo("GET", 200, "/sip/login.php", login);
    sip["request"]["operation"] = Value::Null;
    let replay = cassette(&[
        passo("GET", 302, "/sei/controlador.php", ""),
        sip,
        get(login),
    ]);
    assert_eq!(
        resultado(&replay, Leitor::TelaDeLogin),
        [&Resultado::Recusado("PaginaDeLoginRecebidaNoControle")]
    );
    assert!(
        de(&replay, Leitor::Transporte)[0]
            .formas
            .contains(&"location_acao_procedimento_controlar".to_owned())
    );
}

#[test]
fn unidade_da_tela_anterior_diferente_e_recusada() {
    let mut outra = r_paginada();
    outra.unidade = "OUTRA";
    let replay = cassette(&[
        get(&r_paginada().html()),
        get(&r_paginada().html()),
        get(&outra.html()),
    ]);
    assert!(
        de(&replay, Leitor::UnidadeSolicitada)
            .iter()
            .all(|r| !r.formas.contains(&"juntar_diverge_da_tela".to_owned()))
    );
}

#[test]
fn pagina_seguinte_gravada_continua_a_anterior() {
    let mut segunda = r_paginada();
    segunda.recebidos = (25, 11, (11..=20).collect());
    segunda.pagina_r = "1";
    let replay = cassette(&[get(&r_paginada().html()), get(&segunda.html())]);
    let relacoes: Vec<Relacao> = pares(&replay.paginas).iter().map(|p| p.relacao).collect();
    assert_eq!(relacoes, [Relacao::Continua, Relacao::MesmaPaginaIgual]);
    assert_eq!(
        de(&replay, Leitor::PaginaRecebidos)[1].contagem("esperada_inicio"),
        Some(21)
    );
}

fn pagina(faixa: (usize, usize), ids: &[&str]) -> PaginaLida {
    PaginaLida {
        card: "c".into(),
        instalacao: "unb".into(),
        passo: 0,
        lista: Lista::Recebidos,
        unidade: "U".into(),
        criterio: None,
        total: 30,
        faixa: Some(faixa),
        ids: ids.iter().map(|s| (*s).to_owned()).collect(),
    }
}

#[test]
fn lacuna_sobreposicao_e_id_repetido_sao_apontados() {
    let casos = [
        (pagina((11, 12), &["c", "d"]), Relacao::Lacuna),
        (pagina((2, 3), &["b", "c"]), Relacao::Sobreposta),
        (pagina((3, 4), &["b", "z"]), Relacao::ContinuaComIdRepetido),
        (pagina((1, 2), &["b", "a"]), Relacao::MesmaPaginaDiferente),
    ];
    for (b, esperada) in casos {
        let paginas = [pagina((1, 2), &["a", "b"]), b];
        assert_eq!(pares(&paginas)[0].relacao, esperada);
    }
    let mut outro_total = pagina((3, 4), &["c", "d"]);
    outro_total.total = 31;
    assert!(pares(&[pagina((1, 2), &["a", "b"]), outro_total]).is_empty());
}

#[test]
fn relatorio_agrega_sem_urls_nem_hashes() {
    let replay = cassette(&[get(&r_paginada().html())]);
    let texto = relatorio::markdown(1, &replay);
    assert!(texto.contains("| recebidos | unb | 1 | 1 | 0 | 0 |"));
    assert!(texto.contains("Páginas gravadas além da primeira (faixa começando depois de 1): 0."));
    assert!(!texto.contains("controlador.php"));
    assert!(!texto.contains("cccc"));
}
