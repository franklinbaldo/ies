//! Junção de páginas por intervalos de posição, sem emitir requisições.
//! A prova de cobertura vive somente aqui, não em Tabela nem em `CaixaDeEntrada`.
use crate::controle_contexto::contexto_da_pagina;
use crate::dominio::{CaixaDeEntrada, Categoria, Processo, Tabela};
use crate::erro_web::ErroCaixa;
use crate::html_atributos::nos_com_id;
use crate::paginacao_controle::{controles_de_paginacao, paginador_da_tabela};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Faixa {
    total: usize,
    inicio: usize,
    fim: usize,
    /// A legenda publicou `- inicio a fim`; sem isso a faixa é deduzida.
    explicita: bool,
}

/// A legenda declara total ou faixa fora da forma observada.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroFaixaDaLegenda {
    TotalIlegivel,
    FaixaSemSeparador,
    InicioIlegivel(#[source] std::num::ParseIntError),
    FimIlegivel(#[source] std::num::ParseIntError),
    /// Início zero, início depois do fim, fim acima do total ou tamanho
    /// diferente das linhas servidas.
    FaixaIncoerenteComAsLinhas,
}

fn faixa_da_legenda(legenda: &str, linhas: usize) -> Result<Option<Faixa>, ErroFaixaDaLegenda> {
    let normalizada = legenda
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace('\u{a0}', " ");
    // Singular (`1 registro:`, UFRJ `(1 registro):`), como a visão D já lê.
    let Some((antes, depois)) = normalizada.trim().split_once(" registro") else {
        return Ok(None);
    };
    let depois = depois.strip_prefix('s').unwrap_or(depois);
    let total = antes
        .split_whitespace()
        .last()
        .and_then(|x| x.trim_start_matches('(').parse::<usize>().ok())
        .ok_or(ErroFaixaDaLegenda::TotalIlegivel)?;
    if let Some(resto) = depois.strip_prefix(" - ") {
        let (inicio, fim) = resto
            .split_once(" a ")
            .ok_or(ErroFaixaDaLegenda::FaixaSemSeparador)?;
        let inicio = inicio
            .trim()
            .parse::<usize>()
            .map_err(ErroFaixaDaLegenda::InicioIlegivel)?;
        let fim = fim
            .trim()
            .trim_end_matches(|c: char| !c.is_ascii_digit())
            .parse::<usize>()
            .map_err(ErroFaixaDaLegenda::FimIlegivel)?;
        if inicio == 0 || inicio > fim || fim > total || fim - inicio + 1 != linhas {
            return Err(ErroFaixaDaLegenda::FaixaIncoerenteComAsLinhas);
        }
        Ok(Some(Faixa {
            total,
            inicio,
            fim,
            explicita: true,
        }))
    } else if total == linhas {
        // Sentinela de faixa vazia: a tabela informou zero registros.
        Ok(Some(Faixa {
            total,
            inicio: 1,
            fim: total,
            explicita: false,
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
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroCaixa::HtmlDaCaixaRecusadoPeloParser)?;
    let parser = dom.parser();
    let tabela = match nos_com_id(&dom, id)
        .map_err(ErroCaixa::AtributoDaCaixaIlegivel)?
        .as_slice()
    {
        // Lista vazia comprovada pela estrutura: a mesma faixa de `0 registros:`.
        [] if crate::caixa::lista_vazia_comprovada(&dom, categoria)? => {
            return Ok(Some(Faixa {
                total: 0,
                inicio: 1,
                fim: 0,
                explicita: false,
            }));
        }
        [] => return Err(ErroCaixa::TabelaAusente { categoria }),
        [tabela] => *tabela,
        _ => return Err(ErroCaixa::TabelasDaCategoriaRepetidas { categoria }),
    };
    let tabela = tabela
        .get(parser)
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
    faixa_da_legenda(&legenda, linhas)
        .map_err(|motivo| ErroCaixa::FaixaInvalida { categoria, motivo })
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
        // Páginas contíguas entre si; a primeira pode não ser a 1, porque o
        // servidor lembra a página pedida na sessão.
        let mut proximo = self.faixas.keys().next().copied().unwrap_or(1);
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
    #[cfg(test)]
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

    /// Posições lidas (`inicio`, `fim`), contíguas; `None` sem linhas ou
    /// sem faixa comprovada pela legenda.
    pub fn faixa_observada(&self) -> Option<(usize, usize)> {
        if !self.confirmada {
            return None;
        }
        let inicio = self.intervalos.first()?.0;
        let fim = self.intervalos.last()?.1;
        (fim >= inicio).then_some((inicio, fim))
    }
}

/// Total e posições de uma tabela numa única página, lidos da legenda.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FaixaNaPagina {
    pub(crate) total: usize,
    pub(crate) inicio: usize,
    pub(crate) fim: usize,
}

fn id_da_tabela(categoria: Categoria) -> &'static str {
    match categoria {
        Categoria::Recebidos => "tblProcessosRecebidos",
        Categoria::Gerados => "tblProcessosGerados",
    }
}

/// Faixa de uma tabela numa página isolada; `None` se a tabela não tem
/// linhas ou a legenda não a comprova.
pub(crate) fn faixa_na_pagina(
    html: &str,
    categoria: Categoria,
) -> Result<Option<FaixaNaPagina>, ErroCaixa> {
    let caixa = crate::caixa::parse_html(html)?;
    let tabela = match categoria {
        Categoria::Recebidos => &caixa.recebidos,
        Categoria::Gerados => &caixa.gerados,
    };
    let faixa = faixa_da_tabela(
        html,
        id_da_tabela(categoria),
        categoria,
        tabela.processos.len(),
    )?;
    Ok(faixa.filter(|f| f.fim >= f.inicio).map(|f| FaixaNaPagina {
        total: f.total,
        inicio: f.inicio,
        fim: f.fim,
    }))
}

/// Recusa tabela cuja faixa observada não pode ser publicada com o total:
/// legenda sem `registros`, ou sem intervalo e com total diferente das linhas.
pub(crate) fn conferir_faixa_publicavel(
    categoria: Categoria,
    tabela: &Tabela<Processo>,
    cobertura: &Cobertura,
) -> Result<(), ErroCaixa> {
    if tabela.total_declarado.is_none() {
        return Err(ErroCaixa::TotalNaoDeclaradoNaLegenda { categoria });
    }
    if !cobertura.confirmada {
        return Err(ErroCaixa::LegendaSemFaixaComTotalDiferenteDasLinhas { categoria });
    }
    Ok(())
}

pub struct Leitura {
    pub caixa: CaixaDeEntrada,
    pub recebidos: Cobertura,
    pub gerados: Cobertura,
    pub unidade: Option<String>,
}

pub fn juntar(paginas: &[String], unidade_esperada: Option<&str>) -> Result<Leitura, ErroCaixa> {
    let Some(primeira) = paginas.first() else {
        return Err(ErroCaixa::SemPaginas);
    };
    if paginas.len() > 200 || paginas.iter().map(String::len).sum::<usize>() > 48_000_000 {
        return Err(ErroCaixa::LimitePaginas);
    }
    let contexto_inicial = contexto_da_pagina(primeira)?;
    if contexto_inicial.eh_resumida() && controles_de_paginacao(primeira)? {
        return Err(ErroCaixa::VisaoResumidaComPaginacao);
    }
    for html in paginas {
        if crate::controle_contexto::filtro_de_marcador_ativo(html)? {
            return Err(ErroCaixa::FiltroDeMarcadorAtivoNoControle);
        }
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
            // Paginador próprio da tabela com legenda sem intervalo: a
            // coincidência entre total e linhas não comprova página única.
            if faixa.is_some_and(|f| !f.explicita && f.total > 0)
                && paginador_da_tabela(html, categoria)?
            {
                return Err(ErroCaixa::PaginadorDaTabelaSemFaixaNaLegenda { categoria });
            }
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

#[cfg(test)]
mod tests_faixa;

#[cfg(test)]
mod tests_filtro;

#[cfg(test)]
mod tests_vazia;
