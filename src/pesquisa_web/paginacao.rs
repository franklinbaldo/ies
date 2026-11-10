//! `--pagina N` da pesquisa: `navegar(inicio)` da tela de resultado.
//! Regra servida (`UnB` 4.0.9, 03/11/2026, captura privada só estrutural):
//! `document.getElementById('hdnInicio').value = inicio;`, `onSubmitForm()` e
//! `document.getElementById('frmPesquisaProtocolo').submit();` (sem botão),
//! com links `javascript:navegar('10')`…`navegar('90')` e "Próxima".
use super::formulario_pesquisa::CAMPOS_OCULTOS_DA_PESQUISA;
#[cfg(test)]
use super::resposta::ListaDaPesquisa;
use super::resposta::{FaixaDaPesquisa, RespostaDaPesquisa};
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    decodificar_atributo, localizar_formulario,
};
use crate::html_atributos::valor_do_atributo;

/// `navegar(inicio)`: `submit()` por script, só `hdnInicio` reescrito.
pub(crate) const PAGINAR_PESQUISA_DE_PROTOCOLO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "protocolo_pesquisar",
    formulario: "frmPesquisaProtocolo",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: CAMPOS_OCULTOS_DA_PESQUISA,
    campos_ocultos_reescritos: &["hdnInicio"],
    campos_usuario: super::formulario_pesquisa::CAMPOS_USUARIO_DA_PESQUISA,
    resultado: super::formulario_pesquisa::MARCADORES_DA_RESPOSTA,
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

/// Uma variante por fato verificável da paginação da pesquisa.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroPaginacaoDaPesquisa {
    PaginaForaDoIntervalo { solicitada: u32, paginas: u64 },
    RegraDeNavegacaoDaPesquisaNaoObservada,
    PaginadorDaPesquisaNaoPublicado,
    ProtocoloDaTelaDeResultadoDiferenteDoPesquisado,
    RedirecionamentoAoPaginarAPesquisa,
    PaginaDaPesquisaSemResultadoAposPaginar,
    TotalDaPesquisaAlteradoAposPaginar,
    FaixaDaPesquisaDiferenteDaPaginaSolicitada,
    Html(#[from] ErroHtml),
}

/// O que a paginação precisa da resposta da página 1, em processos ou em
/// documentos: a forma e, na lista, a faixa e o total da barra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Contagem {
    SemResultado,
    /// 302 interpretado: uma página só.
    Unico,
    Lista {
        faixa: FaixaDaPesquisa,
        total: u64,
    },
}

impl RespostaDaPesquisa {
    pub(crate) fn contagem(&self) -> Contagem {
        match self {
            Self::SemResultado => Contagem::SemResultado,
            Self::ProcessoUnico { .. } => Contagem::Unico,
            Self::Lista(lista) => Contagem::Lista {
                faixa: lista.faixa,
                total: lista.total_declarado,
            },
        }
    }
}

/// O POST de página a enviar: `hdnInicio` e a faixa que a resposta deve ter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PedidoDePaginaDaPesquisa {
    /// A página pedida (N de `--pagina N`).
    pub(crate) numero: u64,
    pub(crate) inicio: u64,
    pub(crate) total: u64,
    pub(crate) esperada: FaixaDaPesquisa,
}

/// Página N a partir da resposta da página 1, sem rede. `Ok(None)`: a
/// resposta já é a página pedida (N = 1). N fora do intervalo é erro antes
/// de qualquer POST de página.
#[cfg(test)]
pub(crate) fn planejar_pagina(
    resposta: &RespostaDaPesquisa,
    html_da_resposta: Option<&str>,
    solicitada: u32,
) -> Result<Option<PedidoDePaginaDaPesquisa>, ErroPaginacaoDaPesquisa> {
    planejar_pagina_por(resposta.contagem(), html_da_resposta, solicitada)
}

/// [`planejar_pagina`] pela contagem da página 1 (processos ou documentos).
pub(crate) fn planejar_pagina_por(
    contagem: Contagem,
    html_da_resposta: Option<&str>,
    solicitada: u32,
) -> Result<Option<PedidoDePaginaDaPesquisa>, ErroPaginacaoDaPesquisa> {
    use ErroPaginacaoDaPesquisa as E;
    if solicitada <= 1 {
        return Ok(None);
    }
    let fora = |paginas| E::PaginaForaDoIntervalo {
        solicitada,
        paginas,
    };
    let (faixa, total) = match contagem {
        Contagem::SemResultado => return Err(fora(0)),
        Contagem::Unico => return Err(fora(1)),
        Contagem::Lista { faixa, total } => (faixa, total),
    };
    // O POST da pesquisa envia `hdnInicio=0`: a página 1 começa em 1.
    if faixa.inicio != 1 {
        return Err(E::FaixaDaPesquisaDiferenteDaPaginaSolicitada);
    }
    if faixa.fim >= total {
        return Err(fora(1));
    }
    // Página 1 incompleta só existe quando é a única (`fim = total`).
    let tamanho = faixa.fim;
    let paginas = total.div_ceil(tamanho);
    if u64::from(solicitada) > paginas {
        return Err(fora(paginas));
    }
    let html = html_da_resposta.ok_or(E::RegraDeNavegacaoDaPesquisaNaoObservada)?;
    if !REGRA_DE_NAVEGAR.iter().all(|linha| html.contains(linha)) {
        return Err(E::RegraDeNavegacaoDaPesquisaNaoObservada);
    }
    conferir_links_de_navegar(html, tamanho, total)?;
    let inicio = (u64::from(solicitada) - 1) * tamanho;
    Ok(Some(PedidoDePaginaDaPesquisa {
        numero: u64::from(solicitada),
        inicio,
        total,
        esperada: FaixaDaPesquisa {
            inicio: inicio + 1,
            fim: (inicio + tamanho).min(total),
        },
    }))
}

