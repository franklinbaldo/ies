//! Plano de página de cada tabela, pelos mesmos leitores de `--pagina N`:
//! `pagina_do_controle::planejar` na R (Recebidos, Gerados) e
//! `posicao`/`planejar` da D. A página 1 tem de ser a exibida; numa lista
//! com mais de uma página, a página 2 tem de ser um pedido com a faixa
//! seguinte; numa lista de uma página, a página 2 fica fora do intervalo.
use super::super::passos::ler_detalhada;
use super::super::plano::{planejar, posicao, valores_ocultos};
use super::leitores::{Cassette, forma_da_lista};
use super::{Leitor, Lista, Registro, Replay, Resultado};
use crate::caixa_detalhada::ValorDaColuna;
use crate::cassette::acervo::gravacao::Resposta;
use crate::dominio::Categoria;
use crate::html_atributos::valor_do_atributo;
use crate::pagina_do_controle::{ErroPaginaDoControle, PlanoDaPagina};
use crate::paginas::faixa_na_pagina;

fn prefixo(categoria: Categoria) -> &'static str {
    match categoria {
        Categoria::Recebidos => "Recebidos",
        Categoria::Gerados => "Gerados",
    }
}

/// `hdn{T}HashCriterios` servido (só para comparar páginas).
pub(super) fn criterio(html: &str, categoria: Categoria) -> Option<String> {
    let id = format!("hdn{}HashCriterios", prefixo(categoria));
    valores_ocultos(html, &id).ok()?.into_iter().next()
}

/// Controles de paginação publicados com o prefixo da tabela (`lnk{P}…`,
/// `sel{P}Paginacao…`), sem o sufixo `Superior`/`Inferior`.
pub(super) fn controles(html: &str, prefixo: &str) -> Vec<String> {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return Vec::new();
    };
    let parser = dom.parser();
    let mut achados = Vec::new();
    for seletor in ["a", "select"] {
        for no in dom.query_selector(seletor).into_iter().flatten() {
            let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            let Ok(Some(id)) = valor_do_atributo(tag, "id") else {
                continue;
            };
            let tipo = if seletor == "a" { "lnk" } else { "sel" };
            let Some(resto) = id.strip_prefix(tipo).and_then(|r| r.strip_prefix(prefixo)) else {
                continue;
            };
            let resto = resto
                .strip_suffix("Superior")
                .or_else(|| resto.strip_suffix("Inferior"))
                .unwrap_or(resto);
            let marca = format!("{tipo}{resto}");
            if ["Pagina", "Paginacao"].iter().any(|m| resto.contains(m))
                && !achados.contains(&marca)
            {
                achados.push(marca);
            }
        }
    }
    achados.sort();
    achados
}

fn leitor_da_pagina(categoria: Categoria) -> Leitor {
    match categoria {
        Categoria::Recebidos => Leitor::PaginaRecebidos,
        Categoria::Gerados => Leitor::PaginaGerados,
    }
}

/// `pagina_do_controle::planejar` para as páginas 1 e 2 da tabela R.
pub(super) fn da_tabela(
    cas: &Cassette<'_>,
    resp: &Resposta<'_>,
    html: &str,
    categoria: Categoria,
) -> Registro {
    let mut p = cas.registro(leitor_da_pagina(categoria), resp.passo, Resultado::Lido);
    p.formas = controles(html, prefixo(categoria));
    match crate::paginacao_controle::paginador_da_tabela(html, categoria) {
        Ok(true) => p.formas.push("paginador_publicado".into()),
        Ok(false) => p.formas.push("sem_paginador".into()),
        Err(erro) => p.formas.push(format!("paginador_{erro:?}")),
    }
    match planejar_r(html, categoria) {
        Ok(contagens) => p.contagens = contagens,
        Err(sub) => p.resultado = Resultado::Recusado(sub),
    }
    p
}

fn planejar_r(html: &str, categoria: Categoria) -> Result<Vec<(&'static str, u64)>, String> {
    use crate::pagina_do_controle::planejar as plano;
    let faixa = faixa_na_pagina(html, categoria).map_err(|e| format!("{e:?}"))?;
    // A página exibida pela faixa (páginas cheias antes dela).
    let exibida = faixa.map_or(1, |f| (f.inicio - 1) / (f.fim - f.inicio + 1) + 1);
    match plano(html, categoria, exibida).map_err(|e| format!("{e:?}"))? {
        PlanoDaPagina::JaExibida => {}
        PlanoDaPagina::Pedir { .. } => return Err("pagina_exibida_nao_reconhecida".to_owned()),
    }
    let mut contagens = vec![("pagina_exibida", exibida as u64)];
    match plano(html, categoria, exibida + 1) {
        Ok(PlanoDaPagina::Pedir { indice, esperada }) => {
            let f = faixa.ok_or("pedido_sem_faixa")?;
            let tamanho = (f.fim - f.inicio + 1) as u64;
            contagens.extend([
                ("indice_pedido", indice as u64),
                ("esperada_inicio", esperada.inicio as u64),
                ("esperada_fim", esperada.fim as u64),
                ("tamanho", tamanho),
                ("paginas", (f.total as u64).div_ceil(tamanho)),
            ]);
        }
        Err(ErroPaginaDoControle::PaginaForaDoIntervalo { paginas, .. }) => {
            contagens.push(("paginas", paginas as u64));
        }
        Ok(PlanoDaPagina::JaExibida) => return Err("pagina_seguinte_dada_como_exibida".to_owned()),
        Err(erro) => return Err(format!("{erro:?}")),
    }
    Ok(contagens)
}

