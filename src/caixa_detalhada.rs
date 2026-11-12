//! Leitura da visão detalhada já retornada pelo Controle Web.
//! Não executa paginação POST, comandos JS nem atribui categorias fictícias.
mod celula_marcadores;
mod colunas;

use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{ConsultaDoControlador, menciona_acao};
use crate::controle_contexto::{ContextoPagina, contexto_da_pagina};
use crate::dominio::{Categoria, IdProcedimento};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::legenda::{ErroLegenda, Faixa, Legenda, ler_legenda};
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
    pub(crate) visualizado: bool,
    pub(crate) sigiloso: bool,
    /// Categoria das linhas, preenchida por `visao_do_controle` quando as
    /// tabelas R do mesmo GET a mostraram; `null` = não observada.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) categoria: Option<Categoria>,
    /// Colunas configuradas do cabeçalho, por chave estável.
    pub(crate) colunas: ColunasDaLinha,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LeituraDetalhada {
    pub(crate) unidade: String,
    pub(crate) total_declarado: usize,
    /// Posições lidas (`inicio`, `fim`); `SemLinhas` para zero registros.
    pub(crate) faixa_observada: crate::paginas::FaixaObservada,
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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDetalhada {
    VisualizacaoIncorreta,
    UnidadeNaoIdentificada,
    UnidadeDiferente,
    ContextoDiferente,
    ColunasDaLinhaDiferentes,
    ClassesDeEstadoDiferentesNoLink(crate::classe_link_processo::ClassesDeEstadoDiferentesNoLink),
    /// O link do processo não publica nenhuma classe de estado (visualizado/sigiloso).
    EstadoDoLinkDoProcessoAusente,
    TotalNaoDeclarado,
    FaixaSemIntervaloComPaginacao,
    FaixaDescontinua,
    TotalAlterado,
    LimitePaginas,
    RotuloDaCelulaDiferenteDoCabecalho,
    ItensDaCelulaNaoSeparados,
    CabecalhoDiferenteEntrePaginas,
    TextoForaDosMarcadoresNaCelula,
    CabecalhoDaVisaoDetalhadaNaoReconhecido,
    CabecalhoProcessoDetalhadoAusente,
    CabecalhoProcessoDetalhadoRepetido,
    ColunaDaVisaoDetalhadaRepetida,
    FaixaDetalhadaInvalida,
    AtributoDaDetalhadaIlegivel(#[from] crate::html_atributos::ErroAtributoHtml),
    /// A legenda não está em nenhuma forma observada (leitor `legenda`).
    Legenda(#[from] ErroLegenda),
    /// O controle da página recusou o HTML recebido.
    ControleDaPaginaNaoInterpretado(#[source] crate::erro_web::ErroCaixa),
    IdentificadorDetalhadoInvalido(#[source] crate::erro_web::ErroCaixa),
    /// A consulta do marcador da célula não se deixa ler em pares.
    ConsultaDoMarcadorIlegivel(#[source] crate::consulta_do_controlador::ErroConsultaDoControlador),
    /// `acao` ou `id_procedimento` do marcador ausente ou repetido.
    ChaveDoMarcadorDetalhado(#[source] crate::consulta_do_controlador::ErroChaveDaConsulta),
    /// A consulta do link do processo não se deixa ler em pares.
    ConsultaDoLinkDoProcessoIlegivel(
        #[source] crate::consulta_do_controlador::ErroConsultaDoControlador,
    ),
    /// `acao` ou `id_procedimento` do link do processo ausente ou repetido.
    ChaveDoLinkDoProcessoDetalhado(#[source] crate::consulta_do_controlador::ErroChaveDaConsulta),
    /// O link do processo não é `controlador.php` com a ação de trabalhar ou visualizar.
    LinkDoProcessoDetalhadoForaDaForma,
    /// O link do processo aponta para outro `id_procedimento` que o da linha.
    IdDoLinkDoProcessoDiferente,
    IdentificadorDetalhadoRepetido,
    ItensNaoPrevistosNaCelulaDetalhada,
    LegendaDetalhadaAusente,
    LegendasDetalhadasRepetidas,
    LinkDoProcessoDetalhadoAusente,
    LinksDoProcessoDetalhadoRepetidos,
    MarcadorDaCelulaDetalhadaForaDaFormaObservada,
    NumeroDetalhadoRepetido,
    NumeroDoProcessoDetalhadoAusente(#[source] crate::processo::NumeroProcessoInvalido),
    TabelaDetalhadaAusente,
    TabelasDetalhadasRepetidas,
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
        .map_err(ErroDetalhada::AtributoDaDetalhadaIlegivel)?;
    for no in nos {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            return Ok(true);
        };
        let valor =
            valor_do_atributo(tag, "value").map_err(ErroDetalhada::AtributoDaDetalhadaIlegivel)?;
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
    let Legenda { total, faixa } = ler_legenda(legenda)?;
    let Some(Faixa { inicio, fim }) = faixa else {
        if paginacao_visivel {
            return Err(ErroDetalhada::FaixaSemIntervaloComPaginacao);
        }
        return Ok((total, 1, linhas));
    };
    if inicio == 0 || inicio > fim || fim > total || fim - inicio + 1 != linhas {
        return Err(ErroDetalhada::FaixaDetalhadaInvalida);
    }
    Ok((total, inicio, fim))
}

/// Seleção tolerante do link do processo: menciona a ação de trabalhar ou de
/// visualizar. Quem escolhe confere o candidato com [`conferir_link_do_processo`].
fn menciona_processo(href: &str) -> bool {
    [
        AcaoDoSei::ProcedimentoTrabalhar,
        AcaoDoSei::ProcedimentoVisualizar,
    ]
    .into_iter()
    .any(|acao| menciona_acao(href, acao))
}

/// Confere de forma estrita o link escolhido: `controlador.php`, a ação de
/// trabalhar ou visualizar única e o `id_procedimento` único da própria linha.
fn conferir_link_do_processo(href: &str, id: &str) -> Result<(), ErroDetalhada> {
    let consulta = href
        .strip_prefix("controlador.php?")
        .ok_or(ErroDetalhada::LinkDoProcessoDetalhadoForaDaForma)?;
    let consulta = ConsultaDoControlador::ler(consulta)
        .map_err(ErroDetalhada::ConsultaDoLinkDoProcessoIlegivel)?;
    let acao = consulta
        .acao()
        .map_err(ErroDetalhada::ChaveDoLinkDoProcessoDetalhado)?;
    if !matches!(
        acao,
        AcaoDoSei::ProcedimentoTrabalhar | AcaoDoSei::ProcedimentoVisualizar
    ) {
        return Err(ErroDetalhada::LinkDoProcessoDetalhadoForaDaForma);
    }
    let alvo = consulta
        .unico("id_procedimento")
        .map_err(ErroDetalhada::ChaveDoLinkDoProcessoDetalhado)?;
    if alvo != id {
        return Err(ErroDetalhada::IdDoLinkDoProcessoDiferente);
    }
    Ok(())
}

fn ler_linha(
    dom: &tl::VDom<'_>,
    tag: &tl::HTMLTag<'_>,
    papeis: &[colunas::PapelDaColuna],
) -> Result<Option<(ProcessoDetalhado, String)>, ErroDetalhada> {
    let parser = dom.parser();
    let Some(id) = valor_do_atributo(tag, "id")? else {
        return Ok(None);
    };
    let Some(id) = id.strip_prefix('P') else {
        return Ok(None);
    };
    let id = IdProcedimento::novo(id).map_err(ErroDetalhada::IdentificadorDetalhadoInvalido)?;
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
    let celula = celula.ok_or(ErroDetalhada::CabecalhoProcessoDetalhadoAusente)?;
    let links = celula
        .query_selector(parser, "a")
        .ok_or(ErroDetalhada::LinkDoProcessoDetalhadoAusente)?;
    let mut escolhido = None;
    for link in links {
        let Some(link) = link.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(bruto) = valor_do_atributo(link, "href")? else {
            continue;
        };
        let href = decodificar_atributo(&bruto);
        if !menciona_processo(&href) {
            continue;
        }
        if escolhido.is_some() {
            return Err(ErroDetalhada::LinksDoProcessoDetalhadoRepetidos);
        }
        conferir_link_do_processo(&href, id.as_str())?;
        let classe = valor_do_atributo(link, "class")?.unwrap_or_default();
        let estado = crate::classe_link_processo::estado_do_link(&classe)
            .map_err(ErroDetalhada::ClassesDeEstadoDiferentesNoLink)?
            .ok_or(ErroDetalhada::EstadoDoLinkDoProcessoAusente)?;
        let numero = NumeroProcesso::novo(&link.inner_text(parser))
            .map_err(ErroDetalhada::NumeroDoProcessoDetalhadoAusente)?;
        escolhido = Some((
            ProcessoDetalhado {
                id: id.clone(),
                numero: numero.as_str().to_owned(),
                visualizado: estado.visualizado,
                sigiloso: estado.sigiloso,
                categoria: None,
                colunas: ColunasDaLinha(configuradas.clone()),
            },
            bruto,
        ));
    }
    escolhido
        .map(Some)
        .ok_or(ErroDetalhada::LinkDoProcessoDetalhadoAusente)
}

fn ler_pagina(html: &crate::tela_html::HtmlServido) -> Result<Pagina, ErroDetalhada> {
    let contexto =
        contexto_da_pagina(html).map_err(ErroDetalhada::ControleDaPaginaNaoInterpretado)?;
    if !contexto.eh_detalhada() {
        return Err(ErroDetalhada::VisualizacaoIncorreta);
    }
    if contexto.unidade.is_none() {
        return Err(ErroDetalhada::UnidadeNaoIdentificada);
    }
    let dom = crate::tela_html::interpretar(html);
    let parser = dom.parser();
    let tabelas = nos_com_id(&dom, "tblProcessosDetalhado")
        .map_err(ErroDetalhada::AtributoDaDetalhadaIlegivel)?;
    let tabela = match tabelas.as_slice() {
        [] => return Err(ErroDetalhada::TabelaDetalhadaAusente),
        [tabela] => *tabela,
        _ => return Err(ErroDetalhada::TabelasDetalhadasRepetidas),
    };
    let tabela = tabela
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroDetalhada::TabelaDetalhadaAusente)?;
    if tabela.name() != "table"
        || tabela
            .query_selector(parser, "table")
            .is_some_and(|mut itens| itens.next().is_some())
    {
        return Err(ErroDetalhada::TabelaDetalhadaAusente);
    }
    let mut legendas = tabela
        .query_selector(parser, "caption")
        .ok_or(ErroDetalhada::LegendaDetalhadaAusente)?;
    let legenda = legendas
        .next()
        .ok_or(ErroDetalhada::LegendaDetalhadaAusente)?;
    if legendas.next().is_some() {
        return Err(ErroDetalhada::LegendasDetalhadasRepetidas);
    }
    let legenda = legenda
        .get(parser)
        .ok_or(ErroDetalhada::LegendaDetalhadaAusente)?;
    let papeis = colunas::papeis_do_cabecalho(&dom, tabela)?;
    let configuradas: Vec<ColunaDoDetalhe> = papeis
        .iter()
        .filter_map(|papel| match papel {
            colunas::PapelDaColuna::Configurada(coluna) => Some(*coluna),
            colunas::PapelDaColuna::Processo | colunas::PapelDaColuna::SemRotulo => None,
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
        || crate::paginacao_controle::paginador_com_prefixo(html, "Detalhado");
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
    html: &crate::tela_html::HtmlServido,
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
    paginas: &[crate::tela_html::HtmlServido],
    unidade_esperada: Option<&str>,
) -> Result<LeituraDetalhada, ErroDetalhada> {
    if paginas.is_empty() {
        return Err(ErroDetalhada::TabelaDetalhadaAusente);
    }
    if paginas.iter().map(|pagina| pagina.len()).sum::<usize>() > 48_000_000 {
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
                return Err(ErroDetalhada::IdentificadorDetalhadoRepetido);
            }
            if !numeros.insert(processo.numero.clone()) {
                return Err(ErroDetalhada::NumeroDetalhadoRepetido);
            }
            processos.push(processo);
        }
    }
    let total = total.ok_or(ErroDetalhada::TotalNaoDeclarado)?;
    // Sem página com linhas, a primeira página não fixou início: zero registros.
    let faixa_observada = match faixa_inicio.filter(|_| !processos.is_empty()) {
        Some(inicio) => crate::paginas::FaixaObservada::Lidas {
            inicio,
            fim: proximo - 1,
        },
        None => crate::paginas::FaixaObservada::SemLinhas,
    };
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
    let servido = crate::tela_html::HtmlServido::de_teste(html);
    let dom = crate::tela_html::interpretar(&servido);
    // Tag ilegível conta como possível tabela: a leitura falhará tipada.
    !nos_com_id(&dom, "tblProcessosDetalhado").is_ok_and(|tabelas| tabelas.is_empty())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_colunas;

#[cfg(test)]
pub(crate) mod tests_marcadores;
