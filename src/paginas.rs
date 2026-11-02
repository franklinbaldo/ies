//! Junção de páginas por intervalos de posição, sem emitir requisições.
//! A prova de cobertura vive somente aqui, não em Tabela nem em `CaixaDeEntrada`.
use crate::controle_contexto::contexto_da_pagina;
use crate::dominio::{CaixaDeEntrada, Categoria, Processo, Tabela};
use crate::erro_web::ErroCaixa;
use crate::paginacao_controle::controles_de_paginacao;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Faixa {
    total: usize,
    inicio: usize,
    fim: usize,
}

fn faixa_da_legenda(legenda: &str, linhas: usize) -> Result<Option<Faixa>, ()> {
    let normalizada = legenda
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace('\u{a0}', " ");
    let Some((antes, depois)) = normalizada.trim().split_once(" registros") else {
        return Ok(None);
    };
    let total = antes
        .split_whitespace()
        .last()
        .and_then(|x| x.trim_start_matches('(').parse::<usize>().ok())
        .ok_or(())?;
    if let Some(resto) = depois.strip_prefix(" - ") {
        let (inicio, fim) = resto.split_once(" a ").ok_or(())?;
        let inicio = inicio.trim().parse::<usize>().map_err(|_| ())?;
        let fim = fim
            .trim()
            .trim_end_matches(|c: char| !c.is_ascii_digit())
            .parse::<usize>()
            .map_err(|_| ())?;
        if inicio == 0 || inicio > fim || fim > total || fim - inicio + 1 != linhas {
            return Err(());
        }
        Ok(Some(Faixa { total, inicio, fim }))
    } else if total == linhas {
        // Sentinela de faixa vazia: a tabela informou zero registros.
        Ok(Some(Faixa {
            total,
            inicio: 1,
            fim: total,
        }))
    } else {
        Ok(None)
    }
}

fn faixa_da_tabela(
    html: &str,
    id: &str,
    categoria: Categoria,
    linhas: usize,
) -> Result<Option<Faixa>, ErroCaixa> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroCaixa::HtmlInvalido)?;
    let parser = dom.parser();
    let tabela = dom
        .get_element_by_id(id)
        .and_then(|no| no.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or(ErroCaixa::TabelaAusente { categoria })?;
    let mut legendas = tabela
        .query_selector(parser, "caption")
        .ok_or(ErroCaixa::LegendaAusente { categoria })?;
    let primeira = legendas
        .next()
        .ok_or(ErroCaixa::LegendaAusente { categoria })?;
    if legendas.next().is_some() {
        return Err(ErroCaixa::LegendasDaTabelaRepetidas { categoria });
    }
    let legenda = primeira
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroCaixa::LegendaAusente { categoria })?
        .inner_text(parser);
    faixa_da_legenda(&legenda, linhas).map_err(|()| ErroCaixa::FaixaInvalida { categoria })
}

#[derive(Debug)]
struct Grupo {
    total: Option<usize>,
    faixas: BTreeMap<usize, (usize, Vec<Processo>)>,
    desconhecida: bool,
}

impl Grupo {
    fn novo() -> Self {
        Self {
            total: None,
            faixas: BTreeMap::new(),
            desconhecida: false,
        }
    }

    fn incluir(
        &mut self,
        categoria: Categoria,
        tabela: &Tabela<Processo>,
        faixa: Option<Faixa>,
    ) -> Result<(), ErroCaixa> {
        if let (Some(anterior), Some(atual)) = (self.total, tabela.total_declarado)
            && anterior != atual
        {
            return Err(ErroCaixa::TotaisDeclaradosDiferentesEntrePaginas { categoria });
        }
        if self.total.is_none() {
            self.total = tabela.total_declarado;
        }
        let Some(faixa) = faixa else {
            if !self.faixas.is_empty() {
                return Err(ErroCaixa::FaixaDesconhecida { categoria });
            }
            self.desconhecida = true;
            self.faixas
                .insert(1, (tabela.processos.len(), tabela.processos.clone()));
            return Ok(());
        };
        if self.desconhecida {
            return Err(ErroCaixa::FaixaDesconhecida { categoria });
        }
        if tabela.total_declarado != Some(faixa.total) {
            return Err(ErroCaixa::TotaisDeclaradosDiferentesEntrePaginas { categoria });
        }
        match self.faixas.get(&faixa.inicio) {
            Some((fim, anterior)) if *fim == faixa.fim && anterior == &tabela.processos => Ok(()),
            Some(_) => Err(ErroCaixa::FaixaAlterada { categoria }),
            None => {
                self.faixas
                    .insert(faixa.inicio, (faixa.fim, tabela.processos.clone()));
                Ok(())
            }
        }
    }

