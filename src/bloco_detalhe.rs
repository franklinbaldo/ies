//! Interpretação somente-leitura da tabela tblProtocolosBlocos.
//! Caso derivado de Todos `src/todos/sei/blocos_lista.py` @323eb01.
use crate::bloco::{Faixa, TipoBloco};
use crate::html_atributos::nos_com_id;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct ItemDoBloco {
    pub(crate) processo: String,
    /// Para blocos internos/de reunião, o item é um processo, não documento.
    pub(crate) documento: Option<String>,
    pub(crate) tipo: Option<String>,
    pub(crate) anotacoes: String,
}
#[derive(Debug, Serialize)]
pub(crate) struct BlocoDetalhado {
    pub(crate) numero: String,
    pub(crate) tipo: TipoBloco,
    /// O estado foi observado na lista que ofereceu o link do detalhe.
    pub(crate) estado: String,
    pub(crate) total_declarado: usize,
    pub(crate) faixa_observada: Option<Faixa>,
    pub(crate) itens: Vec<ItemDoBloco>,
}
fn texto(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> String {
    html_escape::decode_html_entities(&tag.inner_text(parser))
        .into_owned()
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// A tela de detalhe do bloco fora da forma observada: um erro por fato.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroBlocoDetalhe {
    CabecalhoDoBlocoInvalido,
    FaixaDoBlocoInvalida,
    HtmlDoBlocoIlegivel,
    LegendaDoBlocoInvalida,
    LinhaDoBlocoInvalida,
    TabelaDoBlocoAusente,
    TabelasDoBlocoRepetidas,
}

fn faixa(legenda: &str, linhas: usize) -> Result<(usize, Option<Faixa>), ErroBlocoDetalhe> {
    let (inicio, resto) = legenda
        .split_once(" registros")
        .ok_or(ErroBlocoDetalhe::LegendaDoBlocoInvalida)?;
    let total = inicio
        .split_whitespace()
        .last()
        .ok_or(ErroBlocoDetalhe::LegendaDoBlocoInvalida)?
        .trim_start_matches('(')
        .replace('.', "")
        .parse::<usize>()
        .map_err(|_| ErroBlocoDetalhe::LegendaDoBlocoInvalida)?;
    if let Some((_, r)) = resto.split_once(" - ") {
        let (de, ate) = r
            .split_once(" a ")
            .ok_or(ErroBlocoDetalhe::FaixaDoBlocoInvalida)?;
        let de = de
            .parse::<usize>()
            .map_err(|_| ErroBlocoDetalhe::FaixaDoBlocoInvalida)?;
        let ate = ate
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse::<usize>()
            .map_err(|_| ErroBlocoDetalhe::FaixaDoBlocoInvalida)?;
        if de == 0 || ate < de || ate > total || ate - de + 1 != linhas {
            return Err(ErroBlocoDetalhe::FaixaDoBlocoInvalida);
        }
        return Ok((
            total,
            Some(Faixa {
                inicio: de,
                fim: ate,
            }),
        ));
    }
    if total != linhas {
        return Err(ErroBlocoDetalhe::FaixaDoBlocoInvalida);
    }
    Ok((
        total,
        (total > 0).then_some(Faixa {
            inicio: 1,
            fim: total,
        }),
    ))
}
fn ler_itens(
    tabela: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    headers: &[String],
    tipo: TipoBloco,
) -> Result<Vec<ItemDoBloco>, ErroBlocoDetalhe> {
    let index = |nome: &str| headers.iter().position(|c| c == nome);
    let processo = index("Processo").ok_or(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido)?;
    let anotacoes = index("Anotações").ok_or(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido)?;
    let documento = index("Documento");
    let tipo_coluna = index("Tipo");
    let mut itens = Vec::new();
    for no in tabela.query_selector(parser, "tr").into_iter().flatten() {
        let tr = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroBlocoDetalhe::LinhaDoBlocoInvalida)?;
        let td: Vec<_> = tr
            .children()
            .top()
            .iter()
            .filter_map(|h| h.get(parser))
            .filter_map(tl::Node::as_tag)
            .filter(|t| t.name() == "td")
            .collect();
        if td.is_empty() {
            continue;
        }
        if td.len() != headers.len() {
            return Err(ErroBlocoDetalhe::LinhaDoBlocoInvalida);
        }
        let processo = texto(td[processo], parser);
        if processo.is_empty() {
            return Err(ErroBlocoDetalhe::LinhaDoBlocoInvalida);
        }
        let doc = documento
            .map(|i| texto(td[i], parser))
            .filter(|s| !s.is_empty());
        if tipo == TipoBloco::Assinatura && doc.is_none() {
            return Err(ErroBlocoDetalhe::LinhaDoBlocoInvalida);
        }
        itens.push(ItemDoBloco {
            processo,
            documento: doc,
            tipo: tipo_coluna
                .map(|i| texto(td[i], parser))
                .filter(|s| !s.is_empty()),
            anotacoes: texto(td[anotacoes], parser),
        });
    }
    Ok(itens)
}

