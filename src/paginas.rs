//! Consolidação conservadora de páginas já observadas: nunca envia POST.
//! Cada tabela pagina de modo independente. Faixas repetidas só podem coincidir
//! exatamente; lacunas, sobreposições e troca de conteúdo falham fechado.
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Faixa {
    total: usize,
    inicio: usize,
    fim: usize,
}

fn faixa_da_legenda(caption: &str, linhas: usize) -> Result<Option<Faixa>, String> {
    let legenda = caption.trim();
    let marcador = " registros";
    let Some((antes, depois)) = legenda.split_once(marcador) else {
        return Ok(None);
    };
    let total = antes
        .split_whitespace()
        .last()
        .and_then(|s| s.trim_start_matches('(').parse::<usize>().ok())
        .ok_or("total de registros inválido")?;
    if let Some(rest) = depois.strip_prefix(" - ") {
        let (a, b) = rest
            .split_once(" a ")
            .ok_or("intervalo de paginação inválido")?;
        let inicio = a
            .trim()
            .parse::<usize>()
            .map_err(|_| "início de faixa inválido")?;
        let fim = b
            .trim()
            .trim_end_matches(|c: char| !c.is_ascii_digit())
            .parse::<usize>()
            .map_err(|_| "fim de faixa inválido")?;
        if inicio == 0 || inicio > fim || fim > total || fim - inicio + 1 != linhas {
            return Err("faixa de paginação inconsistente com as linhas".into());
        }
        Ok(Some(Faixa { total, inicio, fim }))
    } else if total == linhas {
        Ok(Some(Faixa {
            total,
            inicio: 1,
            fim: total,
        }))
    } else {
        // Sem faixa, não se pode afirmar completude por agregação.
        Ok(None)
    }
}

