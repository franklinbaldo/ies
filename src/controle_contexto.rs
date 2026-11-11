//! Identidade e critérios Web do Controle de Processos. Sem rede ou persistência.
use crate::erro_web::ErroCaixa;
use crate::html_atributos::{nos_com_id, nos_com_id_em, valor_do_atributo};
/// Contexto da página HTML: só é usado para conferir a junção.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextoPagina {
    pub(crate) unidade: Option<String>,
    modo: Option<String>,
    criterios_recebidos: Option<String>,
    criterios_gerados: Option<String>,
}

/// Primeiro campo de contexto que difere entre duas páginas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiferencaDeContexto {
    Unidade,
    Visualizacao,
    CriteriosRecebidos,
    CriteriosGerados,
}

impl ContextoPagina {
    pub(crate) fn diferenca(&self, outro: &Self) -> Option<DiferencaDeContexto> {
        if self.unidade != outro.unidade {
            Some(DiferencaDeContexto::Unidade)
        } else if self.modo != outro.modo {
            Some(DiferencaDeContexto::Visualizacao)
        } else if self.criterios_recebidos != outro.criterios_recebidos {
            Some(DiferencaDeContexto::CriteriosRecebidos)
        } else if self.criterios_gerados != outro.criterios_gerados {
            Some(DiferencaDeContexto::CriteriosGerados)
        } else {
            None
        }
    }

    pub(crate) fn eh_detalhada(&self) -> bool {
        self.modo.as_deref() == Some("D")
    }

    pub(crate) fn eh_resumida(&self) -> bool {
        self.modo.as_deref() == Some("R")
    }

    pub(crate) fn verificavel(&self) -> bool {
        self.unidade.is_some()
            && self.modo.is_some()
            && self.criterios_recebidos.is_some()
            && self.criterios_gerados.is_some()
    }
}

// `id`, `value` e `href` vêm da fonte da tag: o índice de `id` do tl 0.7.8
// usa a chave corrompida após atributo booleano sem valor.
fn primeiro_com_id<'d>(
    dom: &'d tl::VDom<'_>,
    id: &str,
) -> Result<Option<&'d tl::HTMLTag<'d>>, ErroCaixa> {
    let parser = dom.parser();
    Ok(nos_com_id(dom, id)
        .map_err(ErroCaixa::AtributoDaCaixaIlegivel)?
        .first()
        .and_then(|n| n.get(parser))
        .and_then(tl::Node::as_tag))
}

fn unidade_html_original(dom: &tl::VDom<'_>) -> Result<Option<String>, ErroCaixa> {
    let parser = dom.parser();
    let Some(link) = primeiro_com_id(dom, "lnkInfraUnidade")? else {
        return Ok(None);
    };
    if link.name() != "a" {
        return Ok(None);
    }
    let texto = link
        .inner_text(parser)
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace('\u{a0}', " ");
    let unidade = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    Ok((!unidade.is_empty()).then_some(unidade))
}

/// Verificador leve mantido para testes da identidade, sem uso no caminho de produção.
#[cfg(test)]
pub(crate) fn unidade_exibida_no_controle(html: &str) -> Result<String, ErroCaixa> {
    let servido = crate::tela_html::HtmlServido::de_teste(html);
    let dom = crate::tela_html::interpretar(&servido);
    unidade_html(&dom)?.ok_or(ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle)
}

fn valor_oculto_original(dom: &tl::VDom<'_>, id: &str) -> Result<Option<String>, ErroCaixa> {
    let Some(elemento) = primeiro_com_id(dom, id)? else {
        return Ok(None);
    };
    if elemento.name() != "input" {
        return Ok(None);
    }
    let Some(valor) =
        valor_do_atributo(elemento, "value").map_err(ErroCaixa::AtributoDaCaixaIlegivel)?
    else {
        return Ok(None);
    };
    Ok((!valor.is_empty()).then_some(valor))
}

pub(crate) fn contexto_da_pagina(
    html: &crate::tela_html::HtmlServido,
) -> Result<ContextoPagina, ErroCaixa> {
    let dom = crate::tela_html::interpretar(html);
    contexto_do_dom(&dom)
}

