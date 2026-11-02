//! `ies unidade recebidos|gerados [--pagina N]` quando a sessão está na D:
//! troca para R, lê (e pagina, se pedido) e restaura a D.
use super::ErroVisaoDoControle as E;
use super::Visao;
use super::envio::{EnvioDoControle, Sessao, TelaAtual};
use super::passos::{conferir_detalhada_restaurada, contexto, trocar, unidade_da_tela};
use crate::dominio::Categoria;
use crate::formulario_web::TelaServida;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::{PlanoDaPagina, codificacao_da_tela, obter_pagina_via, planejar};
use std::cell::RefCell;
use std::path::Path;

/// HTML da visão R com a página pedida da categoria (ou a lembrada).
/// Orçamento: 1 GET + 2 POST (R, D), ou + 3 POST com a página N.
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
/// sessão R); devolve também a última R, de onde se restaura a D.
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
    match planejar(&resumida.html, categoria, numero) {
        Ok(PlanoDaPagina::JaExibida) => {
            let html = resumida.html.clone();
            return (resumida, Ok(html));
        }
        Ok(PlanoDaPagina::Pedir { .. }) => {}
        Err(erro) => return (resumida, Err(E::Pagina(erro))),
    }
    let Some(codificacao) = codificacao_da_tela(&resumida.mime) else {
        return (resumida, Err(E::CharsetDaTelaNaoSuportado));
    };
    let servida = TelaServida {
        url: &resumida.url,
        html: &resumida.html,
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
    );
    match obtida {
        Ok(Some(html)) => (
            TelaAtual {
                url: url.into_inner(),
                html: html.clone(),
                mime: mime.into_inner(),
            },
            Ok(html),
        ),
        Ok(None) => {
            let html = resumida.html.clone();
            (resumida, Ok(html))
        }
        Err(erro) => (resumida, Err(E::Pagina(erro))),
    }
}