/// Linhas literais de `navegar(inicio)` servidas (sem interpretar JS).
const REGRA_DE_NAVEGAR: [&str; 3] = [
    "function navegar(inicio)",
    "document.getElementById('hdnInicio').value = inicio;",
    "document.getElementById('frmPesquisaProtocolo').submit();",
];

/// Todo `javascript:navegar(…)` publicado tem a forma `navegar('K')`, K
/// decimal, múltiplo do tamanho e menor que o total; pelo menos um existe.
fn conferir_links_de_navegar(
    html: &str,
    tamanho: u64,
    total: u64,
) -> Result<(), ErroPaginacaoDaPesquisa> {
    use ErroPaginacaoDaPesquisa as E;
    let ilegivel = E::Html(ErroHtml::html(
        "html_da_tela_de_resultado_da_pesquisa_nao_interpretado",
    ));
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ilegivel)?;
    let parser = dom.parser();
    let mut publicados = 0;
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(bruto) = valor_do_atributo(tag, "href").map_err(|_| ilegivel)? else {
            continue;
        };
        let href = decodificar_atributo(&bruto).ok_or(ilegivel)?;
        let Some(resto) = href.strip_prefix("javascript:navegar(") else {
            continue;
        };
        let na_regra = resto
            .strip_prefix('\'')
            .and_then(|r| r.strip_suffix("')"))
            .filter(|k| !k.is_empty() && k.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|k| k.parse::<u64>().ok())
            .is_some_and(|k| k % tamanho == 0 && k < total);
        if !na_regra {
            return Err(ErroHtml::valor("link_de_paginacao_da_pesquisa_fora_da_regra").into());
        }
        publicados += 1;
    }
    if publicados == 0 {
        return Err(E::PaginadorDaPesquisaNaoPublicado);
    }
    Ok(())
}

/// O termo servido em `txtProtocoloPesquisa` da tela de resultado é o
/// pesquisado: o POST de página repete o formulário servido.
pub(crate) fn conferir_protocolo_servido(
    html: &str,
    protocolo: &str,
) -> Result<(), ErroPaginacaoDaPesquisa> {
    use ErroPaginacaoDaPesquisa as E;
    let ilegivel = E::Html(ErroHtml::html(
        "html_da_tela_de_resultado_da_pesquisa_nao_interpretado",
    ));
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ilegivel)?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, PAGINAR_PESQUISA_DE_PROTOCOLO.formulario)
        .map_err(|_| ilegivel)?;
    let mut servidos = Vec::new();
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter(|t| t.name().as_utf8_str().eq_ignore_ascii_case("input"))
    {
        if valor_do_atributo(tag, "name")
            .map_err(|_| ilegivel)?
            .as_deref()
            == Some("txtProtocoloPesquisa")
        {
            let bruto = valor_do_atributo(tag, "value").map_err(|_| ilegivel)?;
            servidos.push(decodificar_atributo(&bruto.unwrap_or_default()).ok_or(ilegivel)?);
        }
    }
    if servidos != [protocolo] {
        return Err(E::ProtocoloDaTelaDeResultadoDiferenteDoPesquisado);
    }
    Ok(())
}

/// Confere a resposta do POST de página contra o pedido.
#[cfg(test)]
pub(crate) fn conferir_pagina(
    pedido: &PedidoDePaginaDaPesquisa,
    resposta: RespostaDaPesquisa,
) -> Result<ListaDaPesquisa, ErroPaginacaoDaPesquisa> {
    conferir_contagem(pedido, resposta.contagem())?;
    match resposta {
        RespostaDaPesquisa::Lista(lista) => Ok(lista),
        _ => unreachable!("só a lista passa na conferência"),
    }
}

/// [`conferir_pagina`] pela contagem (processos ou documentos).
pub(crate) fn conferir_contagem(
    pedido: &PedidoDePaginaDaPesquisa,
    contagem: Contagem,
) -> Result<(), ErroPaginacaoDaPesquisa> {
    use ErroPaginacaoDaPesquisa as E;
    match contagem {
        Contagem::Unico => Err(E::RedirecionamentoAoPaginarAPesquisa),
        Contagem::SemResultado => Err(E::PaginaDaPesquisaSemResultadoAposPaginar),
        Contagem::Lista { total, .. } if total != pedido.total => {
            Err(E::TotalDaPesquisaAlteradoAposPaginar)
        }
        Contagem::Lista { faixa, .. } if faixa != pedido.esperada => {
            Err(E::FaixaDaPesquisaDiferenteDaPaginaSolicitada)
        }
        Contagem::Lista { .. } => Ok(()),
    }
}

#[cfg(test)]
#[path = "tests_paginacao.rs"]
mod tests;
