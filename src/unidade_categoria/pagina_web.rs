//! `recebidos|gerados [--pagina N]`: um GET e, se a tela não mostra a página
//! N, um POST do paginador de T. Sem retry, sem outras páginas, sem restaurar
//! a página da categoria. Se a sessão está na visão D, a troca para R (sem
//! volta) fica com `visao_do_controle`.
use crate::colheita_do_controle::{Colheita, EnvioColhendo};
use crate::configuracao::ArquivosDeAcesso;
use crate::dominio::Categoria;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::TelaServida;
use crate::indice_local::IndiceLocal;
use crate::pagina_do_controle::{
    self, ErroPaginaDoControle, PlanoDaPagina, codificacao_da_tela, obter_pagina_via,
};
use crate::paginas;
use crate::transporte::FalhaTransporte;
use crate::transporte::envio_post::{TelaDoControle, enviar_post, ler_tela_do_controle};
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::{self, ErroVisaoDoControle, TelaAtual};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(super) enum ErroPaginaWeb {
    Transporte(#[from] FalhaTransporte),
    Caixa(#[from] ErroCaixa),
    Pagina(#[from] ErroPaginaDoControle),
    Visao(#[from] ErroVisaoDoControle),
}

impl ErroPaginaWeb {
    /// A sessão do SEI acabou (decide o relogin único): por `match` no tipo, sem
    /// percorrer a cadeia `source()`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Transporte(erro) => erro.sessao_encerrada(),
            Self::Pagina(erro) => erro.sessao_encerrada(),
            Self::Visao(erro) => erro.sessao_encerrada(),
            Self::Caixa(_) => false,
        }
    }
}

/// HTML da visão R com a página observada ou a N da categoria: um GET e,
/// na visão D, a troca para R, que fica (orçamento no contrato). Cada tela
/// do Controle lida é colhida no índice local, sem requisição a mais.
pub(super) fn obter(
    arquivos: &ArquivosDeAcesso,
    indice: &IndiceLocal,
    unidade: Option<&str>,
    categoria: Categoria,
    pagina: Option<usize>,
) -> Result<crate::tela_html::HtmlServido, ErroPaginaWeb> {
    let sessao =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let colheita = Colheita::das_referencias(indice, &sessao);
    let tela = ler_tela_do_controle(&sessao)?;
    colheita.colher(&tela.html, Some(&tela.mime));
    let detalhada = crate::controle_contexto::contexto_da_pagina(&tela.html)
        .is_ok_and(|contexto| contexto.eh_detalhada());
    if detalhada {
        let atual = TelaAtual {
            url: tela.url,
            html: tela.html,
            mime: tela.mime,
        };
        return visao_do_controle::obter_na_visao_resumida(
            &tela.instalacao,
            &atual,
            unidade,
            categoria,
            pagina,
            sessao.cookie(),
            &mut visao_do_controle::EnvioContado::novo(EnvioColhendo {
                colheita: &colheita,
            }),
        )
        .map_err(ErroPaginaWeb::Visao);
    }
    match pagina {
        None => Ok(tela.html),
        Some(numero) => obter_da_tela(tela, &sessao, unidade, categoria, numero, &colheita),
    }
}

/// A página N a partir da tela R já obtida: zero ou um POST.
fn obter_da_tela(
    tela: TelaDoControle,
    sessao: &ReferenciasDaSessao,
    unidade: Option<&str>,
    categoria: Categoria,
    numero: usize,
    colheita: &Colheita<'_>,
) -> Result<crate::tela_html::HtmlServido, ErroPaginaWeb> {
    // A tela de origem passa pelas mesmas recusas da leitura sem opção.
    let leitura = paginas::juntar(std::slice::from_ref(&tela.html), unidade)?;
    let unidade_da_tela = leitura.unidade.ok_or(ErroPaginaWeb::Caixa(
        ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle,
    ))?;
    let plano = pagina_do_controle::planejar(&tela.html, categoria, numero)?;
    if plano == PlanoDaPagina::JaExibida {
        return Ok(tela.html);
    }
    let codificacao = codificacao_da_tela(&tela.mime).ok_or(ErroPaginaWeb::Pagina(
        ErroPaginaDoControle::CharsetDaTelaNaoSuportado,
    ))?;
    let servida = TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao,
    };
    let obtida = obter_pagina_via(
        &tela.instalacao,
        &servida,
        &unidade_da_tela,
        categoria,
        numero,
        sessao.cookie(),
        enviar_post,
    )?;
    if let Some(html) = &obtida {
        // O mesmo servidor e a mesma sessão: o charset da tela de origem.
        colheita.colher(html, Some(&tela.mime));
    }
    Ok(obtida.unwrap_or(tela.html))
}
