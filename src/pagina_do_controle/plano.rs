//! Plano de `--pagina N`, sem rede: tamanho da página pela faixa e pelo
//! índice publicados, número de páginas pelo total declarado.
use super::{ErroPaginaDoControle, PlanoDaPagina};
use crate::controle_contexto::contexto_da_pagina;
use crate::dominio::Categoria;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::CodificacaoDaTela;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::paginacao_controle::paginador_da_tabela;
use crate::paginas::{FaixaNaPagina, faixa_na_pagina};

/// Charset do `Content-Type` da tela, para codificar o corpo do POST.
/// Sem `charset` declarado, ou outro que não ISO-8859-1/UTF-8, não há
/// codificação comprovada.
pub(crate) fn codificacao_da_tela(mime: &str) -> Option<CodificacaoDaTela> {
    let charset = mime.split(';').skip(1).find_map(|parametro| {
        let (chave, valor) = parametro.split_once('=')?;
        chave
            .trim()
            .eq_ignore_ascii_case("charset")
            .then(|| valor.trim().trim_matches('"').to_ascii_lowercase())
    })?;
    match charset.as_str() {
        "iso-8859-1" => Some(CodificacaoDaTela::Iso88591),
        "utf-8" => Some(CodificacaoDaTela::Utf8),
        _ => None,
    }
}

pub(super) fn campo_da_pagina(categoria: Categoria) -> &'static str {
    match categoria {
        Categoria::Recebidos => "hdnRecebidosPaginaAtual",
        Categoria::Gerados => "hdnGeradosPaginaAtual",
    }
}

fn html_invalido(_: impl Sized) -> ErroPaginaDoControle {
    ErroPaginaDoControle::Caixa(ErroCaixa::HtmlDaCaixaInvalido)
}

/// Índice (a partir de 0) que a tela publica em `hdn{T}PaginaAtual`.
fn pagina_atual(html: &str, categoria: Categoria) -> Result<usize, ErroPaginaDoControle> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(html_invalido)?;
    let ilegivel = || ErroPaginaDoControle::PaginaAtualDaTabelaIlegivel { categoria };
    let nos = nos_com_id(&dom, campo_da_pagina(categoria)).map_err(html_invalido)?;
    let [no] = nos.as_slice() else {
        return Err(if nos.is_empty() {
            ilegivel()
        } else {
            ErroPaginaDoControle::CamposDePaginaAtualDaTabelaRepetidos { categoria }
        });
    };
    let tag = no
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or_else(ilegivel)?;
    let valor = valor_do_atributo(tag, "value")
        .map_err(html_invalido)?
        .unwrap_or_default();
    if valor.is_empty() || !valor.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ilegivel());
    }
    valor
        .parse()
        .map_err(|erro| ErroPaginaDoControle::PaginaAtualDaTabelaNaoCabeNoTipo { categoria, erro })
}

/// Tabela sem linhas: só a página 1 (vazia) existe.
fn plano_sem_linhas(
    html: &str,
    categoria: Categoria,
    atual: usize,
    pagina: usize,
) -> Result<PlanoDaPagina, ErroPaginaDoControle> {
    let caixa = crate::caixa::parse_html(html)?;
    let linhas = match categoria {
        Categoria::Recebidos => caixa.recebidos.processos.len(),
        Categoria::Gerados => caixa.gerados.processos.len(),
    };
    if linhas > 0 {
        return Err(ErroPaginaDoControle::Caixa(
            ErroCaixa::LegendaSemFaixaComTotalDiferenteDasLinhas { categoria },
        ));
    }
    if atual != 0 {
        return Err(ErroPaginaDoControle::PaginaAtualDaTabelaIncompativelComAFaixa { categoria });
    }
    if pagina == 1 {
        Ok(PlanoDaPagina::JaExibida)
    } else {
        Err(ErroPaginaDoControle::PaginaForaDoIntervalo {
            categoria,
            solicitada: pagina,
            paginas: 1,
        })
    }
}

/// Tamanho da página pelo índice publicado (a partir de 0) e pela faixa:
/// índice 0 começa em 1; índice k > 0 começa em k * tamanho + 1. `None`
/// quando índice e faixa não se combinam. Vale para R e D.
pub(crate) fn tamanho_da_pagina(atual: usize, faixa: FaixaNaPagina) -> Option<usize> {
    let linhas = faixa.fim.checked_sub(faixa.inicio)? + 1;
    let tamanho = match (atual, faixa.inicio.checked_sub(1)?) {
        (0, 0) => linhas,
        (0, _) | (_, 0) => return None,
        (k, antes) if antes % k == 0 => antes / k,
        _ => return None,
    };
    if linhas > tamanho || (faixa.fim < faixa.total && linhas != tamanho) {
        return None;
    }
    Some(tamanho)
}

/// Calcula, sem rede, se e como pedir a página `pagina` (≥ 1).
pub(crate) fn planejar(
    html: &str,
    categoria: Categoria,
    pagina: usize,
) -> Result<PlanoDaPagina, ErroPaginaDoControle> {
    let contexto = contexto_da_pagina(html)?;
    if !contexto.eh_resumida() {
        return Err(ErroPaginaDoControle::PaginaExigeVisaoResumida);
    }
    let atual = pagina_atual(html, categoria)?;
    let Some(faixa) = faixa_na_pagina(html, categoria)? else {
        return plano_sem_linhas(html, categoria, atual, pagina);
    };
    let tamanho = tamanho_da_pagina(atual, faixa)
        .ok_or(ErroPaginaDoControle::PaginaAtualDaTabelaIncompativelComAFaixa { categoria })?;
    let paginas = faixa.total.div_ceil(tamanho);
    if pagina == 0 || pagina > paginas {
        return Err(ErroPaginaDoControle::PaginaForaDoIntervalo {
            categoria,
            solicitada: pagina,
            paginas,
        });
    }
    if pagina == atual + 1 {
        return Ok(PlanoDaPagina::JaExibida);
    }
    if !paginador_da_tabela(html, categoria)? {
        return Err(ErroPaginaDoControle::PaginadorDaTabelaNaoPublicado { categoria });
    }
    let indice = pagina - 1;
    Ok(PlanoDaPagina::Pedir {
        indice,
        esperada: FaixaNaPagina {
            total: faixa.total,
            inicio: indice * tamanho + 1,
            fim: faixa.total.min(pagina * tamanho),
        },
    })
}
