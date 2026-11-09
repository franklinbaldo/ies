//! Leitura conferida das cassettes públicas do acervo. Só existe em
//! `cfg(test)`: o binário de produção não lê cassettes nem liga `zip` e
//! `sha1` (dependências de desenvolvimento). Os replays do acervo são os
//! testes `#[ignore]` `*::acervo*`, executados por `scripts/replay`.
use std::path::Path;
pub(crate) mod acervo;
mod conferencia;
pub(crate) use conferencia::ZipDoAcervo;
pub(crate) use conferencia::{conferir_cassette_ate, html_da_resposta};

fn campo<'a>(source: &'a str, field: &str) -> Option<&'a str> {
    let rest = source.strip_prefix("---\n")?;
    rest.split("\n---\n").next()?.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key == field).then_some(value.trim())
    })
}

// O acervo OKF exige sufixo `.zip` exatamente em minúsculas.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub(crate) fn archive_filename(card: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(card).map_err(|_| "card ausente")?;
    let url = campo(&text, "source_zip").ok_or("origem ausente")?;
    let base = url
        .strip_prefix("https://archive.org/download/todos-sei-cassettes-2026/")
        .ok_or("origem fora do acervo")?;
    if base.contains("..") || !base.ends_with(".zip") {
        return Err("origem insegura".into());
    }
    let name = base.rsplit('/').next().ok_or("nome ausente")?;
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
    {
        return Err("nome de ZIP inválido".into());
    }
    Ok(name.to_owned())
}

#[path = "cassette/anti_ies_ciclo.rs"]
mod anti_ies_ciclo;
#[cfg(test)]
mod anti_ies_retornos;
#[cfg(test)]
mod anti_ies_documento;

#[path = "cassette/tests.rs"]
mod tests;
