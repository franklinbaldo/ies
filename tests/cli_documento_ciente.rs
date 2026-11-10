//! RED #271: superfície CLI de ciência de documento, caso Todos
//! `tests/test_documento_ciente_010.py` @323eb01. Sem sessão nem rede.
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

#[test]
fn documento_ciente_existe_no_contrato_cli() {
    let saida = sem_sessao(&["documento", "ciente", "--help"]);
    assert!(
        saida.status.success(),
        "{:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let texto = String::from_utf8_lossy(&saida.stdout);
    assert!(texto.contains("--confirmar"), "{texto}");
}

#[test]
fn numero_documento_invalido_falha_antes_de_sessao() {
    for confirmar in [false, true] {
        let mut args = vec!["documento", "ciente", " ", "--json"];
        if confirmar {
            args.push("--confirmar");
        }
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2));
        assert!(
            saida.stdout.is_empty(),
            "uso inválido é recusado pelo clap, sem JSON"
        );
    }
}

#[test]
fn comando_sem_sessao_falha_fechado() {
    let saida = sem_sessao(&["documento", "ciente", "0000101", "--confirmar", "--json"]);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(saida.stdout, b"");
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(
        erro.starts_with("Error: ArquivoPrivadoAusenteOuInacessivel"),
        "{erro}"
    );
}
