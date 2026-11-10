//! Testes da CLI de anexação, sem sessão e sem requisições ao SEI.
//! Origem dos cenários: Todos/src/todos/anexacao.py @323eb01, adaptados.
use std::process::{Command, Output};

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

fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("JSON válido")
}

#[test]
fn comando_recusa_numero_vazio_antes_da_sessao() {
    let saida = executar(&["processo", "anexar", " ", "--a", PRINCIPAL, "--json"]);
    assert_eq!(saida.status.code(), Some(2));
    let valor = json(&saida);
    assert_eq!(valor["codigo"], "entrada_invalida");
    assert_eq!(valor["subcodigo"], "numero_processo_vazio");
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
    assert_eq!(saida.status.code(), Some(1));
    let valor = json(&saida);
    assert_eq!(valor["codigo"], "fonte_invalida");
    assert_eq!(valor["retentavel"], false);
}

#[test]
fn mesma_identidade_recusa_antes_da_sessao() {
    let saida = executar(&["processo", "anexar", PRINCIPAL, "--a", PRINCIPAL, "--json"]);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(json(&saida)["subcodigo"], "anexacao_ao_mesmo_processo");
}

#[test]
fn previa_sem_sessao_falha_fechado() {
    let saida = executar(&["processo", "anexar", ANEXADO, "--a", PRINCIPAL, "--json"]);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(json(&saida)["codigo"], "fonte_invalida");
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
