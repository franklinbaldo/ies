//! Leitura da visão detalhada já retornada pelo Controle Web.
//! Não executa paginação POST, comandos JS nem atribui categorias fictícias.
use crate::controle_contexto::{ContextoPagina, contexto_da_pagina};
use crate::dominio::IdProcedimento;
use crate::processo::NumeroProcesso;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ProcessoDetalhado {
    pub(crate) id: IdProcedimento,
    pub(crate) numero: String,
    pub(crate) visualizado: Option<bool>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LeituraDetalhada {
    pub(crate) unidade: String,
    pub(crate) total_declarado: usize,
    pub(crate) processos: Vec<ProcessoDetalhado>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErroDetalhada {
    HtmlNaoInterpretado,
    TabelaAusente,
    TabelasRepetidas,
    VisualizacaoIncorreta,
    UnidadeNaoIdentificada,
    UnidadeDiferente,
    ContextoDiferente,
    CabecalhoProcessoAusente,
    CabecalhoProcessoRepetido,
    ColunasDaLinhaDiferentes,
    LinkDoProcessoAusente,
    LinksDoProcessoRepetidos,
    NumeroDoProcessoAusente,
    IdentificadorInvalido,
    IdentificadorRepetido,
    NumeroRepetido,
    LegendaAusente,
    LegendasRepetidas,
    TotalNaoDeclarado,
    FaixaInvalida,
    FaixaDescontinua,
    TotalAlterado,
    PaginasNaoObtidas,
    LimitePaginas,
}

impl ErroDetalhada {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlNaoInterpretado => "html_detalhado_nao_interpretado",
            Self::TabelaAusente => "tabela_detalhada_ausente",
            Self::TabelasRepetidas => "tabelas_detalhadas_repetidas",
            Self::VisualizacaoIncorreta => "modo_detalhado_nao_confirmado",
            Self::UnidadeNaoIdentificada => "unidade_da_detalhada_ausente",
            Self::UnidadeDiferente => "unidade_da_detalhada_diferente_da_solicitada",
            Self::ContextoDiferente => "contexto_detalhado_alterado",
            Self::CabecalhoProcessoAusente => "cabecalho_processo_detalhado_ausente",
            Self::CabecalhoProcessoRepetido => "cabecalho_processo_detalhado_repetido",
            Self::ColunasDaLinhaDiferentes => "colunas_da_linha_detalhada_diferentes",
            Self::LinkDoProcessoAusente => "link_do_processo_detalhado_ausente",
            Self::LinksDoProcessoRepetidos => "links_do_processo_detalhado_repetidos",
            Self::NumeroDoProcessoAusente => "numero_do_processo_detalhado_ausente",
            Self::IdentificadorInvalido => "identificador_detalhado_invalido",
            Self::IdentificadorRepetido => "identificador_detalhado_repetido",
            Self::NumeroRepetido => "numero_detalhado_repetido",
            Self::LegendaAusente => "legenda_detalhada_ausente",
            Self::LegendasRepetidas => "legendas_detalhadas_repetidas",
            Self::TotalNaoDeclarado => "total_detalhado_nao_declarado",
            Self::FaixaInvalida => "faixa_detalhada_invalida",
            Self::FaixaDescontinua => "faixa_detalhada_descontinua",
            Self::TotalAlterado => "total_detalhado_alterado",
            Self::PaginasNaoObtidas => "paginas_detalhadas_nao_obtidas",
            Self::LimitePaginas => "limite_paginas_detalhadas",
        }
    }
}