pub(crate) fn contexto_do_dom(dom: &tl::VDom<'_>) -> Result<ContextoPagina, ErroCaixa> {
    Ok(ContextoPagina {
        unidade: unidade_html(dom)?,
        modo: valor_oculto(dom, "hdnTipoVisualizacao")?,
        criterios_recebidos: valor_oculto(dom, "hdnRecebidosHashCriterios")?,
        criterios_gerados: valor_oculto(dom, "hdnGeradosHashCriterios")?,
    })
}
/// `hdnIdMarcador{id da unidade}` com valor: a visão R mostra só os
/// processos do marcador escolhido (POST de "Listar processos com o
/// marcador", nas sete instalações do acervo; vazio em todas as telas GET).
pub(crate) fn filtro_de_marcador_ativo(
    html: &crate::tela_html::HtmlServido,
) -> Result<bool, ErroCaixa> {
    let dom = crate::tela_html::interpretar(html);
    let parser = dom.parser();
    for no in dom.query_selector("input").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let id = valor_do_atributo(tag, "id").map_err(ErroCaixa::AtributoDaCaixaIlegivel)?;
        let do_marcador = id.as_deref().and_then(|i| i.strip_prefix("hdnIdMarcador"));
        if do_marcador
            .is_some_and(|resto| !resto.is_empty() && resto.bytes().all(|b| b.is_ascii_digit()))
            && valor_do_atributo(tag, "value")
                .map_err(ErroCaixa::AtributoDaCaixaIlegivel)?
                .is_some_and(|valor| !valor.is_empty())
        {
            return Ok(true);
        }
    }
    Ok(false)
}

// O primeiro ID nunca comprova unicidade. A única exceção de layout
// é o par móvel/desktop demonstrado pela cassette UnB e02dbcfde8c4.
fn id_repetido(dom: &tl::VDom<'_>, id: &str) -> Result<bool, ErroCaixa> {
    Ok(nos_com_id(dom, id)
        .map_err(ErroCaixa::AtributoDaCaixaIlegivel)?
        .len()
        >= 2)
}

/// Falha só por tag ilegível (`HtmlInvalido`) ou por identificações
/// repetidas fora do par móvel/desktop equivalente.
pub(crate) fn unidade_html(dom: &tl::VDom<'_>) -> Result<Option<String>, ErroCaixa> {
    if id_repetido(dom, "lnkInfraUnidade")? && !par_de_barras_equivalentes(dom)? {
        return Err(ErroCaixa::IdentificacoesDaUnidadeRepetidasNoControle);
    }
    unidade_html_original(dom)
}

