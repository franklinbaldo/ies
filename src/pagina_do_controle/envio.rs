//! Derivação, envio único e conferência da página pedida por `--pagina N`.
use super::plano::{campo_da_pagina, planejar};
use super::{ErroPaginaDoControle, OBTER_PAGINA_DO_CONTROLE, PlanoDaPagina, regras};
use crate::controle_contexto::{ContextoPagina, DiferencaDeContexto, contexto_da_pagina};
use crate::dominio::Categoria;
use crate::formulario_web::{
    ErroSubmissaoFormulario, RespostaDoEnvio, RespostaDoPost, SubmissaoDerivada, TelaServida,
    derivar_submissao, enviar_submissao_via, localizar_formulario,
};
use crate::instalacao::Instalacao;
use crate::paginas::{FaixaNaPagina, faixa_na_pagina};
use crate::transporte::FalhaTransporte;
use crate::transporte::sessao::CookieDaSessao;
use std::io::Read;

fn excessiva() -> ErroPaginaDoControle {
    ErroPaginaDoControle::Submissao(ErroSubmissaoFormulario::Transporte(
        FalhaTransporte::RespostaExcessiva,
    ))
}

/// Deriva o POST do paginador, sem rede: como `infraAcaoPaginar('=', indice,
/// T)`, só `hdn{T}PaginaAtual` muda; o resto sai como servido.
pub(crate) fn derivar_pagina(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade: &str,
    categoria: Categoria,
    indice: usize,
) -> Result<SubmissaoDerivada, ErroPaginaDoControle> {
    if tela.html.len() > crate::MAX_HTML_BYTES {
        return Err(excessiva());
    }
    let usuario = {
        let dom = tl::parse(tela.html, tl::ParserOptions::default())
            .map_err(ErroPaginaDoControle::HtmlDaTelaDeEnvioRecusadoPeloParser)?;
        let form = localizar_formulario(&dom, OBTER_PAGINA_DO_CONTROLE.formulario)?;
        regras::campos_do_usuario(form, dom.parser(), regras::CAMPOS_USUARIO)?
    };
    let indice = indice.to_string();
    let mut campos: Vec<(&str, &str)> = usuario
        .iter()
        .map(|(nome, valor)| (nome.as_str(), valor.as_str()))
        .collect();
    campos.push((campo_da_pagina(categoria), indice.as_str()));
    derivar_submissao(
        &OBTER_PAGINA_DO_CONTROLE,
        instalacao,
        tela,
        Some(unidade),
        &campos,
    )
    .map_err(ErroPaginaDoControle::Submissao)
}

/// A página obtida precisa manter o contexto da origem e trazer a faixa pedida.
fn conferir_resposta(
    html: &str,
    origem: &ContextoPagina,
    categoria: Categoria,
    esperada: FaixaNaPagina,
) -> Result<(), ErroPaginaDoControle> {
    let contexto = contexto_da_pagina(html)?;
    if let Some(diferenca) = origem.diferenca(&contexto) {
        return Err(match diferenca {
            DiferencaDeContexto::Unidade => ErroPaginaDoControle::UnidadeAlteradaAposPaginar,
            DiferencaDeContexto::Visualizacao => {
                ErroPaginaDoControle::VisualizacaoAlteradaAposPaginar
            }
            DiferencaDeContexto::CriteriosRecebidos => {
                ErroPaginaDoControle::CriteriosRecebidosAlteradosAposPaginar
            }
            DiferencaDeContexto::CriteriosGerados => {
                ErroPaginaDoControle::CriteriosGeradosAlteradosAposPaginar
            }
        });
    }
    match faixa_na_pagina(html, categoria)? {
        Some(faixa) if faixa.total != esperada.total => {
            Err(ErroPaginaDoControle::TotalDeclaradoAlteradoAposPaginar { categoria })
        }
        Some(faixa) if faixa == esperada => Ok(()),
        _ => Err(ErroPaginaDoControle::FaixaDaRespostaDiferenteDaPaginaSolicitada { categoria }),
    }
}

/// Obtém a página: `Ok(None)` se a tela já a mostra (zero POST), senão
/// envia uma vez (`FnOnce`, sem retry) e confere a resposta.
pub(crate) fn obter_pagina_via<R: Read, F>(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade: &str,
    categoria: Categoria,
    pagina: usize,
    cookie: &CookieDaSessao,
    enviar: F,
) -> Result<Option<String>, ErroPaginaDoControle>
where
    F: FnOnce(SubmissaoDerivada, &str) -> Result<RespostaDoPost<R>, FalhaTransporte>,
{
    let PlanoDaPagina::Pedir { indice, esperada } = planejar(tela.html, categoria, pagina)? else {
        return Ok(None);
    };
    let origem = contexto_da_pagina(tela.html)?;
    let submissao = derivar_pagina(instalacao, tela, unidade, categoria, indice)?;
    let html = match enviar_submissao_via(submissao, cookie, enviar)? {
        RespostaDoEnvio::Html(html) => html,
        RespostaDoEnvio::Redirecionamento(_) => {
            return Err(ErroPaginaDoControle::RedirecionamentoAposPaginar);
        }
    };
    conferir_resposta(&html, &origem, categoria, esperada)?;
    Ok(Some(html))
}
