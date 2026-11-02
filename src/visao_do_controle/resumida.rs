//! `ies unidade recebidos|gerados [--pagina N]` quando a sessão está na D:
//! troca para R, lê (e pagina, se pedido, voltando à página encontrada da R)
//! e restaura a D.
use super::ErroVisaoDoControle as E;
use super::Visao;
use super::envio::{EnvioDoControle, Sessao, TelaAtual};
use super::passos::{
    conferir_detalhada_restaurada, conferir_resumida_restaurada, contexto, trocar, unidade_da_tela,
};
use crate::dominio::Categoria;
use crate::formulario_web::TelaServida;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::{
    PlanoDaPagina, codificacao_da_tela, obter_pagina_via, pagina_atual, planejar,
};
use std::cell::RefCell;
use std::path::Path;

/// HTML da visão R com a página pedida da categoria (ou a lembrada).
/// Orçamento: 1 GET + 2 POST (R, D), ou 1 GET + 4 POST (R, página N, página
/// encontrada, D) com a página N.
pub(crate) fn obter_na_visao_resumida<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: &TelaAtual,
    unidade_solicitada: Option<&str>,
    categoria: Categoria,
    pagina: Option<usize>,
    cookie_file: &Path,
    envio: &mut V,
) -> Result<String, E> {
    let encontrado = contexto(&tela.html)?;
    let unidade = unidade_da_tela(&encontrado, unidade_solicitada)?;
    if !encontrado.eh_detalhada() {
        return Err(E::VisaoDoControleNaoReconhecida);
    }
    let mut sessao = Sessao {
        instalacao,
        unidade: &unidade,
        cookie_file,
        envio,
    };
    let resumida = trocar(&mut sessao, tela, Visao::Resumida)?;
    let (ultima, lido) = pagina_da_categoria(&mut sessao, resumida, categoria, pagina);
    trocar(&mut sessao, &ultima, Visao::Detalhada)
        .and_then(|restaurada| conferir_detalhada_restaurada(&tela.html, &restaurada.html))
        .map_err(|causa| E::VisaoDoControleNaoRestaurada {
            visao_deixada: Visao::Resumida,
            causa: Box::new(causa),
        })?;
    lido
}

/// A R obtida ou a página N da categoria (o POST do paginador da R, como na
/// sessão R) e a volta à página encontrada (a da resposta do POST R);
/// devolve também a última R válida, de onde se restaura a D.
fn pagina_da_categoria<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    resumida: TelaAtual,
    categoria: Categoria,
    pagina: Option<usize>,
) -> (TelaAtual, Result<String, E>) {
    let Some(numero) = pagina else {
        let html = resumida.html.clone();
        return (resumida, Ok(html));
    };
    let encontrada = match planejar(&resumida.html, categoria, numero)
        .and_then(|plano| pagina_atual(&resumida.html, categoria).map(|i| (plano, i + 1)))
    {
        Ok((PlanoDaPagina::JaExibida, _)) => {
            let html = resumida.html.clone();
            return (resumida, Ok(html));
        }
        Ok((PlanoDaPagina::Pedir { .. }, encontrada)) => encontrada,
        Err(erro) => return (resumida, Err(E::Pagina(erro))),
    };
    let tela_n = match postar_pagina(sessao, &resumida, categoria, numero) {
        Ok(Some(tela_n)) => tela_n,
        Ok(None) => {
            let html = resumida.html.clone();
            return (resumida, Ok(html));
        }
        Err(erro) => return (resumida, Err(erro)),
    };
    let html = tela_n.html.clone();
    let volta = postar_pagina(sessao, &tela_n, categoria, encontrada).and_then(|volta| {
        let html_volta = volta
            .as_ref()
            .map_or(tela_n.html.as_str(), |v| v.html.as_str());
        conferir_resumida_restaurada(&resumida.html, html_volta).map(|()| volta)
    });
    match volta {
        Ok(Some(volta)) => (volta, Ok(html)),
        Ok(None) => (tela_n, Ok(html)),
        Err(causa) => (
            tela_n,
            Err(E::PaginaDaVisaoResumidaNaoRestaurada {
                categoria,
                pagina_deixada: numero,
                causa: Box::new(causa),
            }),
        ),
    }
}

/// Um POST do paginador da R a partir de `tela` (`None`: a tela já mostra
/// a página), conferido como na sessão R.
fn postar_pagina<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
    categoria: Categoria,
    numero: usize,
) -> Result<Option<TelaAtual>, E> {
    let codificacao = codificacao_da_tela(&tela.mime).ok_or(E::CharsetDaTelaNaoSuportado)?;
    let servida = TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao,
    };
    let url = RefCell::new(String::new());
    let mime = RefCell::new(String::new());
    let envio = &mut *sessao.envio;
    let obtida = obter_pagina_via(
        sessao.instalacao,
        &servida,
        sessao.unidade,
        categoria,
        numero,
        sessao.cookie_file,
        |submissao, cookie| {
            url.replace(submissao.url().to_owned());
            let resposta = envio.post(submissao, cookie)?;
            mime.replace(resposta.resposta.mime.clone());
            Ok(resposta)
        },
    )
    .map_err(E::Pagina)?;
    Ok(obtida.map(|html| TelaAtual {
        url: url.into_inner(),
        html,
        mime: mime.into_inner(),
    }))
}