fn par_de_barras_equivalentes(dom: &tl::VDom<'_>) -> Result<bool, ErroCaixa> {
    let parser = dom.parser();
    if nos_com_id(dom, "lnkInfraUnidade")
        .map_err(ErroCaixa::AtributoDaCaixaIlegivel)?
        .len()
        != 2
    {
        return Ok(false);
    }
    let mut links = Vec::new();
    for id in ["divInfraBarraSistemaMovel", "divInfraBarraSistemaPadrao"] {
        if id_repetido(dom, id)? {
            return Ok(false);
        }
        let Some(barra) = primeiro_com_id(dom, id)? else {
            return Ok(false);
        };
        let mut achados = nos_com_id_em(barra, parser, "lnkInfraUnidade")
            .map_err(ErroCaixa::AtributoDaCaixaIlegivel)?
            .into_iter();
        let Some(link) = achados.next() else {
            return Ok(false);
        };
        // tl mantém a barra desktop dentro da móvel neste HTML observado;
        // ambas as ocorrências precisam pertencer ao par, sem terceiro link.
        if let Some(outro) = achados.next() {
            if id != "divInfraBarraSistemaMovel" || achados.next().is_some() {
                return Ok(false);
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
        return Ok(false);
    }
    let tags: Vec<_> = links
        .iter()
        .filter_map(|n| n.get(parser))
        .filter_map(tl::Node::as_tag)
        .collect();
    let [primeira, segunda] = tags.as_slice() else {
        return Ok(false);
    };
    if !tags.iter().all(|tag| tag.name() == "a")
        || primeira.inner_text(parser) != segunda.inner_text(parser)
    {
        return Ok(false);
    }
    // `href` repetido numa das tags não comprova o par: não equivalente.
    let href = |tag: &tl::HTMLTag<'_>| valor_do_atributo(tag, "href").ok().flatten();
    Ok(href(primeira).is_some_and(|primeiro| href(segunda).is_some_and(|h| h == primeiro)))
}

fn valor_oculto(dom: &tl::VDom<'_>, id: &str) -> Result<Option<String>, ErroCaixa> {
    if id_repetido(dom, id)? {
        return Err(match id {
            "hdnTipoVisualizacao" => ErroCaixa::TiposDeVisualizacaoRepetidosNoControle,
            "hdnRecebidosHashCriterios" => ErroCaixa::HashesDeCriteriosRecebidosRepetidosNoControle,
            "hdnGeradosHashCriterios" => ErroCaixa::HashesDeCriteriosGeradosRepetidosNoControle,
            _ => ErroCaixa::ContextoNaoVerificavel,
        });
    }
    valor_oculto_original(dom, id)
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
    fn diferenca_aponta_o_primeiro_campo_alterado() {
        let base =
            contexto_da_pagina(&crate::tela_html::HtmlServido::de_teste(&controle())).unwrap();
        assert_eq!(base.diferenca(&base.clone()), None);
        for (de, para, esperada) in [
            ("UNIDADE A", "UNIDADE B", DiferencaDeContexto::Unidade),
            ("value='R'", "value='D'", DiferencaDeContexto::Visualizacao),
            (
                "'recebidos'",
                "'outro'",
                DiferencaDeContexto::CriteriosRecebidos,
            ),
            (
                "'gerados'",
                "'outro'",
                DiferencaDeContexto::CriteriosGerados,
            ),
        ] {
            let outra = contexto_da_pagina(&crate::tela_html::HtmlServido::de_teste(
                &controle().replace(de, para),
            ))
            .unwrap();
            assert_eq!(base.diferenca(&outra), Some(esperada), "{para}");
        }
    }

    #[test]
    fn booleano_antes_do_id_e_do_value_nao_apaga_o_contexto() {
        let html = crate::html_atributos::com_booleano_antes_dos_atributos(&controle());
        let contexto = contexto_da_pagina(&crate::tela_html::HtmlServido::de_teste(&html)).unwrap();
        assert_eq!(contexto.unidade.as_deref(), Some("UNIDADE A"));
        assert_eq!(contexto.modo.as_deref(), Some("R"));
        assert_eq!(contexto.criterios_recebidos.as_deref(), Some("recebidos"));
        assert_eq!(contexto.criterios_gerados.as_deref(), Some("gerados"));
        let repetido = crate::tela_html::HtmlServido::de_teste(&format!(
            "{html}<input disabled id='hdnTipoVisualizacao' value='D'>"
        ));
        assert_eq!(
            contexto_da_pagina(&repetido).unwrap_err(),
            ErroCaixa::TiposDeVisualizacaoRepetidosNoControle
        );
    }

    #[test]
    fn unidade_repetida_na_mesma_tela_nao_pode_virar_uma_identidade_falsa() {
        let html = format!("{}<a id='lnkInfraUnidade'>UNIDADE B</a>", controle());
        assert_eq!(
            unidade_exibida_no_controle(&html),
            Err(ErroCaixa::IdentificacoesDaUnidadeRepetidasNoControle)
        );
        assert_eq!(
            contexto_da_pagina(&crate::tela_html::HtmlServido::de_teste(&html)).unwrap_err(),
            ErroCaixa::IdentificacoesDaUnidadeRepetidasNoControle
        );
    }

    #[test]
    fn tipo_de_visualizacao_repetido_nao_prova_mesma_paginacao() {
        let html = crate::tela_html::HtmlServido::de_teste(&format!(
            "{}<input id='hdnTipoVisualizacao' value='D'>",
            controle()
        ));
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
            let html = crate::tela_html::HtmlServido::de_teste(&format!(
                "{}<input id='{id}' value='CRITERIO ALTERADO'>",
                controle()
            ));
            assert_eq!(contexto_da_pagina(&html).unwrap_err(), esperado);
        }
    }

    #[test]
    fn controle_com_identificadores_unicos_preserva_o_contexto_original() {
        let ctx =
            contexto_da_pagina(&crate::tela_html::HtmlServido::de_teste(&controle())).unwrap();
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
