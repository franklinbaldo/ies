//! Parser Web puro: HTML -> `CaixaDeEntrada` tipada; sem estado de sessão ou JSON.
use crate::classe_link_processo::{ClassesDeEstadoDiferentesNoLink, estado_do_link};
use crate::dominio::{CaixaDeEntrada, Categoria, IdProcedimento, Processo, Tabela};
use crate::erro_web::ErroCaixa;
use crate::html_atributos::{nos_com_id, valor_do_atributo};

mod vazia;
pub(crate) use vazia::lista_vazia_comprovada;

/// Lido da fonte da tag; tag ilegível não vira atributo ausente.
fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, ErroCaixa> {
    valor_do_atributo(tag, nome).map_err(ErroCaixa::AtributoDaCaixaIlegivel)
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

/// Uma linha do Controle (visão R ou D) com o `href` do link do número
/// como a fonte da tag o traz (entidades ainda codificadas). Só para a
/// colheita do índice local; nunca sai em saída nem em `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct LinhaDoControle {
    pub(crate) id: String,
    pub(crate) numero: String,
    pub(crate) href: Option<String>,
}

impl std::fmt::Debug for LinhaDoControle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinhaDoControle")
            .field("id", &self.id)
            .field("href", &self.href.as_ref().map(|_| "<redigido>"))
            .finish_non_exhaustive()
    }
}

fn ler_tabela(
    dom: &tl::VDom<'_>,
    table_id: &str,
    categoria: Categoria,
) -> Result<Tabela<Processo>, ErroCaixa> {
    let (total, linhas) = ler_linhas(dom, table_id, categoria)?;
    Tabela::nova(
        categoria,
        total,
        linhas.into_iter().map(|(processo, _)| processo).collect(),
    )
}

/// Um processo da tabela com o `href` bruto do seu link.
type LinhaLida = (Processo, Option<String>);

/// Total da legenda e cada processo com o `href` bruto do seu link.
fn ler_linhas(
    dom: &tl::VDom<'_>,
    table_id: &str,
    categoria: Categoria,
) -> Result<(Option<usize>, Vec<LinhaLida>), ErroCaixa> {
    let parser = dom.parser();
    // A primeira ocorrência do ID não comprova que não houve uma segunda tabela.
    // Se duas existem, escolher uma pode omitir processos apesar dos totais declarados.
    let candidatas = nos_com_id(dom, table_id).map_err(ErroCaixa::AtributoDaCaixaIlegivel)?;
    let primeira = match candidatas.as_slice() {
        [] if vazia::lista_vazia_comprovada(dom, categoria)? => return Ok((Some(0), Vec::new())),
        [] => return Err(ErroCaixa::TabelaAusente { categoria }),
        [primeira] => *primeira,
        _ => return Err(ErroCaixa::TabelasDaCategoriaRepetidas { categoria }),
    };
    let tabela = primeira
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroCaixa::TabelaInvalida { categoria })?;
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
        let Some(id) = atributo(tag, "id")? else {
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
            let classes = atributo(a, "class")?.unwrap_or_default();
            let estado = estado_do_link(&classes).map_err(|ClassesDeEstadoDiferentesNoLink| {
                ErroCaixa::ClassesDeEstadoDiferentesNoLinkDoProcesso {
                    categoria,
                    id: id.as_str().to_owned(),
                }
            })?;
            if let Some(estado) = estado {
                let numero = a.inner_text(parser).into_owned();
                if numero.is_empty() {
                    return Err(ErroCaixa::NumeroInvalido {
                        categoria,
                        id: id.as_str().to_owned(),
                    });
                }
                if encontrado.is_some() {
                    return Err(ErroCaixa::MaisDeUmNumeroNaLinhaDoProcesso {
                        categoria,
                        id: id.as_str().to_owned(),
                    });
                }
                encontrado = Some((
                    Processo {
                        id: id.clone(),
                        numero,
                        visualizado: estado.visualizado,
                        sigiloso: estado.sigiloso,
                    },
                    atributo(a, "href")?,
                ));
            }
        }
        processos.push(
            encontrado.ok_or(ErroCaixa::LinkDoProcessoNaoIdentificadoNaLinha {
                categoria,
                id: id.as_str().to_owned(),
            })?,
        );
    }
    Ok((total, processos))
}

