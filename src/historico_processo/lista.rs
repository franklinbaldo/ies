//! A página `procedimento_consultar_historico`: as linhas pelo leitor de
//! histórico (`crate::historico`, também usado pelo replay do acervo), a legenda `Lista de Andamentos (N registro):` ou
//! `(N registros):` da `table#tblHistorico` e a visão servida em
//! `form#frmProcedimentoHistorico input[name=hdnTipoHistorico]`.
use super::{AndamentosLidos, ErroAndamentos, TipoDoHistorico};
use crate::formulario_web::{decodificar_atributo, localizar_formulario};
use crate::historico::ler_historico;
use crate::html_atributos::{nos_com_id, valor_do_atributo};

/// Lê a página e confere o NUP do título com o pedido.
pub(crate) fn ler_andamentos(html: &str, nup: &str) -> Result<AndamentosLidos, ErroAndamentos> {
    let detalhe = ler_historico(html, nup)?;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroAndamentos::LegendaDosAndamentosAusente)?;
    let total_declarado = legenda(&dom)?;
    let tipo = tipo_servido(&dom)?;
    if detalhe.andamentos.len() as u64 > total_declarado {
        return Err(ErroAndamentos::AndamentosAlemDoTotalDeclarado);
    }
    Ok(AndamentosLidos {
        tipo,
        total_declarado,
        itens: detalhe.andamentos,
    })
}

/// A única `caption` da tabela (já conferida única pelo leitor das linhas).
fn legenda(dom: &tl::VDom<'_>) -> Result<u64, ErroAndamentos> {
    let parser = dom.parser();
    let tabela = nos_com_id(dom, "tblHistorico")
        .ok()
        .and_then(|nos| nos.first().copied())
        .and_then(|no| no.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or(ErroAndamentos::LegendaDosAndamentosAusente)?;
    let legendas: Vec<String> = tabela
        .query_selector(parser, "caption")
        .into_iter()
        .flatten()
        .filter_map(|no| no.get(parser))
        .map(|no| {
            no.inner_text(parser)
                .replace("&nbsp;", " ")
                .replace('\u{a0}', " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    let texto = match legendas.as_slice() {
        [] => return Err(ErroAndamentos::LegendaDosAndamentosAusente),
        [texto] => texto,
        _ => return Err(ErroAndamentos::LegendaDosAndamentosForaDaFormaObservada),
    };
    let fora = ErroAndamentos::LegendaDosAndamentosForaDaFormaObservada;
    let resto = texto.strip_prefix("Lista de Andamentos (").ok_or(fora)?;
    let numero = resto
        .strip_suffix(" registros):")
        .or_else(|| resto.strip_suffix(" registro):"))
        .ok_or(fora)?;
    if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fora);
    }
    numero.parse().map_err(|_| fora)
}

/// O único `hdnTipoHistorico` do formulário: `R` ou `P`.
fn tipo_servido(dom: &tl::VDom<'_>) -> Result<TipoDoHistorico, ErroAndamentos> {
    let ausente = ErroAndamentos::TipoDoHistoricoNaoPublicado;
    let parser = dom.parser();
    let form =
        localizar_formulario(dom, super::VER_HISTORICO_COMPLETO.formulario).map_err(|_| ausente)?;
    let mut valores = Vec::new();
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter(|t| t.name().as_utf8_str().eq_ignore_ascii_case("input"))
    {
        if valor_do_atributo(tag, "name")
            .map_err(ErroAndamentos::Atributo)?
            .as_deref()
            == Some(super::CAMPO_TIPO)
        {
            let bruto = valor_do_atributo(tag, "value").map_err(ErroAndamentos::Atributo)?;
            valores.push(decodificar_atributo(&bruto.unwrap_or_default()).ok_or(ausente)?);
        }
    }
    match valores.as_slice() {
        [v] if v == "R" => Ok(TipoDoHistorico::Resumido),
        [v] if v == "P" => Ok(TipoDoHistorico::Completo),
        _ => Err(ausente),
    }
}
