//! Barreira de CLI enquanto #257 tem apenas leitor e reconciliação offline.
//! Nenhuma escrita de comentários deve ficar acessível antes de existir um
//! fluxo de prévia, POST único, verificação por leitura e testes integrados.
use std::process::Command;

#[test]
fn bdd_257_01_expoe_leitura_comentarios_mas_nao_a_escrita() {
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["processo", "--help"])
        .output()
        .expect("CLI deve iniciar");
    assert!(saida.status.success());
    let ajuda = String::from_utf8(saida.stdout).expect("ajuda UTF-8");
    assert!(!ajuda.contains("comentar"));
    assert!(ajuda.contains("comentarios"), "leitura de comentários deve ser pública");
}
