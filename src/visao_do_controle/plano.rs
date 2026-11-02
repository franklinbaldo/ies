//! Posição e plano de página da visão D e leitura dos ocultos que a
//! paginação confere, sem rede.
use super::ErroVisaoDoControle as E;
use crate::caixa_detalhada::LeituraDetalhada;
use crate::erro_web::ErroCaixa;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::paginas::FaixaNaPagina;

fn html_invalido(_: impl Sized) -> E {
    E::Caixa(ErroCaixa::HtmlInvalido)
}

/// Valores de `value` dos elementos com o `id` dado, na ordem do documento.
pub(super) fn valores_ocultos(html: &str, id: &str) -> Result<Vec<String>, E> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(html_invalido)?;
    let parser = dom.parser();
    let mut valores = Vec::new();
    for no in nos_com_id(&dom, id).map_err(html_invalido)? {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or_else(|| html_invalido(()))?;
        valores.push(
            valor_do_atributo(tag, "value")
                .map_err(html_invalido)?
                .unwrap_or_default(),
        );
    }
    Ok(valores)
}

/// Onde a tela D está e quantas páginas a lista D tem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PosicaoDetalhada {
    pub(super) total: usize,
    pub(super) faixa: Option<(usize, usize)>,
    /// `hdnDetalhadoPaginaAtual` (a partir de 0).
    pub(super) indice: usize,
    pub(super) tamanho: Option<usize>,
    pub(super) paginas: usize,
    pub(super) criterio: Option<String>,
    pub(super) paginador: bool,
}

/// Lê a posição da tela D já interpretada em `leitura`.
pub(super) fn posicao(html: &str, leitura: &LeituraDetalhada) -> Result<PosicaoDetalhada, E> {
    let indices = valores_ocultos(html, "hdnDetalhadoPaginaAtual")?;
    let total = leitura.total_declarado;
    let faixa = leitura.faixa_observada;
    let incompativel = E::PaginaAtualDaVisaoDetalhadaIncompativelComAFaixa;
    let indice = match indices.as_slice() {
        // UnB 4.0.9: sem o campo, sem paginação; só a página 1 inteira.
        [] => match faixa {
            None => 0,
            Some((1, fim)) if fim == total => 0,
            Some(_) => return Err(E::PaginaAtualDaVisaoDetalhadaIlegivel),
        },
        [valor] => {
            if valor.is_empty() || valor.len() > 9 || !valor.bytes().all(|b| b.is_ascii_digit()) {
                return Err(E::PaginaAtualDaVisaoDetalhadaIlegivel);
            }
            valor
                .parse()
                .map_err(|_| E::PaginaAtualDaVisaoDetalhadaIlegivel)?
        }
        _ => return Err(E::PaginaAtualDaVisaoDetalhadaIlegivel),
    };
    let (tamanho, paginas) = match faixa {
        None if indice == 0 => (None, 1),
        None => return Err(incompativel),
        Some((inicio, fim)) => {
            let tamanho = crate::pagina_do_controle::tamanho_da_pagina(
                indice,
                FaixaNaPagina { total, inicio, fim },
            )
            .ok_or(incompativel)?;
            (Some(tamanho), total.div_ceil(tamanho).max(1))
        }
    };
    let criterio = valores_ocultos(html, "hdnDetalhadoHashCriterios")?
        .into_iter()
        .next();
    let paginador =
        crate::paginacao_controle::paginador_com_prefixo(html, "Detalhado").map_err(E::Caixa)?;
    Ok(PosicaoDetalhada {
        total,
        faixa,
        indice,
        tamanho,
        paginas,
        criterio,
        paginador,
    })
}

/// Índice a pedir e a faixa que a resposta precisa trazer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PedidoDePagina {
    pub(super) indice: usize,
    pub(super) esperada: Option<(usize, usize)>,
    pub(super) total: usize,
}

/// `None`: a tela já mostra a página N, nenhum POST.
pub(super) fn planejar(
    posicao: &PosicaoDetalhada,
    pagina: usize,
) -> Result<Option<PedidoDePagina>, E> {
    if pagina == 0 || pagina > posicao.paginas {
        return Err(E::PaginaForaDoIntervalo { solicitada: pagina });
    }
    if pagina == posicao.indice + 1 {
        return Ok(None);
    }
    let Some(tamanho) = posicao.tamanho else {
        return Err(E::PaginaForaDoIntervalo { solicitada: pagina });
    };
    if !posicao.paginador {
        return Err(E::PaginadorDaVisaoDetalhadaNaoPublicado);
    }
    let indice = pagina - 1;
    Ok(Some(PedidoDePagina {
        indice,
        esperada: Some((indice * tamanho + 1, posicao.total.min(pagina * tamanho))),
        total: posicao.total,
    }))
}