fn faixa_da_tabela(html: &str, table_id: &str, linhas: usize) -> Result<Option<Faixa>, String> {
    let dom =
        tl::parse(html, tl::ParserOptions::default()).map_err(|_| "HTML de paginação inválido")?;
    let parser = dom.parser();
    let table = dom
        .get_element_by_id(table_id)
        .and_then(|node| node.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or("tabela ausente para paginação")?;
    let caption = table
        .query_selector(parser, "caption")
        .and_then(|mut it| it.next())
        .and_then(|n| n.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or("legenda de paginação ausente")?
        .inner_text(parser);
    faixa_da_legenda(&caption, linhas)
}

#[derive(Default)]
struct Grupo {
    total: Option<usize>,
    faixas: BTreeMap<usize, (usize, Vec<Value>)>,
    desconhecido: bool,
}

impl Grupo {
    fn add(&mut self, classe: &str, parsed: &Value, faixa: Option<Faixa>) -> Result<(), String> {
        let processes = parsed["processos"]
            .as_array()
            .ok_or("processos não são uma lista")?
            .clone();
        let declared = parsed["total_declarado"].as_u64().map(|v| v as usize);
        if let (Some(previous), Some(current)) = (self.total, declared)
            && previous != current
        {
            return Err(format!("total de {classe} mudou entre páginas"));
        }
        if self.total.is_none() {
            self.total = declared;
        }
        let Some(faixa) = faixa else {
            self.desconhecido = true;
            return if self.faixas.is_empty() {
                self.faixas.insert(1, (processes.len(), processes));
                Ok(())
            } else {
                Err(format!(
                    "faixa indeterminada de {classe} em páginas adicionais"
                ))
            };
        };
        if declared != Some(faixa.total) {
            return Err(format!("legenda de {classe} e total não coincidem"));
        }
        if self.desconhecido {
            return Err(format!(
                "não se pode agregar página de {classe} sem faixa inicial"
            ));
        }
        match self.faixas.get(&faixa.inicio) {
            Some((end, before)) if *end == faixa.fim && *before == processes => Ok(()),
            Some(_) => Err(format!("mesma faixa de {classe} mudou de conteúdo")),
            None => {
                self.faixas.insert(faixa.inicio, (faixa.fim, processes));
                Ok(())
            }
        }
    }

    fn into_result(self, category: &str) -> Result<Value, String> {
        let mut cursor = 1usize;
        let mut ids = HashSet::new();
        let mut processos = Vec::new();
        for (inicio, (fim, rows)) in &self.faixas {
            if *inicio != cursor {
                return Err(format!(
                    "lacuna ou sobreposição em {category}: posição {cursor}"
                ));
            }
            for row in rows {
                let id = row["id"].as_str().ok_or("processo sem id")?;
                if !ids.insert(id.to_owned()) {
                    return Err(format!("processo duplicado entre páginas de {category}"));
                }
                processos.push(row.clone());
            }
            cursor = fim.saturating_add(1);
        }
        if let Some(total) = self.total
            && processos.len() > total
        {
            return Err(format!("mais processos que o total de {category}"));
        }
        let completeness = if self.desconhecido || self.total.is_none() {
            "indeterminada"
        } else if self.total == Some(processos.len()) {
            "completa"
        } else {
            "parcial"
        };
        Ok(json!({
            "categoria":category,
            "total_declarado":self.total,
            "linhas_lidas":processos.len(),
            "paginas_lidas":self.faixas.len(),
            "completude":completeness,
            "processos":processos
        }))
    }
}

pub fn juntar(htmls: &[String], unidade: Option<&str>) -> Result<Value, String> {
    if htmls.is_empty() || htmls.len() > 200 {
        return Err("número de páginas inválido (1 a 200)".into());
    }
    if htmls.iter().map(String::len).sum::<usize>() > 48_000_000 {
        return Err("conjunto de HTML maior que 48 MB".into());
    }
    let mut received = Grupo::default();
    let mut created = Grupo::default();
    for html in htmls {
        let parsed = crate::caixa::parse_html(html, unidade)?;
        for (category, table, group) in [
            ("recebidos", "tblProcessosRecebidos", &mut received),
            ("gerados", "tblProcessosGerados", &mut created),
        ] {
            let rows = parsed[category]["linhas_lidas"]
                .as_u64()
                .ok_or("quantidade de linhas ausente")? as usize;
            let range = faixa_da_tabela(html, table, rows)?;
            group.add(category, &parsed[category], range)?;
        }
    }
    let received = received.into_result("recebidos")?;
    let created = created.into_result("gerados")?;
    let complete = received["completude"] == "completa" && created["completude"] == "completa";
    Ok(json!({
        "unidade": unidade,
        "recebidos": received,
        "gerados": created,
        "completude": if complete { "completa" } else { "parcial_ou_indeterminada" },
        "fonte":"tela_do_sei",
        "paginas_observadas": htmls.len(),
        "avisos": if complete { Vec::<&str>::new() } else {
            vec!["Listagem não comprovadamente completa; não foi executada paginação HTTP."]
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intervalo_reconhecido() {
        assert_eq!(
            faixa_da_legenda("Processos recebidos (131 registros - 1 a 10):", 10).unwrap(),
            Some(Faixa {
                total: 131,
                inicio: 1,
                fim: 10
            })
        );
        assert_eq!(
            faixa_da_legenda("25 registros:", 25).unwrap(),
            Some(Faixa {
                total: 25,
                inicio: 1,
                fim: 25
            })
        );
        assert!(faixa_da_legenda("5 registros - 3 a 4):", 1).is_err());
    }
}
#[cfg(test)]
mod test_paginas {
    use super::*;
    fn linha(id: u32) -> String {
        format!(
            r#"<tr id="P{id}"><td><a class="processoVisualizado" href="controlador.php?acao=procedimento_trabalhar&amp;infra_hash=SECRETO">NUP-{id}</a></td></tr>"#
        )
    }
    fn html(
        rec: (usize, usize, usize, &[u32]),
        generated: (usize, usize, usize, &[u32]),
    ) -> String {
        let tabela = |kind: &str, (total, start, end, ids): (usize, usize, usize, &[u32])| {
            format!(
                r#"<table id="tblProcessos{kind}"><caption>Processos ({total} registros - {start} a {end}):</caption>{}</table>"#,
                ids.iter().map(|i| linha(*i)).collect::<String>()
            )
        };
        format!(
            "<html>{}{}</html>",
            tabela("Recebidos", rec),
            tabela("Gerados", generated)
        )
    }
    #[test]
    fn duas_tabelas_paginam_independentemente() {
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        let result = juntar(&[a, b], Some("TESTE")).unwrap();
        assert_eq!(result["completude"], "completa");
        assert_eq!(result["recebidos"]["linhas_lidas"], 4);
        assert_eq!(result["recebidos"]["paginas_lidas"], 2);
        assert_eq!(result["gerados"]["linhas_lidas"], 1);
        assert_eq!(result["gerados"]["paginas_lidas"], 1);
        assert_eq!(result["paginas_observadas"], 2);
        assert!(!result.to_string().contains("SECRETO"));
    }
    #[test]
    fn recusa_lacuna_de_intervalos() {
        let a = html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((5, 4, 5, &[4, 5]), (1, 1, 1, &[11]));
        assert!(juntar(&[a, b], None).is_err());
    }
    #[test]
    fn recusa_sobreposicao_ou_troca_de_conteudo() {
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 2, 3, &[2, 3]), (1, 1, 1, &[11]));
        assert!(juntar(&[a.clone(), b], None).is_err());
        let changed = html((4, 1, 2, &[1, 7]), (1, 1, 1, &[11]));
        assert!(juntar(&[a, changed], None).is_err());
    }
    #[test]
    fn recusa_processos_duplicados_entre_paginas() {
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 3, 4, &[2, 4]), (1, 1, 1, &[11]));
        assert!(juntar(&[a, b], None).is_err());
    }
    #[test]
    fn recusa_total_que_muda_entre_paginas() {
        let a = html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        assert!(juntar(&[a, b], None).is_err());
    }
    #[test]
    fn pagina_inicial_parcial_nao_finge_completude() {
        let page = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let result = juntar(&[page], None).unwrap();
        assert_eq!(result["completude"], "parcial_ou_indeterminada");
    }
}
