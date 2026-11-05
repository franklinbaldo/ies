//! Testes de integração de borda sem sessão real nem rede.
//! Contrato: contracts/cli/bloco-listar.md; oráculo Todos @323eb01.
use std::process::Command;

#[test]
fn bloco_listar_publica_tipo_e_pagina_no_help(){
    let saida=Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["bloco","listar","--help"]).output().unwrap();
    assert!(saida.status.success());
    let texto=String::from_utf8(saida.stdout).unwrap();
    assert!(texto.contains("--tipo"),"{texto}");
    assert!(texto.contains("--pagina"),"{texto}");
}
#[test]
fn pagina_zero_falha_antes_de_sessao_e_de_rede(){
    let saida=Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["bloco","listar","--pagina","0"]).output().unwrap();
    assert_eq!(saida.status.code(),Some(2));
}
