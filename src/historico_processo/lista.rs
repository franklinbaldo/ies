//! A página `procedimento_consultar_historico`: as linhas pelo leitor de
//! histórico (`crate::historico`, também usado pelo replay do acervo), a legenda `Lista de Andamentos (N registro):` ou
//! `(N registros):` da `table#tblHistorico` e a visão servida em
//! `form#frmProcedimentoHistorico input[name=hdnTipoHistorico]`.
use super::{AndamentosLidos, ErroAndamentos, TipoDoHistorico};
use crate::formulario_web::{decodificar_atributo, localizar_formulario};
use crate::historico::ler_historico;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::legenda::{Legenda, ler_legenda};

/// Lê a página e confere o NUP do título com o pedido.
pub(crate) fn ler_andamentos(
    html: &crate::tela_html::HtmlServido,
    nup: &str,
) -> Result<AndamentosLidos, ErroAndamentos> {
    let detalhe = ler_historico(html, nup)?;
    let dom = crate::tela_html::interpretar(html);
    let total = legenda(&dom)?;
    let tipo = tipo_servido(&dom)?;
    if detalhe.andamentos.len() > total {
        return Err(ErroAndamentos::AndamentosAlemDoTotalDeclarado);
    }
    let total_declarado =
        u64::try_from(total).map_err(ErroAndamentos::TotalDeclaradoNaoCabeEmU64)?;
    Ok(AndamentosLidos {
        tipo,
        total_declarado,
        itens: detalhe.andamentos,
    })
}

/// A única `caption` da tabela (já conferida única pelo leitor das linhas).
fn legenda(dom: &tl::VDom<'_>) -> Result<usize, ErroAndamentos> {
    let parser = dom.parser();
    let nos = nos_com_id(dom, "tblHistorico").map_err(ErroAndamentos::Atributo)?;
    let tabela = nos
        .first()
        .copied()
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
    if !texto.starts_with("Lista de Andamentos (") {
        return Err(ErroAndamentos::LegendaDosAndamentosForaDaFormaObservada);
    }
    let Legenda { total, faixa } = ler_legenda(texto)?;
    if faixa.is_some() {
        return Err(ErroAndamentos::LegendaDosAndamentosForaDaFormaObservada);
    }
    Ok(total)
}

/// O único `hdnTipoHistorico` do formulário: `R` ou `P`.
fn tipo_servido(dom: &tl::VDom<'_>) -> Result<TipoDoHistorico, ErroAndamentos> {
    let parser = dom.parser();
    let form = localizar_formulario(dom, super::VER_HISTORICO_COMPLETO.formulario)
        .map_err(ErroAndamentos::FormularioDoTipoNaoLocalizado)?;
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
            let bruto = valor_do_atributo(tag, "value")
                .map_err(ErroAndamentos::Atributo)?
                .ok_or(ErroAndamentos::TipoDoHistoricoSemValor)?;
            valores.push(
                decodificar_atributo(&bruto).ok_or(ErroAndamentos::TipoDoHistoricoNaoPublicado)?,
            );
        }
    }
    match valores.as_slice() {
        [v] if v == "R" => Ok(TipoDoHistorico::Resumido),
        [v] if v == "P" => Ok(TipoDoHistorico::Completo),
        _ => Err(ErroAndamentos::TipoDoHistoricoNaoPublicado),
    }
}
