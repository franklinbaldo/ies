//! Junção de páginas por intervalos de posição, sem emitir requisições.
//! A prova de cobertura vive somente aqui, não em Tabela nem em `CaixaDeEntrada`.
use crate::dominio::{CaixaDeEntrada, Categoria, Processo, Tabela};
use crate::erro_web::ErroCaixa;
use std::collections::BTreeMap;

/// Contexto da página HTML: só é usado para conferir a junção.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextoPagina {
    unidade: Option<String>,
    modo: Option<String>,
    criterios_recebidos: Option<String>,
    criterios_gerados: Option<String>,
}

impl ContextoPagina {
    fn verificavel(&self) -> bool {
        self.unidade.is_some()
            && self.modo.is_some()
            && self.criterios_recebidos.is_some()
            && self.criterios_gerados.is_some()
    }
}

fn unidade_html(dom: &tl::VDom<'_>) -> Option<String> {
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
    unidade_html(&dom).ok_or(ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle)
}

fn valor_oculto(dom: &tl::VDom<'_>, id: &str) -> Option<String> {
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

fn contexto_da_pagina(html: &str) -> Result<ContextoPagina, ErroCaixa> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroCaixa::HtmlInvalido)?;
    Ok(ContextoPagina {
        unidade: unidade_html(&dom),
        modo: valor_oculto(&dom, "hdnTipoVisualizacao"),
        criterios_recebidos: valor_oculto(&dom, "hdnRecebidosHashCriterios"),
        criterios_gerados: valor_oculto(&dom, "hdnGeradosHashCriterios"),
    })
}

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
mod tests {
    use super::*;

    fn linha(id: u32) -> String {
        format!("<tr id='P{id}'><td><a class='processoVisualizado'>NUP-{id}</a></td></tr>")
    }
    fn html(rec: (usize, usize, usize, &[u32]), ger: (usize, usize, usize, &[u32])) -> String {
        let montar = |nome: &str, (total, ini, fim, ids): (usize, usize, usize, &[u32])| {
            let legenda = if total == 0 {
                "0 registros:".to_owned()
            } else {
                format!("{total} registros - {ini} a {fim}):")
            };
            format!(
                "<table id='tblProcessos{nome}'><caption>{legenda}</caption>{}</table>",
                ids.iter().map(|id| linha(*id)).collect::<String>()
            )
        };
        format!(
            "<a id='lnkInfraUnidade'>TESTE</a>             <input id='hdnTipoVisualizacao' value='R'>             <input id='hdnRecebidosHashCriterios' value='recebidos'>             <input id='hdnGeradosHashCriterios' value='gerados'>             {}{}",
            montar("Recebidos", rec),
            montar("Gerados", ger)
        )
    }
    #[test]
    fn ablacao_da_segunda_pagina_impede_afirmar_cobertura() {
        let primeira = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let segunda = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        let referencia = juntar(&[primeira.clone(), segunda], Some("TESTE")).unwrap();
        let sem_segunda = juntar(&[primeira], Some("TESTE")).unwrap();
        assert_eq!(referencia.caixa.recebidos.processos.len(), 4);
        assert!(referencia.recebidos.completa(&referencia.caixa.recebidos));
        assert_eq!(sem_segunda.caixa.recebidos.processos.len(), 2);
        assert!(!sem_segunda.recebidos.completa(&sem_segunda.caixa.recebidos));
        assert_eq!(
            referencia.caixa.gerados.processos,
            sem_segunda.caixa.gerados.processos
        );
    }

    #[test]
    fn ablacao_de_verificacao_de_contexto_nao_e_segura() {
        let primeira = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let segunda = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        let outra_unidade = segunda.replace(">TESTE</a>", ">OUTRA</a>");
        let erro = juntar(&[primeira, outra_unidade], Some("TESTE"))
            .err()
            .unwrap();
        assert_eq!(erro.subcodigo(), "contexto_divergente");
    }