/// Visão D: a lista com colunas (`ler_detalhada`) e a posição/plano da D.
pub(super) fn detalhada(
    cas: &Cassette<'_>,
    resp: &Resposta<'_>,
    html: &str,
    unidade: &str,
    fora: Option<&'static str>,
    saida: &mut Replay,
) {
    for leitor in [
        Leitor::Resumida,
        Leitor::Recebidos,
        Leitor::Gerados,
        Leitor::PaginaRecebidos,
        Leitor::PaginaGerados,
    ] {
        let motivo = Resultado::NaoAplicavel("tela_na_visao_d_o_comando_troca_para_r".to_owned());
        saida
            .registros
            .push(cas.registro(leitor, resp.passo, motivo));
    }
    let mut lista = cas.registro(Leitor::Detalhada, resp.passo, Resultado::Lido);
    let leitura = match ler_detalhada(html, unidade) {
        Ok(leitura) => leitura,
        Err(erro) => {
            lista.resultado = fora.map_or(Resultado::Recusado(format!("{erro:?}")), |motivo| {
                Resultado::NaoAplicavel(motivo.to_owned())
            });
            lista.formas.push(format!("recusa_{erro:?}"));
            saida.registros.push(lista);
            return;
        }
    };
    let linhas = leitura.processos.len() as u64;
    let total = leitura.total_declarado as u64;
    let marcadores = leitura
        .processos
        .iter()
        .flat_map(|processo| processo.colunas.0.iter())
        .filter(|(_, valor)| matches!(valor, ValorDaColuna::Marcadores(m) if !m.is_empty()))
        .count() as u64;
    lista.contagens = vec![
        ("linhas", linhas),
        ("total_declarado", total),
        ("colunas", leitura.colunas.len() as u64),
        ("colunas_ausentes", leitura.colunas_ausentes().len() as u64),
        ("celulas_com_marcador", marcadores),
    ];
    lista.formas.push(forma_da_lista(linhas, total).into());
    lista.formas.extend(
        leitura
            .colunas
            .iter()
            .map(|col| format!("coluna_{}", col.chave())),
    );
    saida.registros.push(lista);
    let mut pagina = cas.pagina(resp.passo, Lista::Detalhada, unidade);
    pagina.total = leitura.total_declarado;
    // A cassette grava zero linhas como ausência de faixa (NULL).
    pagina.faixa = match leitura.faixa_observada {
        crate::paginas::FaixaObservada::SemLinhas => None,
        crate::paginas::FaixaObservada::Lidas { inicio, fim } => Some((inicio, fim)),
    };
    pagina.ids = leitura
        .processos
        .iter()
        .map(|x| x.id.as_str().to_owned())
        .collect();
    pagina.criterio = valores_ocultos(html, "hdnDetalhadoHashCriterios")
        .ok()
        .and_then(|v| v.into_iter().next());
    saida.paginas.push(pagina);
    saida
        .registros
        .push(posicao_da_d(cas, resp, html, &leitura));
    let mut colheita = cas.registro(Leitor::LinhasComLink, resp.passo, Resultado::Lido);
    match crate::caixa_detalhada::linhas_com_link(html) {
        Ok(linhas) => {
            let com = linhas.iter().filter(|l| l.href.is_some()).count() as u64;
            colheita.contagens = vec![("linhas", linhas.len() as u64), ("com_href", com)];
        }
        Err(erro) => colheita.resultado = Resultado::Recusado(format!("{erro:?}")),
    }
    saida.registros.push(colheita);
}

/// `posicao` da tela D e o plano da página exibida e da seguinte.
fn posicao_da_d(
    cas: &Cassette<'_>,
    resp: &Resposta<'_>,
    html: &str,
    leitura: &crate::caixa_detalhada::LeituraDetalhada,
) -> Registro {
    let mut reg = cas.registro(Leitor::PaginaDetalhada, resp.passo, Resultado::Lido);
    reg.formas = controles(html, "Detalhado");
    let pos = match posicao(html, leitura) {
        Ok(pos) => pos,
        Err(erro) => {
            reg.resultado = Resultado::Recusado(format!("{erro:?}"));
            return reg;
        }
    };
    let paginador = if pos.paginador {
        "paginador_publicado"
    } else {
        "sem_paginador"
    };
    reg.formas.push(paginador.into());
    reg.contagens = vec![
        ("pagina_exibida", pos.indice as u64 + 1),
        ("paginas", pos.paginas as u64),
    ];
    match planejar(&pos, pos.indice + 1) {
        Ok(None) => {}
        Ok(Some(_)) => {
            reg.resultado = Resultado::Recusado("pagina_exibida_nao_reconhecida".to_owned());
        }
        Err(erro) => reg.resultado = Resultado::Recusado(format!("{erro:?}")),
    }
    if pos.paginas <= pos.indice + 1 || reg.resultado != Resultado::Lido {
        return reg;
    }
    match planejar(&pos, pos.indice + 2) {
        Ok(Some(pedido)) => {
            let (inicio, fim) = match pedido.esperada {
                crate::paginas::FaixaObservada::Lidas { inicio, fim } => (inicio, fim),
                crate::paginas::FaixaObservada::SemLinhas => (0, 0),
            };
            reg.contagens.extend([
                ("indice_pedido", pedido.indice as u64),
                ("esperada_inicio", inicio as u64),
                ("esperada_fim", fim as u64),
                ("tamanho", pos.tamanho.unwrap_or(0) as u64),
            ]);
        }
        Ok(None) => {
            reg.resultado = Resultado::Recusado("pagina_seguinte_dada_como_exibida".to_owned());
        }
        Err(erro) => reg.resultado = Resultado::Recusado(format!("{erro:?}")),
    }
    reg
}
