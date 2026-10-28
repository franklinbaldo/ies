//! Gate metodológico OKF: uma implementação não-stub precisa de decisão
//! arquitetural explícita, alternativas e vínculos reais com evidências.
//! Este gate verifica estrutura/proveniência, não julga mérito científico.
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn frontmatter(content: &str) -> Option<HashMap<&str, &str>> {
    let body = content.strip_prefix("---\n")?;
    let (head, _) = body.split_once("\n---\n")?;
    let mut fields = HashMap::new();
    for line in head.lines() {
        let (key, value) = line.split_once(':')?;
        if key.is_empty() || value.trim().is_empty() || fields.insert(key, value.trim()).is_some() {
            return None;
        }
    }
    Some(fields)
}

fn links(markdown: &str) -> Vec<&str> {
    let mut rest = markdown;
    let mut result = Vec::new();
    while let Some((_, tail)) = rest.split_once("](") {
        if let Some((url, after)) = tail.split_once(')') {
            if !url.is_empty() && !url.contains('#') {
                result.push(url);
            }
            rest = after;
        } else {
            break;
        }
    }
    result
}

fn validate_decision(
    contract: &str,
    decision: &str,
    root: &Path,
    decision_path: &Path,
    expected_operation: &str,
) -> Result<(), String> {
    let c = frontmatter(contract).ok_or("frontmatter do contrato inválido")?;
    let d = frontmatter(decision).ok_or("frontmatter da decisão inválido")?;
    if c.get("type") != Some(&"CommandContract")
        || d.get("type") != Some(&"ImplementationDecision")
        || c.get("operation") != Some(&expected_operation)
        || d.get("operation") != Some(&expected_operation)
    {
        return Err("identidades ou tipos OKF inconsistentes".into());
    }
    let implementation = c
        .get("implementation")
        .ok_or("implementação não declarada")?;
    if *implementation == "stub" {
        return Err("decisão aprovada não é exigida para stub".into());
    }
    if !["partial", "complete"].contains(implementation)
        || d.get("implementation") != Some(implementation)
    {
        return Err("estado de implementação desconhecido ou divergente".into());
    }
    if d.get("status") != Some(&"adopted-for-scope") {
        return Err("decisão não adotada explicitamente para o escopo".into());
    }
    let scope = d.get("scope").ok_or("escopo não declarado")?;
    if scope.len() < 12 {
        return Err("escopo insuficiente".into());
    }
    let selected = d
        .get("selected_strategy")
        .ok_or("estratégia não escolhida")?;
    let alternatives = d.get("alternatives").ok_or("alternativas ausentes")?;
    let distinct: HashSet<_> = alternatives.split(',').map(str::trim).collect();
    if distinct.len() < 2 || !distinct.contains(selected) || distinct.contains("") {
        return Err("alternativas precisam ser diferentes e conter a escolhida".into());
    }
    for key in ["evidence_level", "live_validation"] {
        if !d.contains_key(key) {
            return Err(format!("campo obrigatório ausente: {key}"));
        }
    }
    for heading in [
        "## Alternativas consideradas",
        "## Comparação",
        "## Decisão",
        "## Evidências verificáveis",
        "## Limitações e revisão",
    ] {
        if !decision.contains(heading) {
            return Err(format!("seção de decisão ausente: {heading}"));
        }
    }
    let found = links(decision);
    let mut has_contract = false;
    let mut has_matrix = false;
    let mut has_test = false;
    for link in found {
        if link.starts_with("https://") {
            continue;
        }
        if link.starts_with("http://") || link.starts_with('/') || link.contains('?') {
            return Err(format!("referência local inválida: {link}"));
        }
        let target = decision_path
            .parent()
            .ok_or("diretório da decisão inválido")?
            .join(link);
        let canonical =
            fs::canonicalize(&target).map_err(|_| format!("referência ausente: {link}"))?;
        if !canonical.starts_with(root) || !canonical.is_file() {
            return Err(format!("referência fora do repositório: {link}"));
        }
        let relative = canonical
            .strip_prefix(root)
            .map_err(|_| "referência inválida")?;
        let name = relative.to_string_lossy();
        has_contract |= name
            == format!(
                "contracts/cli/{}.md",
                expected_operation.replace(['.', '_'], "-")
            );
        has_matrix |= name
            == format!(
                "contracts/evidencias/{}.md",
                expected_operation.replace('.', "-")
            );
        has_test |= name.starts_with("tests/") && name.ends_with(".rs");
    }
    if !has_contract || !has_matrix || !has_test {
        return Err("decisão precisa ligar contrato, evidência e teste Rust existente".into());
    }
    Ok(())
}

