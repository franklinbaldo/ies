//! Trava de escrita de `ies processo andamento` (#259).
//! Não abre login nem acessa o SEI: só confere a recusa antes da sessão.
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

#[test]
fn confirmar_sem_sessao_falha_antes_da_rede() {
    let saida = sem_sessao(&[
        "processo",
        "andamento",
        NUP,
        "--texto",
        "novo",
        "--confirmar",
        "--json",
    ]);
    assert_eq!(saida.status.code(), Some(1));
}

#[test]
fn texto_vazio_sai_2_antes_da_sessao() {
    let saida = sem_sessao(&["processo", "andamento", NUP, "--texto", "", "--json"]);
    assert_eq!(saida.status.code(), Some(2));
}
