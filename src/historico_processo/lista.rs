//! A página `procedimento_consultar_historico`: as linhas pelo leitor de
//! histórico (`crate::historico`, também usado pelo replay do acervo), a legenda `Lista de Andamentos (N registro):` ou
//! `(N registros):` da `table#tblHistorico` e a visão servida em
//! `form#frmProcedimentoHistorico input[name=hdnTipoHistorico]`.
use super::{AndamentosLidos, ErroAndamentos, TipoDoHistorico};
use crate::formulario_web::localizar_formulario;
use crate::historico::ler_historico;
use crate::legenda::{Legenda, ler_legenda};

/// Lê a página e confere o NUP do título com o pedido.
pub(crate) fn ler_andamentos(
    html: &crate::tela_html::HtmlServido,
    nup: &str,
) -> Result<AndamentosLidos, ErroAndamentos> {
    let detalhe = ler_historico(html, nup)?;
    let dom = crate::tela_html::arvore(html);
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
fn legenda(dom: &crate::tela_html::Documento) -> Result<usize, ErroAndamentos> {
    let nos = dom.com_id("tblHistorico").collect::<Vec<_>>();
    let tabela = nos
        .first()
        .copied()
        .ok_or(ErroAndamentos::LegendaDosAndamentosAusente)?;
    let legendas: Vec<String> = tabela
        .elementos("caption")
        .map(|no| {
            no.texto()
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
fn tipo_servido(dom: &crate::tela_html::Documento) -> Result<TipoDoHistorico, ErroAndamentos> {
    let form = localizar_formulario(dom, super::VER_HISTORICO_COMPLETO.formulario)
        .map_err(ErroAndamentos::FormularioDoTipoNaoLocalizado)?;
    let mut valores = Vec::new();
    for tag in form
        .descendentes()
        .filter(|t| t.nome().eq_ignore_ascii_case("input"))
    {
        if tag.atributo("name") == Some(super::CAMPO_TIPO) {
            let bruto = tag
                .atributo("value")
                .map(str::to_owned)
                .ok_or(ErroAndamentos::TipoDoHistoricoSemValor)?;
            valores.push(bruto);
        }
    }
    match valores.as_slice() {
        [v] if v == "R" => Ok(TipoDoHistorico::Resumido),
        [v] if v == "P" => Ok(TipoDoHistorico::Completo),
        _ => Err(ErroAndamentos::TipoDoHistoricoNaoPublicado),
    }
}