#[derive(Debug)]
struct Pagina {
    contexto: ContextoPagina,
    total: usize,
    inicio: usize,
    fim: usize,
    processos: Vec<ProcessoDetalhado>,
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

/// Legenda da lista detalhada, sem supor que o total é igual à página.
fn faixa(legenda: &str, linhas: usize) -> Result<(usize, usize, usize), ErroDetalhada> {
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
        return Ok((total, 1, linhas));
    };
    let (inicio, fim) = intervalo
        .split_once(" a ")
        .ok_or(ErroDetalhada::FaixaInvalida)?;
    let inicio = inicio
        .trim()
        .parse::<usize>()
        .map_err(|_| ErroDetalhada::FaixaInvalida)?;
    let fim = fim
        .trim()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse::<usize>()
        .map_err(|_| ErroDetalhada::FaixaInvalida)?;
    if inicio == 0 || inicio > fim || fim > total || fim - inicio + 1 != linhas {
        return Err(ErroDetalhada::FaixaInvalida);
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

fn ler_linha(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    coluna: usize,
    total_colunas: usize,
) -> Result<Option<ProcessoDetalhado>, ErroDetalhada> {
    let Some(raw) = tag.attributes().get("id").flatten() else {
        return Ok(None);
    };
    let id = raw.as_utf8_str();
    let Some(id) = id.strip_prefix('P') else {
        return Ok(None);
    };
    let id = IdProcedimento::novo(id).map_err(|_| ErroDetalhada::IdentificadorInvalido)?;
    let celulas: Vec<_> = tag
        .children()
        .top()
        .iter()
        .filter_map(|h| h.get(parser))
        .filter_map(tl::Node::as_tag)
        .filter(|t| t.name() == "td")
        .collect();
    if celulas.len() != total_colunas {
        return Err(ErroDetalhada::ColunasDaLinhaDiferentes);
    }
    let celula = celulas[coluna];
    let links = celula
        .query_selector(parser, "a")
        .ok_or(ErroDetalhada::LinkDoProcessoAusente)?;
    let mut escolhido = None;
    for link in links {
        let Some(link) = link.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let href = link.attributes().get("href").flatten();
        if href.is_none_or(|href| !eh_link_do_processo(&href.as_utf8_str(), id.as_str())) {
            continue;
        }
        if escolhido.is_some() {
            return Err(ErroDetalhada::LinksDoProcessoRepetidos);
        }
        let classe = link
            .attributes()
            .get("class")
            .flatten()
            .map(|v| v.as_utf8_str().into_owned())
            .unwrap_or_default();
        let visualizado = if classe
            .split_whitespace()
            .any(|c| c == "processoNaoVisualizado")
        {
            Some(false)
        } else if classe
            .split_whitespace()
            .any(|c| c == "processoVisualizado")
        {
            Some(true)
        } else {
            None
        };
        let numero = NumeroProcesso::novo(&link.inner_text(parser))
            .map_err(|_| ErroDetalhada::NumeroDoProcessoAusente)?;
        escolhido = Some(ProcessoDetalhado {
            id: id.clone(),
            numero: numero.as_str().to_owned(),
            visualizado,
        });
    }
    escolhido
        .map(Some)
        .ok_or(ErroDetalhada::LinkDoProcessoAusente)
}

fn ler_pagina(html: &str) -> Result<Pagina, ErroDetalhada> {
    let contexto = contexto_da_pagina(html).map_err(|_| ErroDetalhada::HtmlNaoInterpretado)?;
    if !contexto.eh_detalhada() {
        return Err(ErroDetalhada::VisualizacaoIncorreta);
    }
    if contexto.unidade.is_none() {
        return Err(ErroDetalhada::UnidadeNaoIdentificada);
    }
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroDetalhada::HtmlNaoInterpretado)?;
    let parser = dom.parser();
    let mut tabelas = dom
        .query_selector("#tblProcessosDetalhado")
        .ok_or(ErroDetalhada::TabelaAusente)?;
    let tabela = tabelas.next().ok_or(ErroDetalhada::TabelaAusente)?;
    if tabelas.next().is_some() {
        return Err(ErroDetalhada::TabelasRepetidas);
    }
    let tabela = tabela
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroDetalhada::TabelaAusente)?;
    if tabela.name() != "table"
        || tabela
            .query_selector(parser, "table")
            .is_some_and(|mut itens| itens.next().is_some())
    {
        return Err(ErroDetalhada::TabelaAusente);
    }
    let mut legendas = tabela
        .query_selector(parser, "caption")
        .ok_or(ErroDetalhada::LegendaAusente)?;
    let legenda = legendas.next().ok_or(ErroDetalhada::LegendaAusente)?;
    if legendas.next().is_some() {
        return Err(ErroDetalhada::LegendasRepetidas);
    }
    let legenda = legenda.get(parser).ok_or(ErroDetalhada::LegendaAusente)?;
    let mut coluna = None;
    let mut colunas = 0usize;
    let ths = tabela
        .query_selector(parser, "th")
        .ok_or(ErroDetalhada::CabecalhoProcessoAusente)?;
    for th in ths {
        let th = th
            .get(parser)
            .ok_or(ErroDetalhada::CabecalhoProcessoAusente)?;
        if texto(th, parser) == "Processo" && coluna.replace(colunas).is_some() {
            return Err(ErroDetalhada::CabecalhoProcessoRepetido);
        }
        colunas += 1;
    }
    let coluna = coluna.ok_or(ErroDetalhada::CabecalhoProcessoAusente)?;
    let mut processos = Vec::new();
    if let Some(linhas) = tabela.query_selector(parser, "tr") {
        for linha in linhas {
            let tag = linha
                .get(parser)
                .and_then(tl::Node::as_tag)
                .ok_or(ErroDetalhada::ColunasDaLinhaDiferentes)?;
            if let Some(processo) = ler_linha(tag, parser, coluna, colunas)? {
                processos.push(processo);
            }
        }
    }
    let (total, inicio, fim) = faixa(&texto(legenda, parser), processos.len())?;
    Ok(Pagina {
        contexto,
        total,
        inicio,
        fim,
        processos,
    })
}

