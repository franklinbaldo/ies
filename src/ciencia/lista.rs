//! Lista de Ciências relida depois do GET mutador: a atividade do aceite
//! tem de aparecer uma única vez no `table#tblAndamentos` publicado.
use super::{ErroCiencia, identificador};
use crate::html_atributos::{nos_com_id, valor_do_atributo};

/// Verifica a atividade no único `table#tblAndamentos` da Lista de Ciências
/// que o processo/documento publicou na **releitura**. O chamador deve antes
/// conferir a URL da lista contra o alvo e a sessão: o HTML isolado não prova
/// qual processo serviu a tabela. Não deduz usuário nem data não observados.
pub(crate) fn atividade_na_lista(html: &str, atividade: &str) -> Result<bool, ErroCiencia> {
    if !identificador(atividade) {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    }
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroCiencia::ListaDeCienciasNaoInterpretada)?;
    let parser = dom.parser();
    let tabelas = nos_com_id(&dom, "tblAndamentos").map_err(ErroCiencia::Atributo)?;
    let [unica] = tabelas.as_slice() else {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    };
    let tabela = unica
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroCiencia::ListaDeCienciasForaDaFormaObservada)?;
    if tabela.name().as_utf8_str() != "table" {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    }
    // O SEI depende de JavaScript ativo. Mesmo que o parser HTML indexe
    // a tabela ou o checkbox dentro de <template> ou <noscript>, o navegador
    // não os usa como a Lista de Ciências publicada quando JS está ativo.
    // Recusar tanto os ancestrais da tabela como os fragmentos descendentes:
    // uma atividade antiga num nó inerte não prova o efeito do GET atual.
    for seletor in ["template", "noscript"] {
        let ancestral_inerte = dom
            .query_selector(seletor)
            .into_iter()
            .flatten()
            .filter_map(|no| no.get(parser).and_then(tl::Node::as_tag))
            .any(|inerte| {
                inerte
                    .query_selector(parser, "table")
                    .into_iter()
                    .flatten()
                    .any(|no| no == *unica)
            });
        let descendente_inerte = tabela
            .query_selector(parser, seletor)
            .into_iter()
            .flatten()
            .next()
            .is_some();
        if ancestral_inerte || descendente_inerte {
            return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
        }
    }
    let legendas: Vec<String> = tabela
        .query_selector(parser, "caption")
        .into_iter()
        .flatten()
        .filter_map(|no| no.get(parser))
        .map(|no| no.inner_text(parser).into_owned())
        .collect();
    let [legenda] = legendas.as_slice() else {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    };
    // Com ou sem contagem: `UnB` 4.0.9 publica "Lista de Ciências (1 registro):".
    let contagem = legenda
        .strip_prefix("Lista de Ciências")
        .and_then(|resto| resto.strip_suffix(':'))
        .ok_or(ErroCiencia::ListaDeCienciasForaDaFormaObservada)?;
    let contagem_valida = contagem.is_empty()
        || contagem
            .strip_prefix(" (")
            .and_then(|resto| resto.strip_suffix(')'))
            .and_then(|resto| resto.split_once(' '))
            .is_some_and(|(n, palavra)| {
                identificador(n) && matches!(palavra, "registro" | "registros")
            });
    if !contagem_valida {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    }
    let mut ocorrencias = 0;
    for no in tabela.query_selector(parser, "input").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
        };
        let tipo = valor_do_atributo(tag, "type").map_err(ErroCiencia::Atributo)?;
        if tipo.as_deref() != Some("checkbox") {
            continue;
        }
        let nome = valor_do_atributo(tag, "name").map_err(ErroCiencia::Atributo)?;
        let valor = valor_do_atributo(tag, "value").map_err(ErroCiencia::Atributo)?;
        let Some(nome) = nome else {
            return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
        };
        let Some(resto) = nome.strip_prefix("chkInfraItem") else {
            continue;
        };
        if !resto.bytes().all(|c| c.is_ascii_digit()) {
            return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
        }
        if valor.as_deref() == Some(atividade) {
            ocorrencias += 1;
        }
    }
    match ocorrencias {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ErroCiencia::AtividadesRepetidasNaLista),
    }
}
