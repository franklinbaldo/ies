//! Gate de integridade das relações OKF entre operações e cassettes.
//! Não consulta rede nem usa cassettes brutas: valida o grafo de arquivos locais.
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn frontmatter<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let mut lines = source.lines();
    if lines.next()? != "---" {
        return None;
    }
    for line in lines {
        if line == "---" {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name == key
        {
            return Some(value.trim());
        }
    }
    None
}

fn md_links(line: &str) -> Vec<&str> {
    let mut rest = line;
    let mut links = Vec::new();
    while let Some((_, tail)) = rest.split_once("](") {
        if let Some((target, after)) = tail.split_once(')') {
            links.push(target);
            rest = after;
        } else {
            break;
        }
    }
    links
}

fn files_recursive(path: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            files_recursive(&path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "md") {
            output.push(path);
        }
    }
    Ok(())
}

// Auditoria sequencial: preservar a leitura conjunta dos vínculos OKF.
#[allow(clippy::too_many_lines)]
fn verify() -> Result<(usize, usize), String> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("raiz do workspace não encontrada")?;
    let root = fs::canonicalize(workspace).map_err(|e| e.to_string())?;
    let folder = root.join("contracts/evidencias");
    let mut files = Vec::new();
    files_recursive(&folder, &mut files)?;
    let mut operations = HashSet::new();
    let mut details = 0usize;
    let mut missing = Vec::new();
    let mut links_count = 0usize;
    let index = fs::read_to_string(folder.join("README.md")).map_err(|e| e.to_string())?;

    for file in &files {
        let content = fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let kind = frontmatter(&content, "type").unwrap_or("");
        if file == &folder.join("README.md") {
            if kind != "Guide" {
                missing.push("README do catálogo não é OKF Guide".to_owned());
            }
        } else if kind == "OperationEvidence" {
            let operation = frontmatter(&content, "operation").unwrap_or("");
            let name = file.file_stem().unwrap_or_default().to_string_lossy();
            if operation.replace('.', "-") != name || !operations.insert(operation.to_owned()) {
                missing.push(format!(
                    "Identidade de operação inválida: {}",
                    file.display()
                ));
            }
            if frontmatter(&content, "source") != Some("todos-demonstration-matrix")
                || frontmatter(&content, "source_commit").is_none()
            {
                missing.push(format!("Proveniência ausente: {}", file.display()));
            }
            if !index.contains(&format!("]({name}.md)")) {
                missing.push(format!("Operação sem índice: {operation}"));
            }
            let unresolved =
                frontmatter(&content, "unresolved_steps").and_then(|v| v.parse::<usize>().ok());
            if unresolved.is_none() {
                missing.push(format!("Contagem de lacunas ausente: {}", file.display()));
            }
            if unresolved.is_some_and(|n| n > 0)
                && !content.contains("PENDENTE_DE_CASSETTE")
                && !content.contains("detalhes/")
            {
                missing.push(format!("Lacunas não explicitadas: {}", file.display()));
            }
        } else if kind == "OperationEvidenceDetail" {
            details += 1;
            if frontmatter(&content, "operation").is_none()
                || frontmatter(&content, "installation").is_none()
            {
                missing.push(format!("Detalhe sem escopo: {}", file.display()));
            }
        } else {
            missing.push(format!("Tipo OKF desconhecido: {}", file.display()));
        }
        for line in content.lines() {
            for link in md_links(line) {
                if link.starts_with("http://") || link.starts_with("https://") {
                    continue;
                }
                if !link.ends_with(".md") {
                    continue;
                }
                links_count += 1;
                let candidate = file.parent().unwrap_or(&root).join(link);
                let target = match fs::canonicalize(&candidate) {
                    Ok(target) if target.starts_with(&root) => target,
                    _ => {
                        missing.push(format!("Link quebrado: {} → {link}", file.display()));
                        continue;
                    }
                };
                if link.contains("/cassettes/") {
                    let raw = fs::read_to_string(&target).map_err(|e| e.to_string())?;
                    let inst = target
                        .strip_prefix(root.join("cassettes"))
                        .ok()
                        .and_then(|p| p.components().next())
                        .map(|s| s.as_os_str().to_string_lossy().into_owned());
                    if frontmatter(&raw, "type") != Some("Cassette")
                        || frontmatter(&raw, "installation") != inst.as_deref()
                        || frontmatter(&raw, "source_zip").is_none()
                        || frontmatter(&raw, "cassette_sha256_lf").is_none()
                    {
                        missing.push(format!("Cassette sem proveniência: {}", target.display()));
                    }
                }
            }
        }
    }
    if operations.len() < 129 {
        missing.push(format!(
            "Apenas {} operações (esperadas: 129)",
            operations.len()
        ));
    }
    if details == 0 {
        missing.push("Detalhes de operações extensas não encontrados".to_owned());
    }
    if !missing.is_empty() {
        return Err(missing.join("\n"));
    }
    Ok((operations.len(), links_count))
}

fn main() -> ExitCode {
    match verify() {
        Ok((operations, links)) => {
            println!("Evidências OKF: {operations} operações, {links} links válidos");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("Falha de evidências:\n{message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{frontmatter, md_links};

    #[test]
    fn frontmatter_isolado() {
        let source = "---\ntype: OperationEvidence\noperation: processo.enviar\n---\n";
        assert_eq!(frontmatter(source, "operation"), Some("processo.enviar"));
        assert_eq!(frontmatter(source, "type"), Some("OperationEvidence"));
        assert_eq!(frontmatter("sem frontmatter", "type"), None);
    }

    #[test]
    fn multiplas_relacoes_na_mesma_linha() {
        let line = "- [A](../../cassettes/unb/a.md), [B](../../cassettes/go/b.md)";
        assert_eq!(
            md_links(line),
            vec!["../../cassettes/unb/a.md", "../../cassettes/go/b.md"]
        );
    }
}