pub fn parse_html(html: &str) -> Result<CaixaDeEntrada, ErroCaixa> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroCaixa::HtmlDaCaixaInvalido)?;
    Ok(CaixaDeEntrada {
        recebidos: ler_tabela(&dom, "tblProcessosRecebidos", Categoria::Recebidos)?,
        gerados: ler_tabela(&dom, "tblProcessosGerados", Categoria::Gerados)?,
    })
}

/// As linhas de Recebidos e Gerados com o `href` do número, pela mesma
/// leitura e com as mesmas recusas de [`parse_html`].
pub(crate) fn linhas_com_link(html: &str) -> Result<Vec<LinhaDoControle>, ErroCaixa> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroCaixa::HtmlDaCaixaInvalido)?;
    let mut todas = Vec::new();
    for (tabela, categoria) in [
        ("tblProcessosRecebidos", Categoria::Recebidos),
        ("tblProcessosGerados", Categoria::Gerados),
    ] {
        let (total, linhas) = ler_linhas(&dom, tabela, categoria)?;
        let processos = linhas.iter().map(|(p, _)| p.clone()).collect();
        Tabela::nova(categoria, total, processos)?;
        todas.extend(linhas.into_iter().map(|(p, href)| LinhaDoControle {
            id: p.id.as_str().to_owned(),
            numero: p.numero,
            href,
        }));
    }
    Ok(todas)
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
        let _linha = linha("42");
    }

    #[test]
    fn booleano_antes_de_id_e_class_nao_omite_tabela_nem_processo() {
        let original = html(&format!("{}{}", linha("42"), linha("43")), 2);
        let com_booleano = crate::html_atributos::com_booleano_antes_dos_atributos(&original);
        assert_eq!(parse_html(&com_booleano), parse_html(&original));
        assert_eq!(
            parse_html(&com_booleano).unwrap().recebidos.processos.len(),
            2
        );
    }

    #[test]
    fn recusa_dois_links_de_processo_na_linha() {
        let linhas = "<tr id='P7'><td><a class='processoVisualizado'>123</a>\
            <a class='processoNaoVisualizado'>124</a></td></tr>";
        let erro = parse_html(&html(linhas, 1)).unwrap_err();
        assert_eq!(erro.id(), Some("7"));
    }

    #[test]
    fn recusa_legenda_ausente() {
        let html = "<table id='tblProcessosRecebidos'></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        assert!(matches!(
            parse_html(html),
            Err(ErroCaixa::LegendaAusente { .. })
        ));
    }

    #[test]
    fn recusa_excesso_de_linhas() {
        assert!(matches!(
            parse_html(&html(&format!("{}{}", linha("1"), linha("2")), 1)),
            Err(ErroCaixa::TotalExcedido { .. })
        ));
    }

    #[test]
    fn recusa_tabela_aninhada() {
        let _linhas = format!(
            "<tr id='P3'><td>{}<table><tr id='P4'><td><a class='processoVisualizado'>outro</a></td></tr></table></td></tr>",
            "<a class='processoVisualizado'>123</a>"
        );
    }

    #[test]
    fn ancora_de_classe_desconhecida_falha_fechado() {
        let linhas = "<tr id='P9'><td><a class='outraClasse'>123</a></td></tr>";
        assert!(matches!(
            parse_html(&html(linhas, 1)),
            Err(ErroCaixa::LinkDoProcessoNaoIdentificadoNaLinha { .. })
        ));
    }

    #[test]
    fn duas_tabelas_de_recebidos_nao_podem_ocultar_processos() {
        let original = html(&linha("1"), 1);
        let extra = format!(
            "<table id='tblProcessosRecebidos'><caption>1 registros:</caption>{}</table>",
            linha("2")
        );
        let erro = parse_html(&format!("{original}{extra}")).unwrap_err();
        assert_eq!(
            erro,
            ErroCaixa::TabelasDaCategoriaRepetidas {
                categoria: Categoria::Recebidos
            }
        );
    }

    #[test]
    fn duas_tabelas_de_gerados_nao_podem_ser_tratadas_como_uma() {
        let original = html("", 0);
        let extra = "<table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        assert_eq!(
            parse_html(&format!("{original}{extra}")).unwrap_err(),
            ErroCaixa::TabelasDaCategoriaRepetidas {
                categoria: Categoria::Gerados
            }
        );
    }
}

#[cfg(test)]
#[path = "caixa/tests_classes.rs"]
mod tests_classes;
