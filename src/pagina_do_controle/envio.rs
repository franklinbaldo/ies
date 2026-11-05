//! Derivação, envio único e conferência da página pedida por `--pagina N`.
use super::plano::{campo_da_pagina, planejar};
use super::{ErroPaginaDoControle as E, OBTER_PAGINA_DO_CONTROLE, PlanoDaPagina, regras};
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

fn excessiva() -> E {
    E::Submissao(ErroSubmissaoFormulario::Transporte(
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
) -> Result<SubmissaoDerivada, E> {
    if tela.html.len() > crate::MAX_HTML_BYTES {
        return Err(excessiva());
    }
    let usuario = {
        let dom = tl::parse(tela.html, tl::ParserOptions::default()).map_err(|_| excessiva())?;
        let form = localizar_formulario(&dom, OBTER_PAGINA_DO_CONTROLE.formulario)
            .map_err(E::Submissao)?;
        regras::campos_do_usuario(form, dom.parser(), regras::CAMPOS_USUARIO)
            .map_err(E::Formulario)?
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
        unidade,
        &campos,
    )
    .map_err(E::Submissao)
}

/// A página obtida precisa manter o contexto da origem e trazer a faixa pedida.
fn conferir_resposta(
    html: &str,
    origem: &ContextoPagina,
    categoria: Categoria,
    esperada: FaixaNaPagina,
) -> Result<(), E> {
    let contexto = contexto_da_pagina(html).map_err(E::Caixa)?;
    if let Some(diferenca) = origem.diferenca(&contexto) {
        return Err(match diferenca {
            DiferencaDeContexto::Unidade => E::UnidadeAlteradaAposPaginar,
            DiferencaDeContexto::Visualizacao => E::VisualizacaoAlteradaAposPaginar,
            DiferencaDeContexto::CriteriosRecebidos => E::CriteriosRecebidosAlteradosAposPaginar,
            DiferencaDeContexto::CriteriosGerados => E::CriteriosGeradosAlteradosAposPaginar,
        });
    }
    match faixa_na_pagina(html, categoria).map_err(E::Caixa)? {
        Some(faixa) if faixa.total != esperada.total => {
            Err(E::TotalDeclaradoAlteradoAposPaginar { categoria })
        }
        Some(faixa) if faixa == esperada => Ok(()),
        _ => Err(E::FaixaDaRespostaDiferenteDaPaginaSolicitada { categoria }),
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
) -> Result<Option<String>, E>
where
    F: FnOnce(SubmissaoDerivada, &str) -> Result<RespostaDoPost<R>, FalhaTransporte>,
{
    let PlanoDaPagina::Pedir { indice, esperada } = planejar(tela.html, categoria, pagina)? else {
        return Ok(None);
    };
    let origem = contexto_da_pagina(tela.html).map_err(E::Caixa)?;
    let submissao = derivar_pagina(instalacao, tela, unidade, categoria, indice)?;
    let html = match enviar_submissao_via(submissao, cookie, enviar).map_err(E::Submissao)? {
        RespostaDoEnvio::Html(html) => html,
        RespostaDoEnvio::Redirecionamento(_) => return Err(E::RedirecionamentoAposPaginar),
    };
    conferir_resposta(&html, &origem, categoria, esperada)?;
    Ok(Some(html))
}
