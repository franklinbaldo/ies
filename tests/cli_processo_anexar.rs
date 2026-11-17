//! Testes da CLI de anexação, sem sessão e sem requisições ao SEI.
//! Origem dos cenários: Todos/src/todos/anexacao.py @323eb01, adaptados.
use std::process::{Command, Output};

#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::falha;
#[path = "support/falha_por.rs"]
mod falha_por_cli;
use falha_por_cli::falha_por;

const PRINCIPAL: &str = "00001.000001/2026-01";
const ANEXADO: &str = "00001.000002/2026-01";

fn executar(args: &[&str]) -> Output {
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
fn comando_recusa_numero_vazio_antes_da_sessao() {
    let saida = executar(&["processo", "anexar", " ", "--a", PRINCIPAL, "--json"]);
    assert_eq!(saida.status.code(), Some(2));
    assert!(
        saida.stdout.is_empty(),
        "uso inválido é recusado pelo clap, sem JSON"
    );
}

#[test]
fn confirmar_recusa_sem_sessao_ativa() {
    let saida = executar(&[
        "processo",
        "anexar",
        ANEXADO,
        "--a",
        PRINCIPAL,
        "--confirmar",
        "--json",
    ]);
    falha_por(&saida, "UrlDoControleIlegivel", "Ausente");
}

#[test]
fn mesma_identidade_recusa_antes_da_sessao() {
    let saida = executar(&["processo", "anexar", PRINCIPAL, "--a", PRINCIPAL, "--json"]);
    falha(&saida, "AnexacaoAoProprioProcesso");
}

#[test]
fn previa_sem_sessao_falha_fechado() {
    let saida = executar(&["processo", "anexar", ANEXADO, "--a", PRINCIPAL, "--json"]);
    falha_por(&saida, "UrlDoControleIlegivel", "Ausente");
}

#[test]
fn ausente_a_ou_flag_desconhecida_e_uso_invalido() {
    for args in [
        vec!["processo", "anexar", ANEXADO],
        vec!["processo", "anexar", ANEXADO, "--a", PRINCIPAL, "--retry"],
    ] {
        let saida = executar(&args);
        assert_eq!(saida.status.code(), Some(2), "{args:?}");
    }
}
