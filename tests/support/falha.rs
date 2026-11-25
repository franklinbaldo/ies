//! Contrato de falha da CLI (#695): saída 1, stdout vazio e o erro tipado
//! no stderr como `Error: <Erro>`. Só o nome da erro é conferido.
use std::process::Output;

#[track_caller]
pub fn falha(saida: &Output, erro: &str) {
    let stderr = String::from_utf8_lossy(&saida.stderr);
    assert_eq!(saida.status.code(), Some(1), "stderr: {stderr}");
    assert!(
        saida.stdout.is_empty(),
        "stdout deve ficar vazio na falha: {}",
        String::from_utf8_lossy(&saida.stdout)
    );
    let depois = stderr.strip_prefix("Error: ").unwrap_or_else(|| {
        panic!("stderr deve começar com `Error: `: {stderr}");
    });
    let nome: String = depois
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    assert_eq!(nome, erro, "stderr: {stderr}");
}
