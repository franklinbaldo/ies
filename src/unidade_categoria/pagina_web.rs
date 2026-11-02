//! `recebidos|gerados [--pagina N]`: um GET e, se a tela não mostra a página
//! N, um POST do paginador de T. Sem retry, sem outras páginas, sem restaurar
//! a página da categoria. Se a sessão está na visão D, a troca para R (sem
//! volta) fica com `visao_do_controle`.
use crate::configuracao::ArquivosDeAcesso;
use crate::dominio::Categoria;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::TelaServida;
use crate::pagina_do_controle::{
    self, ErroPaginaDoControle, PlanoDaPagina, codificacao_da_tela, obter_pagina_via,
};
use crate::paginas;
use crate::transporte::FalhaTransporte;
use crate::transporte::envio_post::{TelaDoControle, enviar_post, ler_tela_do_controle};
use crate::visao_do_controle::{self, EnvioHttps, ErroVisaoDoControle, TelaAtual};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ErroPaginaWeb {
    Transporte(FalhaTransporte),
    Caixa(ErroCaixa),
    Pagina(ErroPaginaDoControle),
    Visao(ErroVisaoDoControle),
}

impl ErroPaginaWeb {
    pub(super) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Transporte(erro) => erro.subcodigo(),
            Self::Caixa(erro) => erro.subcodigo(),
            Self::Pagina(erro) => erro.subcodigo(),
            Self::Visao(erro) => erro.subcodigo(),
        }
    }

    pub(super) fn mensagem(&self) -> &'static str {
        match self {
            Self::Transporte(erro) => erro.mensagem(),
            Self::Caixa(_) => "Controle de Processos não reconhecido",
            Self::Pagina(_) => "a página pedida do Controle não foi obtida",
            Self::Visao(erro) => erro.mensagem(),
        }
    }

    /// Status e mensagem do SEI, do GET ou do POST do paginador.
    pub(super) fn status_http(&self) -> Option<&crate::transporte::StatusHttpRecusado> {
        match self {
            Self::Transporte(erro) => erro.status_http(),
            Self::Pagina(erro) => erro.status_http(),
            Self::Visao(erro) => erro.status_http(),
            Self::Caixa(_) => None,
        }
    }
}

/// HTML da visão R com a página observada ou a N da categoria: um GET e,
/// na visão D, a troca para R, que fica (orçamento no contrato).
pub(super) fn obter(
    arquivos: &ArquivosDeAcesso,
    unidade: Option<&str>,
    categoria: Categoria,
    pagina: Option<usize>,
) -> Result<String, ErroPaginaWeb> {
    let tela = ler_tela_do_controle(&arquivos.url_controle, &arquivos.cookie, &arquivos.origem)
        .map_err(ErroPaginaWeb::Transporte)?;
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
            &arquivos.cookie,
            &mut EnvioHttps,
        )
        .map_err(ErroPaginaWeb::Visao);
    }
    match pagina {
        None => Ok(tela.html),
        Some(numero) => obter_da_tela(tela, arquivos, unidade, categoria, numero),
    }
}

/// A página N a partir da tela R já obtida: zero ou um POST.
fn obter_da_tela(
    tela: TelaDoControle,
    arquivos: &ArquivosDeAcesso,
    unidade: Option<&str>,
    categoria: Categoria,
    numero: usize,
) -> Result<String, ErroPaginaWeb> {
    // A tela de origem passa pelas mesmas recusas da leitura sem opção.
    let leitura =
        paginas::juntar(std::slice::from_ref(&tela.html), unidade).map_err(ErroPaginaWeb::Caixa)?;
    let unidade_da_tela = leitura.unidade.ok_or(ErroPaginaWeb::Caixa(
        ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle,
    ))?;
    let plano = pagina_do_controle::planejar(&tela.html, categoria, numero)
        .map_err(ErroPaginaWeb::Pagina)?;
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
        &arquivos.cookie,
        enviar_post,
    )
    .map_err(ErroPaginaWeb::Pagina)?;
    Ok(obtida.unwrap_or(tela.html))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulario_web::ErroSubmissaoFormulario;
    use crate::transporte::StatusHttpRecusado;

    fn recusa(status: u16) -> FalhaTransporte {
        FalhaTransporte::StatusHttpDeErroDoServidor(StatusHttpRecusado {
            status,
            mensagem_do_servidor: Some("x".to_owned()),
        })
    }

    #[test]
    fn status_do_get_e_do_post_chegam_a_borda() {
        let get = ErroPaginaWeb::Transporte(recusa(500));
        assert_eq!(get.status_http().map(|h| h.status), Some(500));
        let post = ErroPaginaWeb::Pagina(ErroPaginaDoControle::Submissao(
            ErroSubmissaoFormulario::Transporte(recusa(502)),
        ));
        assert_eq!(post.status_http().map(|h| h.status), Some(502));
        assert_eq!(post.subcodigo(), "status_http_de_erro_do_servidor");
        let local = ErroPaginaWeb::Transporte(FalhaTransporte::CookieInvalido);
        assert_eq!(local.status_http(), None);
    }
}
