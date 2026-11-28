//! Contrato experimental de #876: somente prévia, sem sessão ou rede neste teste.
use std::process::Command;

#[test]
fn alterar_bloco_expoe_unidade_na_ajuda_sem_confirmacao() {
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["bloco", "alterar", "--help"])
        .output()
        .expect("CLI real");
    assert!(saida.status.success(), "subcomando de prévia não disponível");
    let ajuda = String::from_utf8(saida.stdout).expect("ajuda UTF-8");
    assert!(ajuda.contains("--unidade"));
    assert!(!ajuda.contains("--confirmar"));
}

#[test]
fn alterar_bloco_recusa_confirmar_na_analise_dos_argumentos() {
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["bloco", "alterar", "2195", "--unidade", "TESTE", "--confirmar"])
        .output()
        .expect("CLI real");
    assert_eq!(saida.status.code(), Some(2));
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(erro.contains("--confirmar"), "recusa deve ser do argumento");
}
