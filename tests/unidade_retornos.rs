//! Contrato de superficie da CLI de retornos; sem acesso ao SEI.
use std::process::Command;

fn ies(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .output()
        .expect("binario ies compilado no teste")
}

#[test]
fn ajuda_da_unidade_publica_retornos_e_mes() {
    // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
    let help = ies(&["unidade", "retornos", "--help"]);
    assert!(help.status.success());
    let texto = String::from_utf8(help.stdout).unwrap();
    assert!(texto.contains("--mes"), "{texto}");
}

#[test]
fn mes_invalido_falha_antes_de_ler_sessao_ou_rede() {
    // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
    let resposta = ies(&["--json", "unidade", "retornos", "--mes", "2026-13"]);
    assert_eq!(resposta.status.code(), Some(1));
    let _json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
}
