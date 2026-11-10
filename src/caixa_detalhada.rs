//! Leitura da visão detalhada já retornada pelo Controle Web.
//! Não executa paginação POST, comandos JS nem atribui categorias fictícias.
mod celula_marcadores;
mod colunas;

use crate::controle_contexto::{ContextoPagina, contexto_da_pagina};
use crate::dominio::{Categoria, IdProcedimento};
use crate::erro_html::ErroHtml;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::nivel_de_detalhe::ColunaDoDetalhe;
use crate::processo::NumeroProcesso;
pub(crate) use colunas::ColunasDaLinha;
#[cfg(test)]
pub(crate) use colunas::ValorDaColuna;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ProcessoDetalhado {
    pub(crate) id: IdProcedimento,
    pub(crate) numero: String,
    pub(crate) visualizado: Option<bool>,
    pub(crate) sigiloso: Option<bool>,
    /// Categoria observada nas tabelas R do mesmo GET; `null` = não observada.
    #[serde(serialize_with = "colunas::serializar_categoria")]
    pub(crate) categoria: Option<Categoria>,
    /// Colunas configuradas do cabeçalho, por chave estável.
    pub(crate) colunas: ColunasDaLinha,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LeituraDetalhada {
    pub(crate) unidade: String,
    pub(crate) total_declarado: usize,
    /// Posições lidas (`inicio`, `fim`); `None` sem linhas.
    pub(crate) faixa_observada: Option<(usize, usize)>,
    pub(crate) processos: Vec<ProcessoDetalhado>,
    /// Colunas configuradas presentes no cabeçalho, na ordem da tabela.
    pub(crate) colunas: Vec<ColunaDoDetalhe>,
}

impl LeituraDetalhada {
    /// Chaves das nove colunas de texto do catálogo que não vieram no
    /// cabeçalho. Sem o GET da configuração, não diz se a opção existe na
    /// instalação (Prioridade falta na `UnB` 4.0.9); `ies unidade colunas` diz.
    pub(crate) fn colunas_ausentes(&self) -> Vec<&'static str> {
        ColunaDoDetalhe::TODAS
            .into_iter()
            .filter(|c| c.tem_cabecalho_de_texto() && !self.colunas.contains(c))
            .map(ColunaDoDetalhe::chave)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroDetalhada {
    VisualizacaoIncorreta,
    UnidadeNaoIdentificada,
    UnidadeDiferente,
    ContextoDiferente,
    ColunasDaLinhaDiferentes,
    ClassesDeEstadoDiferentesNoLink,
    TotalNaoDeclarado,
    FaixaSemIntervaloComPaginacao,
    FaixaDescontinua,
    TotalAlterado,
    LimitePaginas,
    RotuloDaCelulaDiferenteDoCabecalho,
    ItensDaCelulaNaoSeparados,
    CabecalhoDiferenteEntrePaginas,
    TextoForaDosMarcadoresNaCelula,
    #[causa(transparente)]
    Html(ErroHtml),
}

struct Pagina {
    contexto: ContextoPagina,
    total: usize,
    inicio: usize,
    fim: usize,
    processos: Vec<ProcessoDetalhado>,
    /// `href` bruto do link de cada processo, na mesma ordem.
    links: Vec<String>,
    colunas: Vec<ColunaDoDetalhe>,
}

fn texto(no: &tl::Node<'_>, parser: &tl::Parser<'_>) -> String {
    no.inner_text(parser)
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `hdnDetalhadoPaginaAtual` com índice diferente de `0`. O índice `0`
/// sozinho não é paginação: a UFRJ 4.1.5 o publica numa D de uma página,
/// sem paginador (04/11/2026), como `hdn{T}PaginaAtual` na R.
fn indice_oculto_alem_da_primeira(
    dom: &tl::VDom<'_>,
    parser: &tl::Parser<'_>,
) -> Result<bool, ErroDetalhada> {
    let nos = nos_com_id(dom, "hdnDetalhadoPaginaAtual")
        .map_err(|e| ErroHtml::atributo("html_detalhado_nao_interpretado", e))?;
    for no in nos {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            return Ok(true);
        };
        let valor = valor_do_atributo(tag, "value")
            .map_err(|e| ErroHtml::atributo("html_detalhado_nao_interpretado", e))?;
        if valor.as_deref() != Some("0") {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Legenda da lista detalhada, sem supor que o total é igual à página.
fn faixa(
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
    let (inicio, fim) = intervalo
        .split_once(" a ")
        .ok_or(ErroHtml::valor("faixa_detalhada_invalida"))?;
    let inicio = inicio
        .parse::<usize>()
        .map_err(|_| ErroHtml::valor("faixa_detalhada_invalida"))?;
    let fim = fim
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse::<usize>()
        .map_err(|_| ErroHtml::valor("faixa_detalhada_invalida"))?;
    if inicio == 0 || inicio > fim || fim > total || fim - inicio + 1 != linhas {
        return Err(ErroHtml::valor("faixa_detalhada_invalida").into());
    }
    Ok((total, inicio, fim))
}

fn eh_link_do_processo(href: &str, id: &str) -> bool {
    let href = href.replace("&amp;", "&");
    let Some(query) = href.strip_prefix("controlador.php?") else {
        return false;
    };
    let campos: Vec<_> = query.split('&').collect();
    let acoes: Vec<_> = campos
        .iter()
        .filter_map(|s| s.strip_prefix("acao="))
        .collect();
    let ids: Vec<_> = campos
        .iter()
        .filter_map(|s| s.strip_prefix("id_procedimento="))
        .collect();
    acoes.len() == 1
        && matches!(
            acoes[0],
            "procedimento_trabalhar" | "procedimento_visualizar"
        )
        && ids == [id]
}

/// Lido da fonte da tag; tag ilegível não vira atributo ausente.
fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, ErroDetalhada> {
    valor_do_atributo(tag, nome)
        .map_err(|e| ErroDetalhada::Html(ErroHtml::atributo("html_detalhado_nao_interpretado", e)))
}

fn ler_linha(
    dom: &tl::VDom<'_>,
    tag: &tl::HTMLTag<'_>,
    papeis: &[colunas::PapelDaColuna],
) -> Result<Option<(ProcessoDetalhado, String)>, ErroDetalhada> {
    let parser = dom.parser();
    let Some(id) = atributo(tag, "id")? else {
        return Ok(None);
    };
    let Some(id) = id.strip_prefix('P') else {
        return Ok(None);
    };
    let id = IdProcedimento::novo(id)
        .map_err(|_| ErroHtml::valor("identificador_detalhado_invalido"))?;
    let celulas: Vec<_> = tag
        .children()
        .top()
        .iter()
        .filter_map(|h| h.get(parser))
        .filter_map(tl::Node::as_tag)
        .filter(|t| t.name() == "td")
        .collect();
    if celulas.len() != papeis.len() {
        return Err(ErroDetalhada::ColunasDaLinhaDiferentes);
    }
    let mut configuradas = Vec::new();
    let mut celula = None;
    for (papel, td) in papeis.iter().zip(&celulas) {
        match papel {
            colunas::PapelDaColuna::Processo => celula = Some(*td),
            colunas::PapelDaColuna::Configurada(coluna) => {
                configuradas.push((
                    *coluna,
                    colunas::valor_da_celula(dom, td, *coluna, id.as_str())?,
                ));
            }
            colunas::PapelDaColuna::SemRotulo => {}
        }
    }
    let celula = celula.ok_or(ErroHtml::ausente("cabecalho_processo_detalhado_ausente"))?;
    let links = celula
        .query_selector(parser, "a")
        .ok_or(ErroHtml::ausente("link_do_processo_detalhado_ausente"))?;
    let mut escolhido = None;
    for link in links {
        let Some(link) = link.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(href) =
            atributo(link, "href")?.filter(|href| eh_link_do_processo(href, id.as_str()))
        else {
            continue;
        };
        if escolhido.is_some() {
            return Err(ErroHtml::repetido("links_do_processo_detalhado_repetidos").into());
        }
        let classe = atributo(link, "class")?.unwrap_or_default();
        let estado = crate::classe_link_processo::estado_do_link(&classe)
            .map_err(|_| ErroDetalhada::ClassesDeEstadoDiferentesNoLink)?;
        let numero = NumeroProcesso::novo(&link.inner_text(parser))
            .map_err(|_| ErroHtml::ausente("numero_do_processo_detalhado_ausente"))?;
        escolhido = Some((
            ProcessoDetalhado {
                id: id.clone(),
                numero: numero.as_str().to_owned(),
                visualizado: estado.map(|e| e.visualizado),
                sigiloso: estado.map(|e| e.sigiloso),
                categoria: None,
                colunas: ColunasDaLinha(configuradas.clone()),
            },
            href,
        ));
    }
    escolhido
        .map(Some)
        .ok_or(ErroDetalhada::Html(ErroHtml::ausente(
            "link_do_processo_detalhado_ausente",
        )))
}

fn ler_pagina(html: &str) -> Result<Pagina, ErroDetalhada> {
    let contexto =
        contexto_da_pagina(html).map_err(|_| ErroHtml::html("html_detalhado_nao_interpretado"))?;
    if !contexto.eh_detalhada() {
        return Err(ErroDetalhada::VisualizacaoIncorreta);
    }
    if contexto.unidade.is_none() {
        return Err(ErroDetalhada::UnidadeNaoIdentificada);
    }
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("html_detalhado_nao_interpretado"))?;
    let parser = dom.parser();
    let tabelas = nos_com_id(&dom, "tblProcessosDetalhado")
        .map_err(|e| ErroHtml::atributo("html_detalhado_nao_interpretado", e))?;
    let tabela = match tabelas.as_slice() {
        [] => return Err(ErroHtml::ausente("tabela_detalhada_ausente").into()),
        [tabela] => *tabela,
        _ => return Err(ErroHtml::repetido("tabelas_detalhadas_repetidas").into()),
    };
    let tabela = tabela
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroHtml::ausente("tabela_detalhada_ausente"))?;
    if tabela.name() != "table"
        || tabela
            .query_selector(parser, "table")
            .is_some_and(|mut itens| itens.next().is_some())
    {
        return Err(ErroHtml::ausente("tabela_detalhada_ausente").into());
    }
    let mut legendas = tabela
        .query_selector(parser, "caption")
        .ok_or(ErroHtml::ausente("legenda_detalhada_ausente"))?;
    let legenda = legendas
        .next()
        .ok_or(ErroHtml::ausente("legenda_detalhada_ausente"))?;
    if legendas.next().is_some() {
        return Err(ErroHtml::repetido("legendas_detalhadas_repetidas").into());
    }
    let legenda = legenda
        .get(parser)
        .ok_or(ErroHtml::ausente("legenda_detalhada_ausente"))?;
    let papeis = colunas::papeis_do_cabecalho(&dom, tabela)?;
    let configuradas: Vec<ColunaDoDetalhe> = papeis
        .iter()
        .filter_map(|papel| match papel {
            colunas::PapelDaColuna::Configurada(coluna) => Some(*coluna),
            _ => None,
        })
        .collect();
    let mut processos = Vec::new();
    let mut links = Vec::new();
    if let Some(linhas) = tabela.query_selector(parser, "tr") {
        for linha in linhas {
            let tag = linha
                .get(parser)
                .and_then(tl::Node::as_tag)
                .ok_or(ErroDetalhada::ColunasDaLinhaDiferentes)?;
            if let Some((processo, href)) = ler_linha(&dom, tag, &papeis)? {
                processos.push(processo);
                links.push(href);
            }
        }
    }
    // Marcadores de paginação sem faixa explícita não demonstram cobertura.
    let paginacao_visivel = indice_oculto_alem_da_primeira(&dom, parser)?
        || crate::paginacao_controle::controles_de_paginacao(html)
            .map_err(|_| ErroHtml::html("html_detalhado_nao_interpretado"))?
        || crate::paginacao_controle::paginador_com_prefixo(html, "Detalhado")
            .map_err(|_| ErroHtml::html("html_detalhado_nao_interpretado"))?;
    let (total, inicio, fim) = faixa(&texto(legenda, parser), processos.len(), paginacao_visivel)?;
    Ok(Pagina {
        contexto,
        total,
        inicio,
        fim,
        processos,
        links,
        colunas: configuradas,
    })
}

/// As linhas de uma página D com o `href` do link do processo, pela mesma
/// leitura e com as mesmas recusas da página em [`ler_paginas`].
pub(crate) fn linhas_com_link(
    html: &str,
) -> Result<Vec<crate::caixa::LinhaDoControle>, ErroDetalhada> {
    let pagina = ler_pagina(html)?;
    Ok(pagina
        .processos
        .into_iter()
        .zip(pagina.links)
        .map(|(processo, href)| crate::caixa::LinhaDoControle {
            id: processo.id.as_str().to_owned(),
            numero: processo.numero,
            href: Some(href),
        })
        .collect())
}

/// Consome páginas de uma mesma sessão sem inventar uma cobertura transacional.
pub(crate) fn ler_paginas(
    paginas: &[String],
    unidade_esperada: Option<&str>,
) -> Result<LeituraDetalhada, ErroDetalhada> {
    if paginas.is_empty() {
        return Err(ErroHtml::ausente("tabela_detalhada_ausente").into());
    }
    if paginas.iter().map(String::len).sum::<usize>() > 48_000_000 {
        return Err(ErroDetalhada::LimitePaginas);
    }
    let mut contexto = None;
    let mut total = None;
    let mut proximo = 1usize;
    let mut faixa_inicio = None;
    let mut processos = Vec::new();
    let mut ids = HashSet::new();
    let mut numeros = HashSet::new();
    let mut colunas_vistas: Option<Vec<ColunaDoDetalhe>> = None;
    for html in paginas {
        let pagina = ler_pagina(html)?;
        if let Some(ref inicial) = contexto {
            if inicial != &pagina.contexto {
                return Err(ErroDetalhada::ContextoDiferente);
            }
        } else {
            if unidade_esperada.is_some_and(|v| pagina.contexto.unidade.as_deref() != Some(v)) {
                return Err(ErroDetalhada::UnidadeDiferente);
            }
            contexto = Some(pagina.contexto.clone());
        }
        if total.is_some_and(|v| v != pagina.total) {
            return Err(ErroDetalhada::TotalAlterado);
        }
        total = Some(pagina.total);
        if colunas_vistas
            .as_ref()
            .is_some_and(|vistas| *vistas != pagina.colunas)
        {
            return Err(ErroDetalhada::CabecalhoDiferenteEntrePaginas);
        }
        colunas_vistas = Some(pagina.colunas.clone());
        // A primeira página pode não ser a 1 (página lembrada na sessão);
        // as seguintes continuam a anterior.
        if faixa_inicio.is_none() {
            faixa_inicio = Some(pagina.inicio);
            proximo = pagina.inicio;
        }
        if pagina.inicio != proximo {
            return Err(ErroDetalhada::FaixaDescontinua);
        }
        proximo = pagina.fim.saturating_add(1);
        for processo in pagina.processos {
            if !ids.insert(processo.id.clone()) {
                return Err(ErroHtml::repetido("identificador_detalhado_repetido").into());
            }
            if !numeros.insert(processo.numero.clone()) {
                return Err(ErroHtml::repetido("numero_detalhado_repetido").into());
            }
            processos.push(processo);
        }
    }
    let total = total.ok_or(ErroDetalhada::TotalNaoDeclarado)?;
    let faixa_observada = faixa_inicio
        .filter(|_| !processos.is_empty())
        .map(|inicio| (inicio, proximo - 1));
    let unidade = contexto
        .and_then(|v: ContextoPagina| v.unidade)
        .ok_or(ErroDetalhada::UnidadeNaoIdentificada)?;
    Ok(LeituraDetalhada {
        unidade,
        total_declarado: total,
        faixa_observada,
        processos,
        colunas: colunas_vistas.unwrap_or_default(),
    })
}

#[cfg(test)]
pub(crate) fn possui_tabela(html: &str) -> bool {
    tl::parse(html, tl::ParserOptions::default()).is_ok_and(|dom| {
        // Tag ilegível conta como possível tabela: a leitura falhará tipada.
        !nos_com_id(&dom, "tblProcessosDetalhado").is_ok_and(|tabelas| tabelas.is_empty())
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_colunas;

#[cfg(test)]
pub(crate) mod tests_marcadores;
