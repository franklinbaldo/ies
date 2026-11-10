//! Tela de Anexação de Processos do SEI, sem rede nem escrita.
//! Oráculo: Todos src/todos/anexacao.py @323eb01 (`UnB` 4.0.9).
//! Nenhum JavaScript executado, atributos só via `html_atributos`.
use crate::formulario_web::{decodificar_atributo, localizar_formulario};
use crate::html_atributos::valor_do_atributo;
use crate::transporte::texto_de_recusa::decodificar_entidades;
use std::collections::HashSet;

pub(crate) mod fluxo;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TelaDeAnexacao {
    pub(crate) anexados: Vec<String>,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Destino {
    pub(crate) id: String,
    pub(crate) nup: String,
    pub(crate) tipo: String,
}
fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, &'static str> {
    valor_do_atributo(tag, nome)
        .map_err(|_| "atributos_ambiguos_na_anexacao")?
        .map(|v| decodificar_atributo(&v).ok_or("entidade_html_nao_interpretada_na_anexacao"))
        .transpose()
}
/// Exige controles únicos dentro do formulário real.
pub(crate) fn ler_tela(html: &str) -> Result<TelaDeAnexacao, &'static str> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| "html_de_anexacao_nao_interpretado")?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, "frmProcedimentoAnexar")
        .map_err(|_| "formulario_de_anexacao_ausente_ou_repetido")?;
    let esperados = [
        "txtProtocolo",
        "txtIdentificacaoProtocolo",
        "hdnIdProtocolo",
        "hdnAnexacao",
        "sbmAnexar",
    ];
    let mut vistos = [0usize; 5];
    let mut tabelas = Vec::new();
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str();
        // `UnB` 4.0.9 publica o envio como <button type="submit">.
        if (elemento.eq_ignore_ascii_case("input") || elemento.eq_ignore_ascii_case("button"))
            && let Some(nome) = atributo(tag, "name")?
            && let Some(pos) = esperados.iter().position(|c| *c == nome)
        {
            vistos[pos] += 1;
            let tipo = atributo(tag, "type")?;
            let valido = match pos {
                0 | 1 => tipo
                    .as_deref()
                    .is_none_or(|v| v.eq_ignore_ascii_case("text")),
                2 | 3 => tipo
                    .as_deref()
                    .is_some_and(|v| v.eq_ignore_ascii_case("hidden")),
                _ => tipo
                    .as_deref()
                    .is_some_and(|v| v.eq_ignore_ascii_case("submit")),
            };
            if !valido {
                return Err("controles_da_anexacao_ausentes_ou_repetidos");
            }
        }
    }
    if vistos != [1; 5] {
        return Err("controles_da_anexacao_ausentes_ou_repetidos");
    }
    for table in dom.query_selector("table").into_iter().flatten() {
        let Some(tag) = table.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        if atributo(tag, "summary")?.as_deref() == Some("Lista de Processos Anexados") {
            tabelas.push(tag);
        }
    }
    let tabela = match tabelas.as_slice() {
        [] => {
            return Ok(TelaDeAnexacao {
                anexados: Vec::new(),
            });
        }
        [unica] => unica,
        _ => return Err("listas_de_anexados_repetidas"),
    };
    let mut anexados = Vec::new();
    let mut ids = HashSet::new();
    for tag in tabela
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("input")
            || !atributo(tag, "type")?.is_some_and(|v| v.eq_ignore_ascii_case("checkbox"))
        {
            continue;
        }
        let id = atributo(tag, "value")?
            .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
            .ok_or("id_do_anexado_ausente_ou_invalido")?;
        if !ids.insert(id.clone()) {
            return Err("ids_de_anexados_repetidos");
        }
        anexados.push(id);
    }
    Ok(TelaDeAnexacao { anexados })
}
/// Consulta `protocolo_RI1023`: ID, NUP e tipo únicos.
pub(crate) fn ler_destino(xml: &str) -> Result<Destino, &'static str> {
    let valores = ler_complementos(xml)?;
    let ids: Vec<_> = valores
        .iter()
        .filter(|(n, _)| n == "IdProcedimento")
        .collect();
    let nups: Vec<_> = valores
        .iter()
        .filter(|(n, _)| n == "ProtocoloProcedimentoFormatado")
        .collect();
    let tipos: Vec<_> = valores
        .iter()
        .filter(|(n, _)| n == "NomeTipoProcedimento")
        .collect();
    if ids.is_empty() || nups.is_empty() {
        return Err("processo_a_anexar_nao_identificado");
    }
    if ids.len() != 1 || nups.len() != 1 || tipos.len() > 1 {
        return Err("identidade_de_anexado_ambigua");
    }
    let id = ids[0].1.as_str();
    let nup = nups[0].1.as_str();
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) || nup.is_empty() {
        return Err("processo_a_anexar_nao_identificado");
    }
    Ok(Destino {
        id: id.to_owned(),
        nup: nup.to_owned(),
        tipo: tipos.first().map_or("", |v| v.1.as_str()).to_owned(),
    })
}
/// Consulta `procedimento_anexacao_verificar`: S=tudo assinado, N=há pendência.
pub(crate) fn ler_confirmacao(xml: &str) -> Result<bool, &'static str> {
    let valores = ler_complementos(xml)?;
    let encontrados: Vec<_> = valores.iter().filter(|(n, _)| n == "Anexacao").collect();
    match encontrados.as_slice() {
        [(_, valor)] if valor == "S" => Ok(true),
        [(_, valor)] if valor == "N" => Ok(false),
        _ => Err("anexacao_nao_verificada"),
    }
}
fn ler_complementos(xml: &str) -> Result<Vec<(String, String)>, &'static str> {
    let dom = tl::parse(xml, tl::ParserOptions::default())
        .map_err(|_| "complementos_da_anexacao_nao_interpretados")?;
    let parser = dom.parser();
    if dom
        .query_selector("erros")
        .into_iter()
        .flatten()
        .next()
        .is_some()
    {
        return Err("processo_a_anexar_nao_identificado");
    }
    let mut valores = Vec::new();
    for no in dom.query_selector("complemento").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        if let Some(nome) = atributo(tag, "nome")? {
            valores.push((nome, decodificar_entidades(&tag.inner_text(parser))));
        }
    }
    Ok(valores)
}
/// Recusa identidade igual ou já anexada antes da operação irreversível.
pub(crate) fn decidir(
    principal: &str,
    id_principal: &str,
    anexar: &Destino,
    existentes: &[String],
) -> Result<bool, &'static str> {
    if principal == anexar.nup || id_principal == anexar.id {
        return Err("anexacao_ao_mesmo_processo");
    }
    if existentes.contains(&anexar.id) {
        return Err("processo_ja_anexado");
    }
    Ok(true)
}
#[cfg(test)]
#[path = "processo_anexar/tests.rs"]
mod tests;
