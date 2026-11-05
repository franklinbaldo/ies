//! Paginação da visão detalhada: o índice oculto e a legenda com a faixa.
use super::ErroDetalhada;
use crate::html_atributos::{nos_com_id, valor_do_atributo};

/// `hdnDetalhadoPaginaAtual` com índice diferente de `0`. O índice `0`
/// sozinho não é paginação: a UFRJ 4.1.5 o publica numa D de uma página,
/// sem paginador (04/11/2026), como `hdn{T}PaginaAtual` na R.
pub(super) fn indice_oculto_alem_da_primeira(
    dom: &tl::VDom<'_>,
    parser: &tl::Parser<'_>,
) -> Result<bool, ErroDetalhada> {
    let nos = nos_com_id(dom, "hdnDetalhadoPaginaAtual").map_err(|e| {
        ErroDetalhada::Html(crate::erro_html::ErroHtml::atributo(
            "html_detalhado_nao_interpretado",
            "o HTML não pôde ser interpretado",
            e,
        ))
    })?;
    for no in nos {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            return Ok(true);
        };
        let valor = valor_do_atributo(tag, "value").map_err(|e| {
            ErroDetalhada::Html(crate::erro_html::ErroHtml::atributo(
                "html_detalhado_nao_interpretado",
                "o HTML não pôde ser interpretado",
                e,
            ))
        })?;
        if valor.as_deref().map(str::trim) != Some("0") {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Legenda da lista detalhada, sem supor que o total é igual à página.
pub(super) fn faixa(
    legenda: &str,
    linhas: usize,
    paginacao_visivel: bool,
) -> Result<(usize, usize, usize), ErroDetalhada> {
    let (anterior, restante) = legenda
        .split_once(" registro")
        .ok_or(ErroDetalhada::TotalNaoDeclarado)?;
    let bruto = anterior
        .split_whitespace()
        .last()
        .ok_or(ErroDetalhada::TotalNaoDeclarado)?
        .trim_start_matches('(');
    let total = bruto
        .replace('.', "")
        .parse::<usize>()
        .map_err(|_| ErroDetalhada::TotalNaoDeclarado)?;
    let Some((_, intervalo)) = restante.split_once(" - ") else {
        if paginacao_visivel {
            return Err(ErroDetalhada::FaixaSemIntervaloComPaginacao);
        }
        return Ok((total, 1, linhas));
    };
    let (inicio, fim) = intervalo.split_once(" a ").ok_or(ErroDetalhada::Html(
        crate::erro_html::ErroHtml::valor(
            "faixa_detalhada_invalida",
            "o valor não está na forma observada",
        ),
    ))?;
    let inicio = inicio.trim().parse::<usize>().map_err(|_| {
        ErroDetalhada::Html(crate::erro_html::ErroHtml::valor(
            "faixa_detalhada_invalida",
            "o valor não está na forma observada",
        ))
    })?;
    let fim = fim
        .trim()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse::<usize>()
        .map_err(|_| {
            ErroDetalhada::Html(crate::erro_html::ErroHtml::valor(
                "faixa_detalhada_invalida",
                "o valor não está na forma observada",
            ))
        })?;
    if inicio == 0 || inicio > fim || fim > total || fim - inicio + 1 != linhas {
        return Err(ErroDetalhada::Html(crate::erro_html::ErroHtml::valor(
            "faixa_detalhada_invalida",
            "o valor não está na forma observada",
        )));
    }
    Ok((total, inicio, fim))
}
