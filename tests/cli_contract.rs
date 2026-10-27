use std::process::Command;

fn ies(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .output()
        .expect("IES deve iniciar")
}

#[test]
fn ajuda_raiz() {
    let output = ies(&["--help"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("processo"));
}

#[test]
fn substantivo_sozinho_mostra_ajuda() {
    let output = ies(&["processo"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("enviar"));
}

#[test]
fn comando_sem_execucao_retorna_falha_estruturada() {
    let output = ies(&["documento", "77782513", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["codigo"], "nao_implementado");
}

#[test]
fn uso_invalido_falha_antes_da_execucao() {
    let output = ies(&["processo", "enviar", "123"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn escrita_reconhecida_nao_executa_sei() {
    let output = ies(&["processo", "enviar", "123", "--para", "PGE-PPI", "--confirmar", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["codigo"], "nao_implementado");
}
