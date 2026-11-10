//! CLI de Acompanhamento Especial (#254) — offline e sem escrever no SEI.
use std::process::{Command, Output};

const NUP: &str = "00001.000001/2026-01";

fn sem_sessao(argumentos: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(argumentos)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .env("IES_INDICE_FILE", "/nao/existe/ies.sqlite3")
        .output()
        .expect("binário IES executado")
}

fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("JSON válido")
}

#[test]
fn remocao_confirmada_bloqueada_antes_da_sessao_ou_rede() {
    // Caso de todos tests/test_processo_acompanhar_010.py @323eb01:
    // a confirmação exige fluxo próprio; nunca iniciar escrita sem esse fluxo.
    for acao in [vec!["--deixar"], vec!["--deixar", "--grupo", "G"]] {
        let mut args = vec!["processo", "acompanhar", NUP, "--confirmar", "--json"];
        args.extend(acao);
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(1), "{args:?}");
        let valor = json(&saida);
        assert_eq!(valor["retentavel"], false);
    }
}

#[test]
fn entrada_invalida_e_combinacoes_impossiveis() {
    let saida = sem_sessao(&["processo", "acompanhar", " ", "--json"]);
    assert_eq!(saida.status.code(), Some(2));

    let saida = sem_sessao(&[
        "processo",
        "acompanhar",
        NUP,
        "--deixar",
        "--observacao",
        "x",
        "--json",
    ]);
    assert_eq!(saida.status.code(), Some(2));
}

#[test]
fn previa_exige_sessao_privada_e_nunca_vira_stub() {
    let saida = sem_sessao(&["processo", "acompanhar", NUP, "--json"]);
    assert_eq!(saida.status.code(), Some(1));
    let valor = json(&saida);
    assert_eq!(valor["codigo"], "fonte_invalida");
    assert_ne!(valor["codigo"], "nao_implementado");
}

#[test]
fn inclusao_confirmada_nao_e_stub_e_exige_sessao() {
    // Caso de todos tests/test_processo_acompanhar_010.py @323eb01.
    let saida = sem_sessao(&[
        "processo",
        "acompanhar",
        NUP,
        "--grupo",
        "G",
        "--confirmar",
        "--json",
    ]);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(json(&saida)["codigo"], "fonte_invalida");
}
