//! Parser Web puro: HTML -> `CaixaDeEntrada` tipada; sem estado de sessão ou JSON.
use crate::classe_link_processo::estado_do_link;
use crate::dominio::{CaixaDeEntrada, Categoria, IdProcedimento, Processo, Tabela};
use crate::erro_web::ErroCaixa;
use crate::legenda::{ErroLegenda, ler_legenda};

mod vazia;
pub(crate) use vazia::lista_vazia_comprovada;

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
    html: &crate::tela_html::HtmlServido,
    dom: &crate::tela_html::Documento,
    table_id: &str,
    categoria: Categoria,
) -> Result<Tabela<Processo>, ErroCaixa> {
    let (total, linhas) = ler_linhas(html, dom, table_id, categoria)?;
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
    html: &crate::tela_html::HtmlServido,
    dom: &crate::tela_html::Documento,
    table_id: &str,
    categoria: Categoria,
) -> Result<(usize, Vec<LinhaLida>), ErroCaixa> {
    // A primeira ocorrência do ID não comprova que não houve uma segunda tabela.
    // Se duas existem, escolher uma pode omitir processos apesar dos totais declarados.
    let candidatas = dom.com_id(table_id).collect::<Vec<_>>();
    let primeira = match candidatas.as_slice() {
        [] if vazia::lista_vazia_comprovada(html, dom, categoria) => return Ok((0, Vec::new())),
        [] => return Err(ErroCaixa::TabelaAusente { categoria }),
        [primeira] => *primeira,
        _ => return Err(ErroCaixa::TabelasDaCategoriaRepetidas { categoria }),
    };
    if primeira.nome() != "table" || primeira.elementos("table").next().is_some() {
        // Não permitir que linhas de tabelas aninhadas contaminem o resultado.
        return Err(ErroCaixa::TabelaInvalida { categoria });
    }

    let legenda = primeira
        .elementos("caption")
        .next()
        .ok_or(ErroCaixa::LegendaAusente { categoria })?
        .texto();
    let total = match ler_legenda(&legenda) {
        Ok(lida) => lida.total,
        Err(ErroLegenda::SemContagem) => {
            return Err(ErroCaixa::TotalNaoDeclaradoNaLegenda { categoria });
        }
        Err(motivo) => return Err(ErroCaixa::LegendaIlegivel { categoria, motivo }),
    };
    let mut processos = Vec::new();
    let linhas = primeira.elementos("tr");
    for linha in linhas {
        let Some(id) = linha.atributo("id") else {
            // Cabeçalhos e avisos podem aparecer entre as linhas.
            continue;
        };
        let Some(id) = id.strip_prefix('P') else {
            continue;
        };
        let id = IdProcedimento::novo(id)?;
        let links = linha.elementos("a");
        let mut encontrado = None;
        for link in links {
            let classes = link.atributo("class").unwrap_or_default();
            let estado = estado_do_link(classes).map_err(|motivo| {
                ErroCaixa::ClassesDeEstadoDiferentesNoLinkDoProcesso {
                    categoria,
                    id: id.as_str().to_owned(),
                    motivo,
                }
            })?;
            if let Some(estado) = estado {
                let numero = link.texto();
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
                    link.atributo("href").map(str::to_owned),
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

pub fn parse_html(html: &crate::tela_html::HtmlServido) -> Result<CaixaDeEntrada, ErroCaixa> {
    let dom = crate::tela_html::arvore(html);
    Ok(CaixaDeEntrada {
        recebidos: ler_tabela(html, &dom, "tblProcessosRecebidos", Categoria::Recebidos)?,
        gerados: ler_tabela(html, &dom, "tblProcessosGerados", Categoria::Gerados)?,
    })
}

/// As linhas de Recebidos e Gerados com o `href` do número, pela mesma
/// leitura e com as mesmas recusas de [`parse_html`].
pub(crate) fn linhas_com_link(
    html: &crate::tela_html::HtmlServido,
) -> Result<Vec<LinhaDoControle>, ErroCaixa> {
    let dom = crate::tela_html::arvore(html);
    let mut todas = Vec::new();
    for (tabela, categoria) in [
        ("tblProcessosRecebidos", Categoria::Recebidos),
        ("tblProcessosGerados", Categoria::Gerados),
    ] {
        let (total, linhas) = ler_linhas(html, &dom, tabela, categoria)?;
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
        assert_eq!(ler_legenda("25 registros:").map(|l| l.total), Ok(25));
        assert_eq!(
            ler_legenda("Processos recebidos (131 registros - 1 a 10):").map(|l| l.total),
            Ok(131)
        );
        assert_eq!(
            ler_legenda("131&nbsp;registros:"),
            Err(crate::legenda::ErroLegenda::SemContagem)
        );
        assert_eq!(
            ler_legenda("indefinido"),
            Err(crate::legenda::ErroLegenda::SemContagem)
        );
    }

    #[test]
    fn linha_sem_id_nao_cria_processo() {
        let resultado = parse_html(&crate::tela_html::HtmlServido::de_teste(&html(
            "<tr><td>aviso</td></tr>",
            1,
        )))
        .unwrap();
        assert_eq!(resultado.recebidos.processos, []);
        assert_eq!(resultado.recebidos.total_declarado, 1);
    }

    #[test]
    fn legenda_ilegivel_falha_com_erro_tipado() {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<table id='tblProcessosRecebidos'><caption>indefinido:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>",
        );
        assert_eq!(
            parse_html(&html),
            Err(ErroCaixa::TotalNaoDeclaradoNaLegenda {
                categoria: Categoria::Recebidos
            })
        );
    }

    #[test]
    fn recusa_id_repetido() {
        let _linha = linha("42");
    }

    #[test]
    fn booleano_antes_de_id_e_class_nao_omite_tabela_nem_processo() {
        let original = html(&format!("{}{}", linha("42"), linha("43")), 2);
        let com_booleano = crate::tela_html::HtmlServido::de_teste(
            &crate::tela_html::teste::com_booleano_antes_dos_atributos(&original),
        );
        assert_eq!(
            parse_html(&com_booleano),
            parse_html(&crate::tela_html::HtmlServido::de_teste(&original))
        );
        assert_eq!(
            parse_html(&com_booleano).unwrap().recebidos.processos.len(),
            2
        );
    }

    #[test]
    fn recusa_dois_links_de_processo_na_linha() {
        let linhas = "<tr id='P7'><td><a class='processoVisualizado'>123</a>\
            <a class='processoNaoVisualizado'>124</a></td></tr>";
        let erro =
            parse_html(&crate::tela_html::HtmlServido::de_teste(&html(linhas, 1))).unwrap_err();
        assert_eq!(erro.id(), Some("7"));
    }

    #[test]
    fn recusa_legenda_ausente() {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<table id='tblProcessosRecebidos'></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>",
        );
        assert!(matches!(
            parse_html(&html),
            Err(ErroCaixa::LegendaAusente { .. })
        ));
    }

    #[test]
    fn recusa_excesso_de_linhas() {
        assert!(matches!(
            parse_html(&crate::tela_html::HtmlServido::de_teste(&html(
                &format!("{}{}", linha("1"), linha("2")),
                1
            ))),
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
            parse_html(&crate::tela_html::HtmlServido::de_teste(&html(linhas, 1))),
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
        let erro = parse_html(&crate::tela_html::HtmlServido::de_teste(&format!(
            "{original}{extra}"
        )))
        .unwrap_err();
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
            parse_html(&crate::tela_html::HtmlServido::de_teste(&format!(
                "{original}{extra}"
            )))
            .unwrap_err(),
            ErroCaixa::TabelasDaCategoriaRepetidas {
                categoria: Categoria::Gerados
            }
        );
    }
}

#[cfg(test)]
#[path = "caixa/tests_classes.rs"]
mod tests_classes;
