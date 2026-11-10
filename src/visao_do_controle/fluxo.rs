//! `ies unidade listar-processos [--pagina N]` na visão D; vinda da R, a
//! sessão fica na D, e a página N fica lembrada (sem restaurar). Orçamentos
//! no contrato da visão detalhada.
use super::envio::{EnvioDoControle, Sessao, TelaAtual};
use super::passos::{contexto, ler_detalhada, paginar_detalhada, trocar, unidade_da_tela};
use super::plano::{planejar, posicao};
use super::{ErroVisaoDoControle, Visao};
use crate::caixa_detalhada::LeituraDetalhada;
use crate::dominio::Categoria;
use crate::instalacao::Instalacao;
use crate::transporte::sessao::CookieDaSessao;
use std::collections::HashMap;

/// A lista D (página observada ou a página N), com a categoria das linhas
/// que as tabelas R do mesmo GET mostraram.
pub(crate) fn listar_na_visao_detalhada<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: &TelaAtual,
    unidade_solicitada: Option<&str>,
    pagina: Option<usize>,
    cookie: &CookieDaSessao,
    envio: &mut V,
) -> Result<LeituraDetalhada, super::ErroVisaoDoControle> {
    let encontrado = contexto(&tela.html)?;
    let unidade = unidade_da_tela(&encontrado, unidade_solicitada)?;
    let mut sessao = Sessao {
        instalacao,
        unidade: &unidade,
        cookie,
        envio,
    };
    if encontrado.eh_detalhada() {
        return na_detalhada(&mut sessao, tela, pagina);
    }
    if !encontrado.eh_resumida() {
        return Err(ErroVisaoDoControle::VisaoDoControleNaoReconhecida);
    }
    vindo_da_resumida(&mut sessao, tela, pagina)
}

/// A partir de uma tela D: 0 POST, ou o POST da página N.
fn na_detalhada<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
    pagina: Option<usize>,
) -> Result<LeituraDetalhada, super::ErroVisaoDoControle> {
    let leitura = ler_detalhada(&tela.html, sessao.unidade)?;
    let Some(numero) = pagina else {
        return Ok(leitura);
    };
    let encontrada = posicao(&tela.html, &leitura)?;
    let Some(pedido) = planejar(&encontrada, numero)? else {
        return Ok(leitura);
    };
    let (_, leitura_n) = paginar_detalhada(sessao, tela, pedido, encontrada.criterio.as_deref())?;
    Ok(leitura_n)
}

/// Sessão na R: categorias das tabelas R, POST D e, se pedido, a página N.
fn vindo_da_resumida<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
    pagina: Option<usize>,
) -> Result<LeituraDetalhada, super::ErroVisaoDoControle> {
    let resumida = crate::paginas::juntar(std::slice::from_ref(&tela.html), Some(sessao.unidade))?;
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
        return Err(super::ErroVisaoDoControle::PaginaForaDoIntervalo { solicitada: numero });
    }
    let detalhada = trocar(sessao, tela, Visao::Detalhada)?;
    let mut leitura = na_detalhada(sessao, &detalhada, pagina)?;
    for processo in &mut leitura.processos {
        processo.categoria = categorias.get(processo.id.as_str()).copied();
    }
    Ok(leitura)
}
