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
