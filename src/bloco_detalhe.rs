//! Interpretação somente-leitura da tabela tblProtocolosBlocos.
//! Caso derivado de Todos `src/todos/sei/blocos_lista.py` @323eb01.
use crate::bloco::{Faixa, TipoBloco};
use crate::erro_html::ErroHtml;
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
fn faixa(legenda: &str, linhas: usize) -> Result<(usize, Option<Faixa>), ErroHtml> {
    use ErroHtml as E;
    let (inicio, resto) = legenda
        .split_once(" registros")
        .ok_or(E::valor("legenda_do_bloco_invalida"))?;
    let total = inicio
        .split_whitespace()
        .last()
        .ok_or(E::valor("legenda_do_bloco_invalida"))?
        .trim_start_matches('(')
        .replace('.', "")
        .parse::<usize>()
        .map_err(|_| E::valor("legenda_do_bloco_invalida"))?;
    if let Some((_, r)) = resto.split_once(" - ") {
        let (de, ate) = r
            .split_once(" a ")
            .ok_or(E::valor("faixa_do_bloco_invalida"))?;
        let de = de
            .parse::<usize>()
            .map_err(|_| E::valor("faixa_do_bloco_invalida"))?;
        let ate = ate
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse::<usize>()
            .map_err(|_| E::valor("faixa_do_bloco_invalida"))?;
        if de == 0 || ate < de || ate > total || ate - de + 1 != linhas {
            return Err(E::valor("faixa_do_bloco_invalida"));
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
        return Err(E::valor("faixa_do_bloco_invalida"));
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
) -> Result<Vec<ItemDoBloco>, ErroHtml> {
    use ErroHtml as E;
    let index = |nome: &str| headers.iter().position(|c| c == nome);
    let processo = index("Processo").ok_or(E::valor("cabecalho_do_bloco_invalido"))?;
    let anotacoes = index("Anotações").ok_or(E::valor("cabecalho_do_bloco_invalido"))?;
    let documento = index("Documento");
    let tipo_coluna = index("Tipo");
    let mut itens = Vec::new();
    for no in tabela.query_selector(parser, "tr").into_iter().flatten() {
        let tr = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(E::valor("linha_do_bloco_invalida"))?;
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
            return Err(E::valor("linha_do_bloco_invalida"));
        }
        let processo = texto(td[processo], parser);
        if processo.is_empty() {
            return Err(E::valor("linha_do_bloco_invalida"));
        }
        let doc = documento
            .map(|i| texto(td[i], parser))
            .filter(|s| !s.is_empty());
        if tipo == TipoBloco::Assinatura && doc.is_none() {
            return Err(E::valor("linha_do_bloco_invalida"));
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
) -> Result<BlocoDetalhado, ErroHtml> {
    use ErroHtml as E;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| E::html("html_do_bloco_ilegivel"))?;
    let parser = dom.parser();
    let tabelas = nos_com_id(&dom, "tblProtocolosBlocos")
        .map_err(|e| E::atributo("html_do_bloco_ilegivel", e))?;
    let tabela = match tabelas.as_slice() {
        [] => return Err(E::ausente("tabela_do_bloco_ausente")),
        [no] => no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(E::ausente("tabela_do_bloco_ausente"))?,
        _ => return Err(E::repetido("tabelas_do_bloco_repetidas")),
    };
    if tabela.name() != "table" {
        return Err(E::ausente("tabela_do_bloco_ausente"));
    }
    let mut captions = tabela
        .query_selector(parser, "caption")
        .ok_or(E::valor("legenda_do_bloco_invalida"))?;
    let caption = captions
        .next()
        .and_then(|h| h.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or(E::valor("legenda_do_bloco_invalida"))?;
    if captions.next().is_some() {
        return Err(E::valor("legenda_do_bloco_invalida"));
    }
    let mut headers = None;
    for no in tabela.query_selector(parser, "tr").into_iter().flatten() {
        let tr = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(E::valor("cabecalho_do_bloco_invalido"))?;
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
                return Err(E::valor("cabecalho_do_bloco_invalido"));
            }
            headers = Some(th.iter().map(|t| texto(t, parser)).collect::<Vec<_>>());
        }
    }
    let headers = headers.ok_or(E::valor("cabecalho_do_bloco_invalido"))?;
    for campo in ["Processo", "Anotações"] {
        if headers.iter().filter(|h| h.as_str() == campo).count() != 1 {
            return Err(E::valor("cabecalho_do_bloco_invalido"));
        }
    }
    if tipo == TipoBloco::Assinatura
        && headers.iter().filter(|h| h.as_str() == "Documento").count() != 1
    {
        return Err(E::valor("cabecalho_do_bloco_invalido"));
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
            Some(ErroHtml::valor("faixa_do_bloco_invalida"))
        );
    }
}
