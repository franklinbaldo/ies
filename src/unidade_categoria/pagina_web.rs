//! `--categoria T --pagina N`: um GET e, se a tela não mostra a página N,
//! um POST do paginador de T. Sem retry, sem outras páginas, sem restaurar.
use crate::configuracao::ArquivosDeAcesso;
use crate::dominio::Categoria;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::TelaServida;
use crate::pagina_do_controle::{
    self, ErroPaginaDoControle, PlanoDaPagina, codificacao_da_tela, obter_pagina_via,
};
use crate::paginas;
use crate::transporte::FalhaTransporte;
use crate::transporte::envio_post::{enviar_post, ler_tela_do_controle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ErroPaginaWeb {
    Transporte(FalhaTransporte),
    Caixa(ErroCaixa),
    Pagina(ErroPaginaDoControle),
}

impl ErroPaginaWeb {
    pub(super) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Transporte(erro) => erro.subcodigo(),
            Self::Caixa(erro) => erro.subcodigo(),
            Self::Pagina(erro) => erro.subcodigo(),
        }
    }

    pub(super) fn mensagem(&self) -> &'static str {
        match self {
            Self::Transporte(erro) => erro.mensagem(),
            Self::Caixa(_) => "Controle de Processos não reconhecido",
            Self::Pagina(_) => "a página pedida do Controle não foi obtida",
        }
    }

    /// Status e mensagem do SEI, do GET ou do POST do paginador.
    pub(super) fn status_http(&self) -> Option<&crate::transporte::StatusHttpRecusado> {
        None
    }
}

/// HTML da página N da categoria, conferido contra a tela de origem.
pub(super) fn obter(
    arquivos: &ArquivosDeAcesso,
    unidade: Option<&str>,
    categoria: Categoria,
    numero: usize,
) -> Result<String, ErroPaginaWeb> {
    let tela = ler_tela_do_controle(&arquivos.url_controle, &arquivos.cookie, &arquivos.origem)
        .map_err(ErroPaginaWeb::Transporte)?;
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
