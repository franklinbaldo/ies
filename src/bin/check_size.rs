//! Verifica o limite de 20.000 bytes em todos os arquivos texto do repositorio.
//! O que tem NUL ou nao e UTF-8 e tratado como binario.
//! Arquivos ignorados pelo Git (como target/) nao entram no inventario.

use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

const LIMITE: usize = 20_000;

fn arquivo_textual(bytes: &[u8]) -> bool {
    !bytes.contains(&0) && std::str::from_utf8(bytes).is_ok()
}

fn viola_limite(bytes: &[u8]) -> bool {
    arquivo_textual(bytes) && bytes.len() > LIMITE
}

fn caminhos_git() -> Result<Vec<String>, String> {
    let saida = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .output()
        .map_err(|e| format!("git ls-files falhou: {e}"))?;
    if !saida.status.success() {
        return Err(String::from_utf8_lossy(&saida.stderr).into_owned());
    }
    saida
        .stdout
        .split(|&byte| byte == 0)
        .filter(|item| !item.is_empty())
        .map(|item| {
            String::from_utf8(item.to_vec())
                .map_err(|_| "Caminho Git nao e UTF-8; recusa conservadora".to_string())
        })
        .collect()
}

fn verificar() -> Result<(), String> {
    let mut excessos = Vec::new();
    let mut conferidos = 0usize;
    for nome in caminhos_git()? {
        let caminho = Path::new(&nome);
        if !caminho.is_file() {
            continue;
        }
        let conteudo = fs::read(caminho).map_err(|e| format!("{nome}: {e}"))?;
        if !arquivo_textual(&conteudo) {
            continue;
        }
        conferidos += 1;
        if viola_limite(&conteudo) {
            excessos.push(format!(
                "{nome}: {} bytes (limite: {LIMITE})",
                conteudo.len()
            ));
        }
    }
    if excessos.is_empty() {
        println!("Tamanho: {conferidos} arquivos texto verificados (maximo {LIMITE} bytes)");
        Ok(())
    } else {
        Err(format!(
            "Arquivos acima do limite:\n{}\nDivida por responsabilidade; nao minifique nem omita dados.",
            excessos.join("\n")
        ))
    }
}

fn main() -> ExitCode {
    match verificar() {
        Ok(()) => ExitCode::SUCCESS,
        Err(erro) => {
            eprintln!("{erro}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limite_inclusivo() {
        assert!(!viola_limite(&vec![b'a'; LIMITE]));
        assert!(viola_limite(&vec![b'a'; LIMITE + 1]));
    }

    #[test]
    fn detecta_binarios_e_texto_nao_utf8() {
        assert!(!arquivo_textual(b"abc\0def"));
        assert!(!arquivo_textual(&[0xff, 0xfe]));
        assert!(arquivo_textual("Olá, SEI".as_bytes()));
    }
}
