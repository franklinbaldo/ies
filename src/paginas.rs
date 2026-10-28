//! Junção de páginas por intervalos de posição, sem emitir requisições.
//! A prova de cobertura vive somente aqui, não em Tabela nem em Caixa.
use crate::dominio::{Caixa, Categoria, ErroCaixa, Processo, Tabela};
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
    let legenda = tabela
        .query_selector(parser, "caption")
        .and_then(|mut itens| itens.next())
        .and_then(|no| no.get(parser))
        .and_then(tl::Node::as_tag)
        .ok_or(ErroCaixa::LegendaAusente { categoria })?
        .inner_text(parser);
    faixa_da_legenda(&legenda, linhas).map_err(|_| ErroCaixa::FaixaInvalida { categoria })
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
            return Err(ErroCaixa::TotalDivergente { categoria });
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
            return Err(ErroCaixa::TotalDivergente { categoria });
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
    pub caixa: Caixa,
    pub recebidos: Cobertura,
    pub gerados: Cobertura,
}

pub fn juntar(paginas: &[String]) -> Result<Leitura, ErroCaixa> {
    if paginas.is_empty() {
        return Err(ErroCaixa::SemPaginas);
    }
    if paginas.len() > 200 || paginas.iter().map(String::len).sum::<usize>() > 48_000_000 {
        return Err(ErroCaixa::LimitePaginas);
    }
    let mut recebidos = Grupo::novo();
    let mut gerados = Grupo::novo();
    for html in paginas {
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
        caixa: Caixa {
            recebidos: rec,
            gerados: ger,
        },
        recebidos: cobertura_rec,
        gerados: cobertura_ger,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linha(id: u32) -> String {
        format!("<tr id='P{id}'><td><a class='processoVisualizado'>NUP-{id}</a></td></tr>")
    }
    fn html(rec: (usize, usize, usize, &[u32]), ger: (usize, usize, usize, &[u32])) -> String {
        let montar = |nome: &str, (total, ini, fim, ids): (usize, usize, usize, &[u32])| {
            format!(
                "<table id='tblProcessos{nome}'><caption>{total} registros - {ini} a {fim}):</caption>{}</table>",
                ids.iter().map(|id| linha(*id)).collect::<String>()
            )
        };
        format!("{}{}", montar("Recebidos", rec), montar("Gerados", ger))
    }
    #[test]
    fn duas_tabelas_paginam_independentemente() {
        let primeira = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let segunda = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        let leitura = juntar(&[primeira, segunda]).unwrap();
        assert!(leitura.recebidos.completa(&leitura.caixa.recebidos));
        assert!(leitura.gerados.completa(&leitura.caixa.gerados));
        assert_eq!(leitura.caixa.recebidos.processos.len(), 4);
    }
    #[test]
    fn recusa_lacuna_e_sobreposicao() {
        let a = html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((5, 4, 5, &[4, 5]), (1, 1, 1, &[11]));
        assert_eq!(
            juntar(&[a, b]).err().unwrap().subcodigo(),
            "faixa_descontinua"
        );
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 2, 3, &[2, 3]), (1, 1, 1, &[11]));
        assert_eq!(
            juntar(&[a, b]).err().unwrap().subcodigo(),
            "faixa_descontinua"
        );
    }
    #[test]
    fn recusa_troca_conteudo_total_e_identidade_repetida() {
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 1, 2, &[1, 7]), (1, 1, 1, &[11]));
        assert_eq!(juntar(&[a, b]).err().unwrap().subcodigo(), "faixa_alterada");
        let a = html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        assert_eq!(
            juntar(&[a, b]).err().unwrap().subcodigo(),
            "total_divergente"
        );
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 3, 4, &[2, 4]), (1, 1, 1, &[11]));
        assert_eq!(juntar(&[a, b]).err().unwrap().subcodigo(), "id_repetido");
    }
    #[test]
    fn primeira_pagina_incompleta_nao_inventa_completude() {
        let a = html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let l = juntar(&[a]).unwrap();
        assert!(!l.recebidos.completa(&l.caixa.recebidos));
        assert!(l.gerados.completa(&l.caixa.gerados));
    }
    #[test]
    fn legenda_sem_faixa_nao_gera_prova_falsa() {
        let a = "<table id='tblProcessosRecebidos'><caption>5 registros:</caption>\
            <tr id='P1'><td><a class='processoVisualizado'>NUP-1</a></td></tr></table>\
            <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let l = juntar(&[a.to_owned()]).unwrap();
        assert!(!l.recebidos.completa(&l.caixa.recebidos));
        assert!(l.gerados.completa(&l.caixa.gerados));
    }
    #[test]
    fn zero_itens_verificados_e_uma_colecao_completa() {
        let a = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
            <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let l = juntar(&[a.to_owned()]).unwrap();
        assert!(l.recebidos.completa(&l.caixa.recebidos));
        assert!(l.gerados.completa(&l.caixa.gerados));
    }
}
