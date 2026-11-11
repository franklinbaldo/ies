//! [`falha`] mais uma causa da cadeia `Caused by:` (#695). Separado de
//! `falha.rs` para que cada binário de teste inclua só o que usa.
use crate::falha_cli::falha;
use std::process::Output;

/// [`falha`](crate::falha_cli::falha) com o invólucro `erro` e, na cadeia `Caused by:` que o
/// anyhow imprime, uma causa da erro `causa`. Só nomes são conferidos.
#[track_caller]
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
