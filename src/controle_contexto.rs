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

/// Verificador leve mantido para testes da identidade, sem uso no caminho de produção.
#[cfg(test)]
fn unidade_exibida_no_controle(html: &str) -> Result<String, ErroCaixa> {
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
// O primeiro ID nunca comprova unicidade. A única exceção de layout
// é o par móvel/desktop demonstrado pela cassette UnB e02dbcfde8c4.
fn id_repetido(dom: &tl::VDom<'_>, id: &str) -> bool {
    dom.query_selector(&format!("#{id}"))
        .is_some_and(|mut encontrados| encontrados.next().is_some() && encontrados.next().is_some())
}

fn unidade_html(dom: &tl::VDom<'_>) -> Result<Option<String>, ErroCaixa> {
    if id_repetido(dom, "lnkInfraUnidade") && !par_de_barras_equivalentes(dom) {
        return Err(ErroCaixa::IdentificacoesDaUnidadeRepetidasNoControle);
    }
    Ok(unidade_html_original(dom))
}

fn par_de_barras_equivalentes(dom: &tl::VDom<'_>) -> bool {
    let parser = dom.parser();
    let Some(todos) = dom.query_selector("#lnkInfraUnidade") else {
        return false;
    };
    if todos.count() != 2 {
        return false;
    }
    let mut links = Vec::new();
    for id in ["divInfraBarraSistemaMovel", "divInfraBarraSistemaPadrao"] {
        if id_repetido(dom, id) {
            return false;
        }
        let Some(barra) = dom
            .get_element_by_id(id)
            .and_then(|n| n.get(parser))
            .and_then(tl::Node::as_tag)
        else {
            return false;
        };
        let Some(mut achados) = barra.query_selector(parser, "#lnkInfraUnidade") else {
            return false;
        };
        let Some(link) = achados.next() else {
            return false;
        };
        // tl mantém a barra desktop dentro da móvel neste HTML observado;
        // ambas as ocorrências precisam pertencer ao par, sem terceiro link.
        if let Some(outro) = achados.next() {
            if id != "divInfraBarraSistemaMovel" || achados.next().is_some() {
                return false;
            }
            if !links.contains(&outro) {
                links.push(outro);
            }
        }
        if !links.contains(&link) {
            links.push(link);
        }
    }
    if links.len() != 2 {
        return false;
    }
    let tags: Vec<_> = links
        .iter()
        .filter_map(|n| n.get(parser))
        .filter_map(tl::Node::as_tag)
        .collect();
    tags.len() == 2
        && tags.iter().all(|tag| tag.name() == "a")
        && tags[0].inner_text(parser) == tags[1].inner_text(parser)
        && tags[0].attributes().get("href").flatten().is_some()
        && tags[0].attributes().get("href") == tags[1].attributes().get("href")
}

fn valor_oculto(dom: &tl::VDom<'_>, id: &str) -> Result<Option<String>, ErroCaixa> {
    if id_repetido(dom, id) {
        return Err(match id {
            "hdnTipoVisualizacao" => ErroCaixa::TiposDeVisualizacaoRepetidosNoControle,
            "hdnRecebidosHashCriterios" => ErroCaixa::HashesDeCriteriosRecebidosRepetidosNoControle,
            "hdnGeradosHashCriterios" => ErroCaixa::HashesDeCriteriosGeradosRepetidosNoControle,
            _ => ErroCaixa::ContextoNaoVerificavel,
        });
    }
    Ok(valor_oculto_original(dom, id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controle() -> String {
        "<a id='lnkInfraUnidade'>UNIDADE A</a>\
         <input id='hdnTipoVisualizacao' value='R'>\
         <input id='hdnRecebidosHashCriterios' value='recebidos'>\
         <input id='hdnGeradosHashCriterios' value='gerados'>"
            .to_owned()
    }

    #[test]
    fn unidade_repetida_na_mesma_tela_nao_pode_virar_uma_identidade_falsa() {
        let html = format!("{}<a id='lnkInfraUnidade'>UNIDADE B</a>", controle());
        assert_eq!(
            unidade_exibida_no_controle(&html),
            Err(ErroCaixa::IdentificacoesDaUnidadeRepetidasNoControle)
        );
        assert_eq!(
            contexto_da_pagina(&html).unwrap_err(),
            ErroCaixa::IdentificacoesDaUnidadeRepetidasNoControle
        );
    }

    #[test]
    fn tipo_de_visualizacao_repetido_nao_prova_mesma_paginacao() {
        let html = format!("{}<input id='hdnTipoVisualizacao' value='D'>", controle());
        assert_eq!(
            contexto_da_pagina(&html).unwrap_err(),
            ErroCaixa::TiposDeVisualizacaoRepetidosNoControle
        );
    }

    #[test]
    fn hashes_de_criterios_repetidos_nao_selecionam_o_primeiro() {
        for (id, esperado) in [
            (
                "hdnRecebidosHashCriterios",
                ErroCaixa::HashesDeCriteriosRecebidosRepetidosNoControle,
            ),
            (
                "hdnGeradosHashCriterios",
                ErroCaixa::HashesDeCriteriosGeradosRepetidosNoControle,
            ),
        ] {
            let html = format!("{}<input id='{id}' value='CRITERIO ALTERADO'>", controle());
            assert_eq!(contexto_da_pagina(&html).unwrap_err(), esperado);
            assert_ne!(esperado.subcodigo(), "contexto_divergente");
        }
    }

    #[test]
    fn controle_com_identificadores_unicos_preserva_o_contexto_original() {
        let ctx = contexto_da_pagina(&controle()).unwrap();
        assert_eq!(ctx.unidade.as_deref(), Some("UNIDADE A"));
        assert!(ctx.verificavel());
    }

    fn barras(movel: &str, padrao: &str, destino: &str) -> String {
        format!(
            "<div id='divInfraBarraSistemaMovel'><a id='lnkInfraUnidade' href='#'>{movel}</a></div>\
             <div id='divInfraBarraSistemaPadrao'><a id='lnkInfraUnidade' href='{destino}'>{padrao}</a></div>"
        )
    }

    #[test]
    fn unidade_nas_barras_movel_e_desktop_observadas_na_unb() {
        assert_eq!(
            unidade_exibida_no_controle(&barras("Unidade4", "Unidade4", "#")),
            Ok("Unidade4".into())
        );
        let nested = barras("Unidade4", "Unidade4", "#").replacen("</div>", "", 1) + "</div>";
        assert_eq!(unidade_exibida_no_controle(&nested), Ok("Unidade4".into()));
    }

    #[test]
    fn barras_com_unidades_ou_destinos_distintos_continuam_recusadas() {
        for html in [
            barras("A", "B", "#"),
            barras("A", "A", "#outro"),
            "<a id='lnkInfraUnidade'>A</a><a id='lnkInfraUnidade'>A</a>".into(),
            format!("{}<a id='lnkInfraUnidade'>A</a>", barras("A", "A", "#")),
        ] {
            assert_eq!(
                unidade_exibida_no_controle(&html),
                Err(ErroCaixa::IdentificacoesDaUnidadeRepetidasNoControle)
            );
        }
    }
}
