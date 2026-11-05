//! Passos compartilhados pelos fluxos: troca de visão e página da D. Cada
//! passo é no máximo um POST.
use super::envio::{EnvioDoControle, Redirecionado, Sessao, TelaAtual};
use super::plano::{PedidoDePagina, valores_ocultos};
use super::{
    CAMPOS_USUARIO, ErroVisaoDoControle as E, PAGINAR_VISAO_DETALHADA, TROCAR_VISAO_DO_CONTROLE,
    Visao,
};
use crate::caixa_detalhada::{LeituraDetalhada, ler_paginas};
use crate::controle_contexto::{ContextoPagina, contexto_da_pagina};
use crate::formulario_web::localizar_formulario;
use crate::pagina_do_controle::{ErroPaginaDoControle, campos_do_usuario};

pub(super) fn contexto(html: &str) -> Result<ContextoPagina, E> {
    contexto_da_pagina(html).map_err(E::Caixa)
}

/// Rótulo publicado em `lnkInfraUnidade`, conferido com `--unidade`.
pub(super) fn unidade_da_tela(
    contexto: &ContextoPagina,
    solicitada: Option<&str>,
) -> Result<String, E> {
    let unidade = contexto
        .unidade
        .clone()
        .ok_or(E::Html(crate::erro_html::ErroHtml::ausente(
            "identificacao_da_unidade_ausente_no_controle",
            "a tela não tem o elemento observado",
        )))?;
    if solicitada.is_some_and(|s| s.split_whitespace().collect::<Vec<_>>().join(" ") != unidade) {
        return Err(E::UnidadeDiferenteDaSolicitada);
    }
    Ok(unidade)
}

/// Controles de usuário que o `submit()` repete, com os valores servidos.
fn campos_do_formulario(html: &str) -> Result<Vec<(String, String)>, E> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| {
        E::Caixa(crate::erro_web::ErroCaixa::Html(
            crate::erro_html::ErroHtml::html(
                "html_da_caixa_invalido",
                "o HTML não pôde ser interpretado",
            ),
        ))
    })?;
    let form = localizar_formulario(&dom, TROCAR_VISAO_DO_CONTROLE.formulario)?;
    campos_do_usuario(form, dom.parser(), CAMPOS_USUARIO).map_err(E::Formulario)
}

/// `trocarVisualizacao(destino)`: um POST; a resposta é a visão `destino`
/// da mesma unidade.
pub(super) fn trocar<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
    destino: Visao,
) -> Result<TelaAtual, E> {
    let usuario = campos_do_formulario(&tela.html)?;
    let mut campos: Vec<(&str, &str)> = usuario
        .iter()
        .map(|(n, v)| (n.as_str(), v.as_str()))
        .collect();
    campos.push(("hdnTipoVisualizacao", destino.valor()));
    let nova = sessao.enviar(
        tela,
        &TROCAR_VISAO_DO_CONTROLE,
        &campos,
        Redirecionado::AoTrocarVisao,
    )?;
    let obtido = contexto(&nova.html)?;
    if obtido.unidade.as_deref() != Some(sessao.unidade) {
        return Err(E::UnidadeAlteradaAoTrocarVisao);
    }
    let certa = match destino {
        Visao::Resumida => obtido.eh_resumida(),
        Visao::Detalhada => obtido.eh_detalhada(),
    };
    if !certa {
        return Err(E::VisaoNaoTrocadaAposEnvio { esperada: destino });
    }
    Ok(nova)
}

pub(super) fn ler_detalhada(html: &str, unidade: &str) -> Result<LeituraDetalhada, E> {
    ler_paginas(&[html.to_owned()], Some(unidade)).map_err(E::Detalhada)
}

/// `infraAcaoPaginar('=', indice, 'Detalhado')`: um POST a partir da tela D;
/// a resposta mantém unidade, visão, critério e total, com a faixa pedida.
pub(super) fn paginar_detalhada<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
    pedido: PedidoDePagina,
    criterio: Option<&str>,
) -> Result<(TelaAtual, LeituraDetalhada), E> {
    let usuario = campos_do_formulario(&tela.html)?;
    let indice = pedido.indice.to_string();
    let mut campos: Vec<(&str, &str)> = usuario
        .iter()
        .map(|(n, v)| (n.as_str(), v.as_str()))
        .collect();
    campos.push(("hdnDetalhadoPaginaAtual", indice.as_str()));
    let nova = sessao.enviar(
        tela,
        &PAGINAR_VISAO_DETALHADA,
        &campos,
        Redirecionado::AoPaginar,
    )?;
    let obtido = contexto(&nova.html)?;
    if obtido.unidade.as_deref() != Some(sessao.unidade) {
        return Err(E::Pagina(ErroPaginaDoControle::UnidadeAlteradaAposPaginar));
    }
    if !obtido.eh_detalhada() {
        return Err(E::Pagina(
            ErroPaginaDoControle::VisualizacaoAlteradaAposPaginar,
        ));
    }
    if valores_ocultos(&nova.html, "hdnDetalhadoHashCriterios")?
        .first()
        .map(String::as_str)
        != criterio
    {
        return Err(E::CriteriosDaVisaoDetalhadaAlteradosAposPaginar);
    }
    let leitura = ler_detalhada(&nova.html, sessao.unidade)?;
    if leitura.total_declarado != pedido.total {
        return Err(E::TotalDeclaradoAlteradoAposPaginar);
    }
    if leitura.faixa_observada != pedido.esperada {
        return Err(E::FaixaDaRespostaDiferenteDaPaginaSolicitada);
    }
    Ok((nova, leitura))
}
