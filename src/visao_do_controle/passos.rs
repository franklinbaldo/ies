//! Passos compartilhados pelos fluxos: troca de visão e página da D. Cada
//! passo é no máximo um POST.
use super::envio::{EnvioDoControle, Redirecionado, Sessao, TelaAtual};
use super::plano::{PedidoDePagina, valores_ocultos};
use super::{
    CAMPOS_USUARIO, ErroVisaoDoControle, PAGINAR_VISAO_DETALHADA, TROCAR_VISAO_DO_CONTROLE, Visao,
};
use crate::caixa_detalhada::{LeituraDetalhada, ler_paginas};
use crate::controle_contexto::{ContextoPagina, contexto_da_pagina};
use crate::formulario_web::localizar_formulario;
use crate::pagina_do_controle::{ErroPaginaDoControle, campos_do_usuario};

pub(super) fn contexto(html: &str) -> Result<ContextoPagina, ErroVisaoDoControle> {
    contexto_da_pagina(html).map_err(ErroVisaoDoControle::Caixa)
}

/// Rótulo publicado em `lnkInfraUnidade`, conferido com `--unidade`.
pub(super) fn unidade_da_tela(
    contexto: &ContextoPagina,
    solicitada: Option<&str>,
) -> Result<String, ErroVisaoDoControle> {
    let unidade = contexto
        .unidade
        .clone()
        .ok_or(ErroVisaoDoControle::IdentificacaoDaUnidadeAusenteNoControle)?;
    if solicitada.is_some_and(|s| s.split_whitespace().collect::<Vec<_>>().join(" ") != unidade) {
        return Err(ErroVisaoDoControle::UnidadeDiferenteDaSolicitada);
    }
    Ok(unidade)
}

/// Controles de usuário que o `submit()` repete, com os valores servidos.
fn campos_do_formulario(html: &str) -> Result<Vec<(String, String)>, ErroVisaoDoControle> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroVisaoDoControle::HtmlDoFormularioDoControleRecusadoPeloParser)?;
    let form = localizar_formulario(&dom, TROCAR_VISAO_DO_CONTROLE.formulario)?;
    campos_do_usuario(form, dom.parser(), CAMPOS_USUARIO).map_err(ErroVisaoDoControle::Formulario)
}

/// `trocarVisualizacao(destino)`: um POST; a resposta é a visão `destino`
/// da mesma unidade.
pub(super) fn trocar<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
    destino: Visao,
) -> Result<TelaAtual, ErroVisaoDoControle> {
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
        return Err(ErroVisaoDoControle::UnidadeAlteradaAoTrocarVisao);
    }
    let certa = match destino {
        Visao::Resumida => obtido.eh_resumida(),
        Visao::Detalhada => obtido.eh_detalhada(),
    };
    if !certa {
        return Err(ErroVisaoDoControle::VisaoNaoTrocadaAposEnvio { esperada: destino });
    }
    Ok(nova)
}

pub(super) fn ler_detalhada(
    html: &str,
    unidade: &str,
) -> Result<LeituraDetalhada, ErroVisaoDoControle> {
    ler_paginas(&[html.to_owned()], Some(unidade)).map_err(ErroVisaoDoControle::Detalhada)
}

/// `infraAcaoPaginar('=', indice, 'Detalhado')`: um POST a partir da tela D;
/// a resposta mantém unidade, visão, critério e total, com a faixa pedida.
pub(super) fn paginar_detalhada<V: EnvioDoControle>(
    sessao: &mut Sessao<'_, V>,
    tela: &TelaAtual,
    pedido: PedidoDePagina,
    criterio: Option<&str>,
) -> Result<(TelaAtual, LeituraDetalhada), ErroVisaoDoControle> {
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
        return Err(ErroVisaoDoControle::Pagina(
            ErroPaginaDoControle::UnidadeAlteradaAposPaginar,
        ));
    }
    if !obtido.eh_detalhada() {
        return Err(ErroVisaoDoControle::Pagina(
            ErroPaginaDoControle::VisualizacaoAlteradaAposPaginar,
        ));
    }
    if valores_ocultos(&nova.html, "hdnDetalhadoHashCriterios")?
        .first()
        .map(String::as_str)
        != criterio
    {
        return Err(ErroVisaoDoControle::CriteriosDaVisaoDetalhadaAlteradosAposPaginar);
    }
    let leitura = ler_detalhada(&nova.html, sessao.unidade)?;
    if leitura.total_declarado != pedido.total {
        return Err(ErroVisaoDoControle::TotalDeclaradoAlteradoAposPaginar);
    }
    if leitura.faixa_observada != pedido.esperada {
        return Err(ErroVisaoDoControle::FaixaDaRespostaDiferenteDaPaginaSolicitada);
    }
    Ok((nova, leitura))
}