    #[test]
    fn duas_tabelas_paginam_independentemente() {
        let primeira = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let segunda = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        let leitura = juntar(&[primeira, segunda], None).unwrap();
        assert!(leitura.recebidos.completa(&leitura.caixa.recebidos));
        assert!(leitura.gerados.completa(&leitura.caixa.gerados));
        assert_eq!(leitura.caixa.recebidos.processos.len(), 4);
    }
    #[test]
    fn recusa_lacuna_e_sobreposicao() {
        let a = html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((5, 4, 5, &[4, 5]), (1, 1, 1, &[11]));
        assert_eq!(
            juntar(&[a, b], None).err().unwrap().subcodigo(),
            "faixa_descontinua"
        );
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 2, 3, &[2, 3]), (1, 1, 1, &[11]));
        assert_eq!(
            juntar(&[a, b], None).err().unwrap().subcodigo(),
            "faixa_descontinua"
        );
    }
    #[test]
    fn recusa_troca_conteudo_total_e_identidade_repetida() {
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 1, 2, &[1, 7]), (1, 1, 1, &[11]));
        assert_eq!(
            juntar(&[a, b], None).err().unwrap().subcodigo(),
            "faixa_alterada"
        );
        let a = html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        assert_eq!(
            juntar(&[a, b], None).err().unwrap().subcodigo(),
            "total_divergente"
        );
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 3, 4, &[2, 4]), (1, 1, 1, &[11]));
        assert_eq!(
            juntar(&[a, b], None).err().unwrap().subcodigo(),
            "id_repetido"
        );
    }
    #[test]
    fn primeira_pagina_incompleta_nao_inventa_completude() {
        let a = html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let l = juntar(&[a], None).unwrap();
        assert!(!l.recebidos.completa(&l.caixa.recebidos));
        assert!(l.gerados.completa(&l.caixa.gerados));
    }
    #[test]
    fn legenda_sem_faixa_nao_gera_prova_falsa() {
        let a = "<table id='tblProcessosRecebidos'><caption>5 registros:</caption>\
            <tr id='P1'><td><a class='processoVisualizado'>NUP-1</a></td></tr></table>\
            <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let l = juntar(&[a.to_owned()], None).unwrap();
        assert!(!l.recebidos.completa(&l.caixa.recebidos));
        assert!(l.gerados.completa(&l.caixa.gerados));
    }
    #[test]
    fn zero_itens_verificados_e_uma_colecao_completa() {
        let a = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
            <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let l = juntar(&[a.to_owned()], None).unwrap();
        assert!(l.recebidos.completa(&l.caixa.recebidos));
        assert!(l.gerados.completa(&l.caixa.gerados));
    }

    #[test]
    fn unidades_distintas_nao_podem_ser_mescladas() {
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        let b = b.replace(">TESTE</a>", ">OUTRA</a>");
        assert_eq!(
            juntar(&[a, b], None).err().unwrap().subcodigo(),
            "contexto_divergente"
        );
    }

    #[test]
    fn filtros_e_modo_divergentes_bloqueiam_juncao() {
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        assert_eq!(
            juntar(
                &[a.clone(), b.replace("value='recebidos'", "value='outro'")],
                None
            )
            .err()
            .unwrap()
            .subcodigo(),
            "contexto_divergente"
        );
        let b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        assert_eq!(
            juntar(&[a, b.replace("value='R'", "value='D'")], None)
                .err()
                .unwrap()
                .subcodigo(),
            "contexto_divergente"
        );
    }

    #[test]
    fn unidade_solicitada_precisa_estar_na_pagina() {
        let a = html((1, 1, 1, &[42]), (0, 1, 0, &[]));
        assert_eq!(
            juntar(std::slice::from_ref(&a), Some("TESTE"))
                .unwrap()
                .unidade
                .as_deref(),
            Some("TESTE")
        );
        assert_eq!(
            juntar(&[a,], Some("OUTRA")).err().unwrap().subcodigo(),
            "unidade_inesperada"
        );
        let b = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table><table id='tblProcessosGerados'><caption>0 registros:</caption></table>".to_owned();
        assert_eq!(
            juntar(&[b], Some("TESTE")).err().unwrap().subcodigo(),
            "unidade_inesperada"
        );
    }

    #[test]
    fn identifica_unidade_publicada_sem_consultar_paginacao() {
        let html = "<a id='lnkInfraUnidade'>PGE-PPI</a>";
        assert_eq!(unidade_exibida_no_controle(html).unwrap(), "PGE-PPI");
        assert_eq!(
            unidade_exibida_no_controle("<body>sem unidade</body>").unwrap_err(),
            ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle
        );
    }

    #[test]
    fn contexto_ausente_nao_certifica_mescla() {
        let a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
        let b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
        let b = b.replace("id='lnkInfraUnidade'", "id='outro'");
        assert_eq!(
            juntar(&[a, b], None).err().unwrap().subcodigo(),
            "contexto_nao_verificavel"
        );
    }
}
