//! Parser Web puro: HTML -> Caixa tipada; sem estado de sessão ou JSON.
use crate::dominio::{Caixa, Categoria, IdProcedimento, Processo, Tabela};
use crate::erro_web::ErroCaixa;

fn atributo(tag: &tl::HTMLTag<'_>, nome: &str) -> Option<String> {
    tag.attributes()
        .get(nome)
        .flatten()
        .map(|v| v.as_utf8_str().into_owned())
}

pub(crate) fn total_da_legenda(texto: &str) -> Option<usize> {
    // O tl mantém algumas entidades HTML como texto literal.
    let texto = texto
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace('\u{a0}', " ");
    let palavras: Vec<&str> = texto.split_whitespace().collect();
    palavras
        .windows(2)
        .find(|par| par[1].starts_with("registro"))
        .and_then(|par| par[0].trim_start_matches('(').parse::<usize>().ok())
}

fn ler_tabela(
    dom: &tl::VDom<'_>,
    table_id: &str,
    categoria: Categoria,
) -> Result<Tabela<Processo>, ErroCaixa> {
    let parser = dom.parser();
    let tabela = dom
        .get_element_by_id(table_id)
        .and_then(|no| no.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or(ErroCaixa::TabelaAusente { categoria })?;
    if tabela.name() != "table"
        || tabela
            .query_selector(parser, "table")
            .is_some_and(|mut filhos| filhos.next().is_some())
    {
        // Não permitir que linhas de tabelas aninhadas contaminem o resultado.
        return Err(ErroCaixa::TabelaInvalida { categoria });
    }

    let legenda = tabela
        .query_selector(parser, "caption")
        .and_then(|mut itens| itens.next())
        .and_then(|no| no.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or(ErroCaixa::LegendaAusente { categoria })?
        .inner_text(parser);
    let total = total_da_legenda(&legenda);
    let mut processos = Vec::new();
    let linhas = tabela
        .query_selector(parser, "tr")
        .ok_or(ErroCaixa::TabelaInvalida { categoria })?;
    for linha in linhas {
        let Some(tag) = linha.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(id) = atributo(tag, "id") else {
            // Cabeçalhos e avisos podem aparecer entre as linhas.
            continue;
        };
        let Some(id) = id.strip_prefix('P') else {
            continue;
        };
        let id = IdProcedimento::novo(id)?;
        let links = tag
            .query_selector(parser, "a")
            .ok_or(ErroCaixa::TabelaInvalida { categoria })?;
        let mut encontrado = None;
        for link in links {
            let Some(a) = link.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            let classes = atributo(a, "class").unwrap_or_default();
            let visualizado = if classes
                .split_whitespace()
                .any(|x| x == "processoVisualizado")
            {
                Some(true)
            } else if classes
                .split_whitespace()
                .any(|x| x == "processoNaoVisualizado")
            {
                Some(false)
            } else {
                None
            };
            if let Some(visualizado) = visualizado {
                let numero = a.inner_text(parser).trim().to_owned();
                if numero.is_empty() || numero.len() > 100 {
                    return Err(ErroCaixa::NumeroInvalido {
                        categoria,
                        id: id.as_str().to_owned(),
                    });
                }
                if encontrado.is_some() {
                    return Err(ErroCaixa::ProcessoAmbiguo {
                        categoria,
                        id: id.as_str().to_owned(),
                    });
                }
                encontrado = Some(Processo {
                    id: id.clone(),
                    numero,
                    visualizado,
                });
            }
        }
        processos.push(encontrado.ok_or(ErroCaixa::ProcessoAmbiguo {
            categoria,
            id: id.as_str().to_owned(),
        })?);
    }
    Tabela::nova(categoria, total, processos)
}

pub fn parse_html(html: &str) -> Result<Caixa, ErroCaixa> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroCaixa::HtmlInvalido)?;
    Ok(Caixa {
        recebidos: ler_tabela(&dom, "tblProcessosRecebidos", Categoria::Recebidos)?,
        gerados: ler_tabela(&dom, "tblProcessosGerados", Categoria::Gerados)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn html(recebidos: &str, total: usize) -> String {
        format!(
            "<table id='tblProcessosRecebidos'><caption>{total} registros:</caption>{recebidos}</table>\
             <table id='tblProcessosGerados'><caption>0 registros:</caption></table>"
        )
    }
    fn linha(id: &str) -> String {
        format!("<tr id='P{id}'><td><a class='processoVisualizado'>123</a></td></tr>")
    }

    #[test]
    fn legendas_reais_e_entidades() {
        assert_eq!(total_da_legenda("25 registros:"), Some(25));
        assert_eq!(
            total_da_legenda("Processos recebidos (131 registros - 1 a 10):"),
            Some(131)
        );
        assert_eq!(total_da_legenda("131&nbsp;registros:"), Some(131));
        assert_eq!(total_da_legenda("indefinido"), None);
    }

    #[test]
    fn linha_sem_id_nao_cria_processo() {
        let resultado = parse_html(&html("<tr><td>aviso</td></tr>", 1)).unwrap();
        assert_eq!(resultado.recebidos.processos, []);
        assert_eq!(resultado.recebidos.total_declarado, Some(1));
    }

    #[test]
    fn recusa_id_repetido() {
        let linha = linha("42");
        assert_eq!(
            parse_html(&html(&format!("{linha}{linha}"), 2))
                .unwrap_err()
                .subcodigo(),
            "id_repetido"
        );
    }

    #[test]
    fn recusa_link_ambiguo_com_id() {
        let linhas = "<tr id='P7'><td><a class='processoVisualizado'>123</a>\
            <a class='processoNaoVisualizado'>124</a></td></tr>";
        let erro = parse_html(&html(linhas, 1)).unwrap_err();
        assert_eq!(erro.subcodigo(), "processo_ambiguo");
        assert_eq!(erro.id(), Some("7"));
    }

    #[test]
    fn recusa_legenda_ausente() {
        let html = "<table id='tblProcessosRecebidos'></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        assert_eq!(parse_html(html).unwrap_err().subcodigo(), "legenda_ausente");
    }

    #[test]
    fn recusa_excesso_de_linhas() {
        assert_eq!(
            parse_html(&html(&format!("{}{}", linha("1"), linha("2")), 1))
                .unwrap_err()
                .subcodigo(),
            "total_excedido"
        );
    }

    #[test]
    fn recusa_tabela_aninhada() {
        let linhas = format!(
            "<tr id='P3'><td>{}<table><tr id='P4'><td><a class='processoVisualizado'>outro</a></td></tr></table></td></tr>",
            "<a class='processoVisualizado'>123</a>"
        );
        assert_eq!(
            parse_html(&html(&linhas, 2)).unwrap_err().subcodigo(),
            "tabela_invalida"
        );
    }

    #[test]
    fn ancora_de_classe_desconhecida_falha_fechado() {
        let linhas = "<tr id='P9'><td><a class='outraClasse'>123</a></td></tr>";
        assert_eq!(
            parse_html(&html(linhas, 1)).unwrap_err().subcodigo(),
            "processo_ambiguo"
        );
    }
}
