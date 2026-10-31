//! Identidade e critérios Web do Controle de Processos. Sem rede ou persistência.
use crate::erro_web::ErroCaixa;
/// Contexto da página HTML: só é usado para conferir a junção.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextoPagina {
    pub(crate) unidade: Option<String>,
    modo: Option<String>,
    criterios_recebidos: Option<String>,
    criterios_gerados: Option<String>,
}

impl ContextoPagina {
    pub(crate) fn verificavel(&self) -> bool {
        self.unidade.is_some()
            && self.modo.is_some()
            && self.criterios_recebidos.is_some()
            && self.criterios_gerados.is_some()
    }
}

fn unidade_html_original(dom: &tl::VDom<'_>) -> Option<String> {
    let parser = dom.parser();
    let link = dom
        .get_element_by_id("lnkInfraUnidade")
        .and_then(|n| n.get(parser))
        .and_then(tl::Node::as_tag)?;
    if link.name() != "a" {
        return None;
    }
    let texto = link
        .inner_text(parser)
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace('\u{a0}', " ");
    let unidade = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    (!unidade.is_empty() && unidade.len() <= 128).then_some(unidade)
}

/// Rótulo mostrado pelo SEI, não um ID nem uma lista de unidades.
pub fn unidade_exibida_no_controle(html: &str) -> Result<String, ErroCaixa> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroCaixa::HtmlInvalido)?;
    unidade_html(&dom)?.ok_or(ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle)
}

fn valor_oculto_original(dom: &tl::VDom<'_>, id: &str) -> Option<String> {
    let parser = dom.parser();
    let elemento = dom
        .get_element_by_id(id)
        .and_then(|n| n.get(parser))
        .and_then(tl::Node::as_tag)?;
    if elemento.name() != "input" {
        return None;
    }
    let valor = elemento
        .attributes()
        .get("value")
        .flatten()?
        .as_utf8_str()
        .trim()
        .to_owned();
    (!valor.is_empty() && valor.len() <= 128).then_some(valor)
}

pub(crate) fn contexto_da_pagina(html: &str) -> Result<ContextoPagina, ErroCaixa> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroCaixa::HtmlInvalido)?;
    Ok(ContextoPagina {
        unidade: unidade_html(&dom)?,
        modo: valor_oculto(&dom, "hdnTipoVisualizacao")?,
        criterios_recebidos: valor_oculto(&dom, "hdnRecebidosHashCriterios")?,
        criterios_gerados: valor_oculto(&dom, "hdnGeradosHashCriterios")?,
    })
}


// Shims do contrato: inicialmente preservam a escolha do primeiro elemento.
// Os testes RED demonstram que duplicatas ainda não produzem erro.
fn unidade_html(dom: &tl::VDom<'_>) -> Result<Option<String>, ErroCaixa> {
    Ok(unidade_html_original(dom))
}

fn valor_oculto(dom: &tl::VDom<'_>, id: &str) -> Result<Option<String>, ErroCaixa> {
    Ok(valor_oculto_original(dom, id))
}
