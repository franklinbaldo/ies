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

/// Falha de comando convertido (#695): saída 1, stdout vazio e
/// `Error: <Variante>` no stderr.
fn assert_falha(saida: &Output, variante: &str) {
    assert_eq!(saida.status.code(), Some(1));
    assert!(
        saida.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&saida.stdout)
    );
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(erro.starts_with(&format!("Error: {variante}")), "{erro}");
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
    assert_falha(&saida, "ArquivoPrivadoAusenteOuInacessivel");
}

#[test]
fn mesma_identidade_recusa_antes_da_sessao() {
    let saida = executar(&["processo", "anexar", PRINCIPAL, "--a", PRINCIPAL, "--json"]);
    assert_falha(&saida, "AnexacaoAoProprioProcesso");
}

#[test]
fn previa_sem_sessao_falha_fechado() {
    let saida = executar(&["processo", "anexar", ANEXADO, "--a", PRINCIPAL, "--json"]);
    assert_falha(&saida, "ArquivoPrivadoAusenteOuInacessivel");
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