pub(crate) fn ler_detalhe(
    html: &str,
    numero: &str,
    tipo: TipoBloco,
    estado: &str,
) -> Result<BlocoDetalhado, ErroBlocoDetalhe> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroBlocoDetalhe::HtmlDoBlocoIlegivel)?;
    let parser = dom.parser();
    let tabelas = nos_com_id(&dom, "tblProtocolosBlocos")
        .map_err(|_| ErroBlocoDetalhe::HtmlDoBlocoIlegivel)?;
    let tabela = match tabelas.as_slice() {
        [] => return Err(ErroBlocoDetalhe::TabelaDoBlocoAusente),
        [no] => no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroBlocoDetalhe::TabelaDoBlocoAusente)?,
        _ => return Err(ErroBlocoDetalhe::TabelasDoBlocoRepetidas),
    };
    if tabela.name() != "table" {
        return Err(ErroBlocoDetalhe::TabelaDoBlocoAusente);
    }
    let mut captions = tabela
        .query_selector(parser, "caption")
        .ok_or(ErroBlocoDetalhe::LegendaDoBlocoInvalida)?;
    let caption = captions
        .next()
        .and_then(|h| h.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or(ErroBlocoDetalhe::LegendaDoBlocoInvalida)?;
    if captions.next().is_some() {
        return Err(ErroBlocoDetalhe::LegendaDoBlocoInvalida);
    }
    let mut headers = None;
    for no in tabela.query_selector(parser, "tr").into_iter().flatten() {
        let tr = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido)?;
        let th: Vec<_> = tr
            .children()
            .top()
            .iter()
            .filter_map(|h| h.get(parser))
            .filter_map(tl::Node::as_tag)
            .filter(|t| t.name() == "th")
            .collect();
        if !th.is_empty() {
            if headers.is_some() {
                return Err(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido);
            }
            headers = Some(th.iter().map(|t| texto(t, parser)).collect::<Vec<_>>());
        }
    }
    let headers = headers.ok_or(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido)?;
    for campo in ["Processo", "Anotações"] {
        if headers.iter().filter(|h| h.as_str() == campo).count() != 1 {
            return Err(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido);
        }
    }
    if tipo == TipoBloco::Assinatura
        && headers.iter().filter(|h| h.as_str() == "Documento").count() != 1
    {
        return Err(ErroBlocoDetalhe::CabecalhoDoBlocoInvalido);
    }
    let itens = ler_itens(tabela, parser, &headers, tipo)?;
    let (total_declarado, faixa_observada) = faixa(&texto(caption, parser), itens.len())?;
    Ok(BlocoDetalhado {
        numero: numero.to_owned(),
        tipo,
        estado: estado.to_owned(),
        total_declarado,
        faixa_observada,
        itens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn documentos_do_bloco_sao_lidos_pelas_colunas_nao_pela_posicao() {
        // Caso de Todos tests/paginas_reais/bloco_com_documentos.html @323eb01.
        let html = "<table id='tblProtocolosBlocos'><caption>Lista de Processos/Documentos (1 registros):</caption><tr><th></th><th>Documento</th><th>Processo</th><th>Anotações</th><th>Tipo</th></tr><tr><td><input type='checkbox'/></td><td>0001234</td><td>0000.00/2026-01</td><td>Observação</td><td>Despacho</td></tr></table>";
        let d = ler_detalhe(html, "91", TipoBloco::Assinatura, "Gerado").unwrap();
        assert_eq!(d.total_declarado, 1);
        assert_eq!(d.itens[0].documento.as_deref(), Some("0001234"));
        assert_eq!(d.itens[0].anotacoes, "Observação");
    }
    #[test]
    fn interno_nao_inventa_documento_ou_tipo() {
        // Caso de Todos tests/test_bloco_de_processos_010.py @323eb01.
        let html = "<table id='tblProtocolosBlocos'><caption>Lista (1 registros):</caption><tr><th></th><th>Processo</th><th>Anotações</th></tr><tr><td><input type='checkbox'/></td><td>NUP 001</td><td></td></tr></table>";
        let d = ler_detalhe(html, "91", TipoBloco::Interno, "Gerado").unwrap();
        assert!(d.itens[0].documento.is_none());
        assert!(d.itens[0].tipo.is_none());
    }
    #[test]
    fn detalhe_truncado_recusado() {
        let html = "<table id='tblProtocolosBlocos'><caption>Lista (9 registros):</caption><tr><th>Processo</th><th>Anotações</th></tr><tr><td>NUP</td><td></td></tr></table>";
        assert_eq!(
            ler_detalhe(html, "91", TipoBloco::Reuniao, "Gerado").err(),
            Some(ErroBlocoDetalhe::FaixaDoBlocoInvalida)
        );
    }
}