fn check(root: &Path) -> Result<(usize, usize), String> {
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let dir = root.join("contracts/cli");
    let decisions = root.join("docs/decisoes");
    let mut checked = HashSet::new();
    let mut stubs = 0usize;
    let mut failures = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let file = entry.map_err(|e| e.to_string())?.path();
        if file.extension().is_none_or(|x| x != "md") {
            continue;
        }
        let content = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let fields = match frontmatter(&content) {
            Some(f) => f,
            None => {
                failures.push(format!("{}: frontmatter inválido", file.display()));
                continue;
            }
        };
        if fields.get("type") != Some(&"CommandContract") {
            continue;
        }
        let operation = fields.get("operation").copied().unwrap_or("");
        if operation.is_empty()
            || file.file_stem().unwrap_or_default().to_string_lossy()
                != operation.replace(['.', '_'], "-")
        {
            failures.push(format!(
                "{}: identidade de operação inconsistente",
                file.display()
            ));
            continue;
        }
        match fields.get("implementation").copied() {
            Some("stub") => stubs += 1,
            Some("partial" | "complete") => {
                let decision_path =
                    decisions.join(format!("{}.md", operation.replace(['.', '_'], "-")));
                let result = fs::read_to_string(&decision_path)
                    .map_err(|_| "decisão ausente para contrato implementado".to_owned())
                    .and_then(|body| {
                        validate_decision(&content, &body, &root, &decision_path, operation)
                    });
                match result {
                    Ok(()) => {
                        checked.insert(operation.replace(['.', '_'], "-"));
                    }
                    Err(message) => failures.push(format!("{operation}: {message}")),
                }
            }
            _ => failures.push(format!("{operation}: estado de implementação inválido")),
        }
    }
    if decisions.is_dir() {
        for entry in fs::read_dir(&decisions).map_err(|e| e.to_string())? {
            let file = entry.map_err(|e| e.to_string())?.path();
            if file.extension().is_none_or(|x| x != "md") {
                continue;
            }
            let name = file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let body = fs::read_to_string(&file).map_err(|e| e.to_string())?;
            if frontmatter(&body)
                .and_then(|m| m.get("operation").copied())
                .is_none_or(|op| op.replace(['.', '_'], "-") != name)
            {
                failures.push(format!("{}: decisão sem identidade válida", file.display()));
            } else if !checked.contains(&name) && name != "README" {
                failures.push(format!(
                    "{}: decisão órfã ou sem contrato implementado",
                    file.display()
                ));
            }
        }
    }
    if failures.is_empty() {
        Ok((checked.len(), stubs))
    } else {
        Err(failures.join("\n"))
    }
}

fn main() -> ExitCode {
    match check(Path::new(env!("CARGO_MANIFEST_DIR"))) {
        Ok((implemented, stubs)) => {
            println!(
                "Decisões de engenharia: {implemented} implementações com alternativas; {stubs} stubs dispensados"
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Gate de decisões OKF:\n{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aceita_opcoes_diferentes_sem_presumir_equivalencia() {
        let fm = "---\ntype: CommandContract\noperation: unidade.listar_processos\nimplementation: partial\n---\n";
        assert_eq!(frontmatter(fm).unwrap()["implementation"], "partial");
        let options: HashSet<_> = "web-observed, rest-wssei, soap-official"
            .split(',')
            .map(str::trim)
            .collect();
        assert_eq!(options.len(), 3);
    }
    #[test]
    fn rejeita_frontmatter_duplicado() {
        assert!(frontmatter("---\ntype: CommandContract\ntype: Guide\n---\n").is_none());
    }
    #[test]
    fn encontra_links_markdown() {
        assert_eq!(
            links("[contrato](../../contracts/cli/unidade-listar_processos.md)"),
            vec!["../../contracts/cli/unidade-listar_processos.md"]
        );
    }
}
