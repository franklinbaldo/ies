//! Leitura estruturada das duas tabelas independentes da caixa do SEI.
//! Não serializa links assinados e nunca infere que uma página parcial está completa.
use serde_json::{Value, json};
use std::collections::HashSet;

fn atributo(tag: &tl::HTMLTag<'_>, key: &str) -> Option<String> {
    tag.attributes()
        .get(key)
        .flatten()
        .map(|v| v.as_utf8_str().into_owned())
}

fn total_da_legenda(text: &str) -> Option<usize> {
    let words: Vec<&str> = text.split_whitespace().collect();
    words
        .windows(2)
        .find(|pair| pair[1].starts_with("registro"))
        .and_then(|pair| pair[0].trim_start_matches('(').parse::<usize>().ok())
}

fn conteudo_tabela(dom: &tl::VDom<'_>, table_id: &str, category: &str) -> Result<Value, String> {
    let parser = dom.parser();
    let table = dom
        .get_element_by_id(table_id)
        .and_then(|node| node.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or_else(|| format!("tabela ausente: {category}"))?;
    if table.name() != "table" {
        return Err(format!("elemento inválido para {category}"));
    }

    let caption = table
        .query_selector(parser, "caption")
        .and_then(|mut it| it.next())
        .and_then(|n| n.get(parser))
        .and_then(tl::Node::as_tag)
        .map(|tag| tag.inner_text(parser).into_owned())
        .ok_or_else(|| format!("legenda ausente: {category}"))?;
    let total = total_da_legenda(&caption);
    let mut processos = Vec::new();
    let mut ids = HashSet::new();
    let rows = table
        .query_selector(parser, "tr")
        .ok_or_else(|| "seletor HTML inválido".to_owned())?;
    for row in rows {
        let Some(tag) = row.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(id) = atributo(tag, "id") else {
            continue;
        };
        let Some(id) = id.strip_prefix('P') else {
            continue;
        };
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("identificador de processo inválido: {category}"));
        }
        if !ids.insert(id.to_owned()) {
            return Err(format!("processo repetido na mesma tabela: {category}"));
        }

        let anchors = tag
            .query_selector(parser, "a")
            .ok_or_else(|| "seletor de links inválido".to_owned())?;
        let mut found = Vec::new();
        for anchor in anchors {
            let Some(a) = anchor.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            let class = atributo(a, "class").unwrap_or_default();
            let visto = if class.split_whitespace().any(|x| x == "processoVisualizado") {
                Some(true)
            } else if class
                .split_whitespace()
                .any(|x| x == "processoNaoVisualizado")
            {
                Some(false)
            } else {
                None
            };
            if let Some(visualizado) = visto {
                // O link assinado nunca sai no modelo JSON nem é salvo no índice.
                let numero = a.inner_text(parser).trim().to_owned();
                if numero.is_empty() || numero.len() > 100 {
                    return Err(format!("número de processo inválido: {category}"));
                }
                found.push(json!({"id":id,"numero":numero,"visualizado":visualizado}));
            }
        }
        if found.len() != 1 {
            return Err(format!("nó de processo ambíguo: {category}"));
        }
        processos.push(found.remove(0));
    }

    let status = match total {
        Some(n) if processos.len() == n => "completa",
        Some(n) if processos.len() < n => "parcial",
        Some(_) => return Err(format!("mais linhas que o total declarado: {category}")),
        None => "indeterminada",
    };
    Ok(json!({
        "categoria":category,
        "total_declarado":total,
        "linhas_lidas":processos.len(),
        "completude":status,
        "processos":processos
    }))
}

pub fn parse_html(html: &str, unidade: Option<&str>) -> Result<Value, String> {
    if html.len() > 8_000_000 {
        return Err("HTML excede o limite de 8 MB".into());
    }
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| "HTML não interpretável".to_owned())?;
    let recebidos = conteudo_tabela(&dom, "tblProcessosRecebidos", "recebidos")?;
    let gerados = conteudo_tabela(&dom, "tblProcessosGerados", "gerados")?;
    let completa = recebidos["completude"] == "completa" && gerados["completude"] == "completa";
    Ok(json!({
        "unidade":unidade,
        "recebidos":recebidos,
        "gerados":gerados,
        "completude": if completa { "completa" } else { "parcial_ou_indeterminada" },
        "fonte":"tela_do_sei",
        "avisos": if completa { Vec::<&str>::new() } else {
            vec!["A caixa pode ter mais páginas. Nenhuma página extra foi inferida ou requisitada."]
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legendas() {
        assert_eq!(total_da_legenda("25 registros:"), Some(25));
        assert_eq!(
            total_da_legenda("Processos recebidos (131 registros - 1 a 10):"),
            Some(131)
        );
        assert_eq!(total_da_legenda("sem total"), None);
    }
}
