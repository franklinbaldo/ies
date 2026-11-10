//! Entrada e trava de escrita de `ies processo prazo` (#272).
//! Não abre login nem acessa SEI: as escritas recusam antes da sessão.
use std::process::{Command, Output};

const NUP: &str = "00001.000001/2026-01";
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
#[test]
fn data_invalida_sai_2_antes_da_sessao() {
    let saida = sem_sessao(&["processo", "prazo", NUP, "--ate", "31/02/2026", "--json"]);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(json(&saida)["subcodigo"], "data_do_prazo_invalida");
}
#[test]
fn confirmar_sem_sessao_falha_antes_da_rede() {
    let saida = sem_sessao(&[
        "processo",
        "prazo",
        NUP,
        "--ate",
        "10/12/2026",
        "--confirmar",
        "--json",
    ]);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(
        json(&saida)["subcodigo"],
        "arquivo_privado_ausente_ou_inacessivel"
    );
}
#[test]
fn excluir_sem_sessao_falha_antes_da_rede() {
    let saida = sem_sessao(&["processo", "prazo", NUP, "--excluir", "--json"]);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(
        json(&saida)["subcodigo"],
        "arquivo_privado_ausente_ou_inacessivel"
    );
}
#[test]
fn data_e_dias_sao_exclusivos() {
    let saida = sem_sessao(&[
        "processo",
        "prazo",
        NUP,
        "--ate",
        "10/12/2026",
        "--dias",
        "5",
    ]);
    assert_eq!(saida.status.code(), Some(2));
}
#[test]
fn confirmar_sem_ate_dias_ou_excluir_e_uso_antes_da_rede() {
    let saida = sem_sessao(&["processo", "prazo", NUP, "--confirmar", "--json"]);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(
        json(&saida)["subcodigo"],
        "confirmar_sem_alteracao_de_prazo"
    );
}
