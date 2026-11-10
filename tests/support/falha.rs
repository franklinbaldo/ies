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

/// [`falha`] com o invólucro `erro` e, na cadeia `Caused by:` que o
/// anyhow imprime, uma causa da erro `causa`. Só nomes são conferidos.
#[track_caller]
#[allow(dead_code)] // Nem todo alvo que inclui o suporte usa as causas.
pub fn falha_por(saida: &Output, erro: &str, causa: &str) {
    falha(saida, erro);
    let stderr = String::from_utf8_lossy(&saida.stderr);
    let causas: Vec<String> = stderr
        .split_once("\n\nCaused by:\n")
        .map_or("", |(_, resto)| resto.split("\n\n").next().unwrap_or(""))
        .lines()
        .map(|linha| {
            let linha = linha.trim_start();
            let linha = linha
                .split_once(": ")
                .filter(|(n, _)| n.chars().all(|c| c.is_ascii_digit()))
                .map_or(linha, |(_, resto)| resto);
            linha
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect()
        })
        .collect();
    assert!(
        causas.iter().any(|nome| nome == causa),
        "causa {causa} ausente de {causas:?}; stderr: {stderr}"
    );
}
