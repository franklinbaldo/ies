//! `ies unidade listar-processos [--pagina N]` na visão D, deixando a tela
//! como foi encontrada. Orçamentos no contrato da visão detalhada.
use super::ErroVisaoDoControle as E;
use super::Visao;
use super::envio::{EnvioDoControle, Sessao, TelaAtual};
use super::passos::{
    conferir_resumida_restaurada, contexto, ler_detalhada, paginar_detalhada, trocar,
    unidade_da_tela, voltar_a_pagina_encontrada,
};
use super::plano::{planejar, posicao};
use crate::caixa_detalhada::LeituraDetalhada;
use crate::dominio::Categoria;
use crate::instalacao::Instalacao;
use std::collections::HashMap;
use std::path::Path;

/// A lista D (página observada ou a página N), com a categoria das linhas
/// que as tabelas R do mesmo GET mostraram.
pub(crate) fn listar_na_visao_detalhada<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: &TelaAtual,
    unidade_solicitada: Option<&str>,
    pagina: Option<usize>,
    cookie_file: &Path,
    envio: &mut V,
) -> Result<LeituraDetalhada, E> {
    let encontrado = contexto(&tela.html)?;
    let unidade = unidade_da_tela(&encontrado, unidade_solicitada)?;
    let mut sessao = Sessao {
        instalacao,
        unidade: &unidade,
        cookie_file,
        envio,
    };
    if encontrado.eh_detalhada() {
        return na_detalhada(&mut sessao, tela, pagina);
    }
    if !encontrado.eh_resumida() {
        return Err(E::VisaoDoControleNaoReconhecida);
    }
    vindo_da_resumida(&mut sessao, tela, pagina)
}

/// Sessão já na D: 0 POST, ou página N e volta à página exibida (2 POST).
fn na_detalhada<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
    pagina: Option<usize>,
) -> Result<LeituraDetalhada, E> {
    let leitura = ler_detalhada(&tela.html, sessao.unidade)?;
    let Some(numero) = pagina else {
        return Ok(leitura);
    };
    let encontrada = posicao(&tela.html, &leitura)?;
    let Some(pedido) = planejar(&encontrada, numero)? else {
        return Ok(leitura);
    };
    let criterio = encontrada.criterio.as_deref();
    let (tela_n, leitura_n) = paginar_detalhada(sessao, tela, pedido, criterio)?;
    voltar_a_pagina_encontrada(sessao, &tela_n, &tela.html, &encontrada).map_err(|causa| {
        E::PaginaDaVisaoDetalhadaNaoRestaurada {
            visao_deixada: Visao::Detalhada,
            pagina_deixada: numero,
            causa: Box::new(causa),
        }
    })?;
    Ok(leitura_n)
}

/// Sessão na R: POST D, página N e volta à página encontrada se preciso,
/// POST R; categorias das tabelas R.
fn vindo_da_resumida<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
    pagina: Option<usize>,
) -> Result<LeituraDetalhada, E> {
    let resumida = crate::paginas::juntar(std::slice::from_ref(&tela.html), Some(sessao.unidade))
        .map_err(E::Caixa)?;
    let mut categorias: HashMap<String, Categoria> = HashMap::new();
    let mut total = 0usize;
    for (categoria, tabela) in [
        (Categoria::Recebidos, &resumida.caixa.recebidos),
        (Categoria::Gerados, &resumida.caixa.gerados),
    ] {
        total += tabela.total_declarado.unwrap_or(tabela.processos.len());
        for processo in &tabela.processos {
            categorias.insert(processo.id.as_str().to_owned(), categoria);
        }
    }
    // Limite seguro antes de qualquer POST: ao menos um processo por página.
    if let Some(numero) = pagina
        && (numero == 0 || numero > total.max(1))
    {
        return Err(E::PaginaForaDoIntervalo { solicitada: numero });
    }
    let detalhada = trocar(sessao, tela, Visao::Detalhada)?;
    let (ultima, lido) = ler_depois_da_troca(sessao, detalhada, pagina);
    trocar(sessao, &ultima, Visao::Resumida)
        .and_then(|restaurada| conferir_resumida_restaurada(&tela.html, &restaurada.html))
        .map_err(|causa| E::VisaoDoControleNaoRestaurada {
            visao_deixada: Visao::Detalhada,
            causa: Box::new(causa),
        })?;
    let mut leitura = lido?;
    for processo in &mut leitura.processos {
        processo.categoria = categorias.get(processo.id.as_str()).copied();
    }
    Ok(leitura)
}

/// Lê a D obtida e, se pedido, a página N e volta à página encontrada (a da
/// resposta do POST D); devolve a última D válida (de onde se restaura a R)
/// e o resultado, mesmo em falha.
fn ler_depois_da_troca<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    detalhada: TelaAtual,
    pagina: Option<usize>,
) -> (TelaAtual, Result<LeituraDetalhada, E>) {
    let leitura = match ler_detalhada(&detalhada.html, sessao.unidade) {
        Ok(leitura) => leitura,
        Err(erro) => return (detalhada, Err(erro)),
    };
    let Some(numero) = pagina else {
        return (detalhada, Ok(leitura));
    };
    let (encontrada, pedido) = match posicao(&detalhada.html, &leitura)
        .and_then(|p| planejar(&p, numero).map(|pedido| (p, pedido)))
    {
        Ok((_, None)) => return (detalhada, Ok(leitura)),
        Ok((encontrada, Some(pedido))) => (encontrada, pedido),
        Err(erro) => return (detalhada, Err(erro)),
    };
    let criterio = encontrada.criterio.as_deref();
    let (tela_n, leitura_n) = match paginar_detalhada(sessao, &detalhada, pedido, criterio) {
        Ok(lido) => lido,
        Err(erro) => return (detalhada, Err(erro)),
    };
    match voltar_a_pagina_encontrada(sessao, &tela_n, &detalhada.html, &encontrada) {
        Ok(volta) => (volta, Ok(leitura_n)),
        Err(causa) => (
            tela_n,
            Err(E::PaginaDaVisaoDetalhadaNaoRestaurada {
                visao_deixada: Visao::Resumida,
                pagina_deixada: numero,
                causa: Box::new(causa),
            }),
        ),
    }
}