/// Consome páginas de uma mesma sessão sem inventar uma cobertura transacional.
pub(crate) fn ler_paginas(
    paginas: &[String],
    unidade_esperada: Option<&str>,
) -> Result<LeituraDetalhada, ErroDetalhada> {
    if paginas.is_empty() {
        return Err(ErroDetalhada::TabelaAusente);
    }
    if paginas.len() > 200 || paginas.iter().map(String::len).sum::<usize>() > 48_000_000 {
        return Err(ErroDetalhada::LimitePaginas);
    }
    let mut contexto = None;
    let mut total = None;
    let mut proximo = 1usize;
    let mut processos = Vec::new();
    let mut ids = HashSet::new();
    let mut numeros = HashSet::new();
    for html in paginas {
        let pagina = ler_pagina(html)?;
        if let Some(ref inicial) = contexto {
            if inicial != &pagina.contexto {
                return Err(ErroDetalhada::ContextoDiferente);
            }
        } else {
            if unidade_esperada
                .is_some_and(|v| pagina.contexto.unidade.as_deref() != Some(v.trim()))
            {
                return Err(ErroDetalhada::UnidadeDiferente);
            }
            contexto = Some(pagina.contexto.clone());
        }
        if total.is_some_and(|v| v != pagina.total) {
            return Err(ErroDetalhada::TotalAlterado);
        }
        total = Some(pagina.total);
        if pagina.inicio != proximo {
            return Err(ErroDetalhada::FaixaDescontinua);
        }
        proximo = pagina.fim.saturating_add(1);
        for processo in pagina.processos {
            if !ids.insert(processo.id.clone()) {
                return Err(ErroDetalhada::IdentificadorRepetido);
            }
            if !numeros.insert(processo.numero.clone()) {
                return Err(ErroDetalhada::NumeroRepetido);
            }
            processos.push(processo);
        }
    }
    let total = total.ok_or(ErroDetalhada::TotalNaoDeclarado)?;
    if processos.len() != total || proximo != total.saturating_add(1) {
        return Err(ErroDetalhada::PaginasNaoObtidas);
    }
    let unidade = contexto
        .and_then(|v: ContextoPagina| v.unidade)
        .ok_or(ErroDetalhada::UnidadeNaoIdentificada)?;
    Ok(LeituraDetalhada {
        unidade,
        total_declarado: total,
        processos,
    })
}

pub(crate) fn possui_tabela(html: &str) -> bool {
    tl::parse(html, tl::ParserOptions::default()).is_ok_and(|dom| {
        dom.query_selector("#tblProcessosDetalhado")
            .is_some_and(|mut tabelas| tabelas.next().is_some())
    })
}

#[cfg(test)]
mod tests;
