//! Superfície de sobrestar e retomar (#269): entradas inválidas e
//! confirmação exige sessão válida antes de qualquer operação remota.
use std::process::{Command, Output};

fn sem_sessao(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .env("IES_INDICE_FILE", "/nao/existe/ies.sqlite3")
        .output()
        .expect("IES deve iniciar")
}
fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("JSON na saída")
}
const NUP: &str = "00001.000001/2026-01";

#[test]
fn motivo_em_branco_e_erro_de_uso_sem_rede() {
    let saida = sem_sessao(&["processo", "sobrestar", NUP, "--motivo", " \t ", "--json"]);
    assert_eq!(saida.status.code(), Some(2));
}
#[test]
fn nup_invalido_prioritario_antes_da_sessao() {
    let saida = sem_sessao(&["processo", "sobrestar", " ", "--motivo", "teste", "--json"]);
    assert_eq!(saida.status.code(), Some(2));
}
#[test]
fn motivo_obrigatorio_no_clap() {
    let saida = sem_sessao(&["processo", "sobrestar", NUP, "--json"]);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(saida.stdout, b"");
}
#[test]
fn confirmar_exige_sessao_valida_e_nao_produz_sucesso_sem_configuracao() {
    for argumentos in [
        vec![
            "processo",
            "sobrestar",
            NUP,
            "--motivo",
            "teste",
            "--confirmar",
            "--json",
        ],
        vec!["processo", "retomar", NUP, "--confirmar", "--json"],
    ] {
        let saida = sem_sessao(&argumentos);
        assert_eq!(saida.status.code(), Some(1), "{argumentos:?}");
        let valor = json(&saida);
        assert_eq!(valor["retentavel"], false);
    }
}