    fn finalizar(self, categoria: Categoria) -> Result<(Tabela<Processo>, Cobertura), ErroCaixa> {
        let mut proximo = 1usize;
        let mut intervalos = Vec::new();
        let mut processos = Vec::new();
        for (inicio, (fim, itens)) in self.faixas {
            if inicio != proximo {
                return Err(ErroCaixa::FaixaDescontinua { categoria });
            }
            processos.extend(itens);
            intervalos.push((inicio, fim));
            proximo = fim.saturating_add(1);
        }
        let tabela = Tabela::nova(categoria, self.total, processos)?;
        Ok((
            tabela,
            Cobertura {
                intervalos,
                confirmada: !self.desconhecida,
            },
        ))
    }
}

/// Evidência de paginação transitória: nunca é guardada no domínio.
#[derive(Debug)]
pub struct Cobertura {
    intervalos: Vec<(usize, usize)>,
    confirmada: bool,
}

impl Cobertura {
    // Comprova somente a cobertura das posições declaradas, sem snapshot
    // transacional; a caixa pode mudar entre requisições.
    pub fn completa(&self, tabela: &Tabela<Processo>) -> bool {
        if !self.confirmada {
            return false;
        }
        let Some(total) = tabela.total_declarado else {
            return false;
        };
        if tabela.processos.len() != total {
            return false;
        }
        let mut proximo = 1usize;
        for &(inicio, fim) in &self.intervalos {
            if inicio != proximo {
                return false;
            }
            if total == 0 && inicio == 1 && fim == 0 && self.intervalos.len() == 1 {
                return true;
            }
            if fim < inicio {
                return false;
            }
            proximo = fim.saturating_add(1);
        }
        proximo == total.saturating_add(1)
    }
}

pub struct Leitura {
    pub caixa: CaixaDeEntrada,
    pub recebidos: Cobertura,
    pub gerados: Cobertura,
    pub unidade: Option<String>,
}

pub fn juntar(paginas: &[String], unidade_esperada: Option<&str>) -> Result<Leitura, ErroCaixa> {
    if paginas.is_empty() {
        return Err(ErroCaixa::SemPaginas);
    }
    if paginas.len() > 200 || paginas.iter().map(String::len).sum::<usize>() > 48_000_000 {
        return Err(ErroCaixa::LimitePaginas);
    }
    let contexto_inicial = contexto_da_pagina(&paginas[0])?;
    if contexto_inicial.eh_resumida() && controles_de_paginacao(&paginas[0])? {
        return Err(ErroCaixa::VisaoResumidaComPaginacao);
    }
    if let Some(esperada) = unidade_esperada
        && contexto_inicial
            .unidade
            .as_deref()
            .is_none_or(|atual| !atual.eq_ignore_ascii_case(esperada.trim()))
    {
        return Err(ErroCaixa::UnidadeInesperada);
    }
    if paginas.len() > 1 && !contexto_inicial.verificavel() {
        return Err(ErroCaixa::ContextoNaoVerificavel);
    }
    let mut recebidos = Grupo::novo();
    let mut gerados = Grupo::novo();
    for html in paginas {
        if paginas.len() > 1 {
            let contexto = contexto_da_pagina(html)?;
            if !contexto.verificavel() {
                return Err(ErroCaixa::ContextoNaoVerificavel);
            }
            if contexto.eh_resumida() && controles_de_paginacao(html)? {
                return Err(ErroCaixa::VisaoResumidaComPaginacao);
            }
            if contexto != contexto_inicial {
                return Err(ErroCaixa::ContextoDaPaginaDiferenteDoAnterior);
            }
        }
        let caixa = crate::caixa::parse_html(html)?;
        for (categoria, nome, tabela, grupo) in [
            (
                Categoria::Recebidos,
                "tblProcessosRecebidos",
                &caixa.recebidos,
                &mut recebidos,
            ),
            (
                Categoria::Gerados,
                "tblProcessosGerados",
                &caixa.gerados,
                &mut gerados,
            ),
        ] {
            let faixa = faixa_da_tabela(html, nome, categoria, tabela.processos.len())?;
            grupo.incluir(categoria, tabela, faixa)?;
        }
    }
    let (rec, cobertura_rec) = recebidos.finalizar(Categoria::Recebidos)?;
    let (ger, cobertura_ger) = gerados.finalizar(Categoria::Gerados)?;
    Ok(Leitura {
        caixa: CaixaDeEntrada {
            recebidos: rec,
            gerados: ger,
        },
        recebidos: cobertura_rec,
        gerados: cobertura_ger,
        unidade: contexto_inicial.unidade,
    })
}

#[cfg(test)]
mod tests;
