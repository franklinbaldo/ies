//! `ies unidade recebidos|gerados [--pagina N]` quando a sessão está na D:
//! troca para R e lê (e pagina, se pedido); a sessão fica na R, com a página
//! N lembrada (sem restaurar).
use super::envio::{EnvioDoControle, Sessao, TelaAtual};
use super::passos::{contexto, trocar, unidade_da_tela};
use super::{ErroVisaoDoControle, Visao};
use crate::dominio::Categoria;
use crate::formulario_web::TelaServida;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::{PlanoDaPagina, codificacao_da_tela, obter_pagina_via, planejar};
use crate::transporte::sessao::CookieDaSessao;

/// HTML da visão R com a página pedida da categoria (ou a lembrada).
/// Orçamento: 1 GET + 1 POST (R), ou 1 GET + 2 POST (R, página N).
pub(crate) fn obter_na_visao_resumida<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: &TelaAtual,
    unidade_solicitada: Option<&str>,
    categoria: Categoria,
    pagina: Option<usize>,
    cookie: &CookieDaSessao,
    envio: &mut V,
) -> Result<crate::tela_html::HtmlServido, super::ErroVisaoDoControle> {
    let encontrado = contexto(&tela.html)?;
    let unidade = unidade_da_tela(&encontrado, unidade_solicitada)?;
    if !encontrado.eh_detalhada() {
        return Err(ErroVisaoDoControle::VisaoDoControleNaoReconhecida);
    }
    let mut sessao = Sessao {
        instalacao,
        unidade: &unidade,
        cookie,
        envio,
    };
    let resumida = trocar(&mut sessao, tela, Visao::Resumida)?;
    let Some(numero) = pagina else {
        return Ok(resumida.html);
    };
    if planejar(&resumida.html, categoria, numero)? == PlanoDaPagina::JaExibida {
        return Ok(resumida.html);
    }
    Ok(postar_pagina(&mut sessao, &resumida, categoria, numero)?.unwrap_or(resumida.html))
}

/// Um POST do paginador da R a partir de `tela` (`None`: a tela já mostra
/// a página), conferido como na sessão R.
fn postar_pagina<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
    categoria: Categoria,
    numero: usize,
) -> Result<Option<crate::tela_html::HtmlServido>, super::ErroVisaoDoControle> {
    let codificacao = codificacao_da_tela(&tela.mime)
        .ok_or(super::ErroVisaoDoControle::CharsetDaTelaNaoSuportado)?;
    let servida = TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao,
    };
    let envio = &mut *sessao.envio;
    let pagina = obter_pagina_via(
        sessao.instalacao,
        &servida,
        sessao.unidade,
        categoria,
        numero,
        sessao.cookie,
        |submissao, cookie| envio.post(submissao, cookie),
    )?;
    if let Some(html) = &pagina {
        // O mesmo servidor e a mesma sessão: o charset da tela de origem.
        sessao.envio.colher_controle(html, Some(&tela.mime));
    }
    Ok(pagina)
}
