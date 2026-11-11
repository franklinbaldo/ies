//! `ies processo buscar TERMO [--pagina N]` (contracts/cli/processo-buscar.md)
//! e `ies documento buscar TERMO [--pagina N]` (contracts/cli/documento-buscar.md):
//! GET do Controle → link do menu `protocolo_pesquisar` → GET da tela →
//! POST da pesquisa → (N > 1) POST de `navegar(inicio)`. Cada chamada do
//! envio é uma requisição real única, sem retry nem redirect seguido; a
//! sessão fica na página/filtro em que a pesquisa a deixou.
use super::alvo::AlvoDaPesquisa;
#[cfg(test)]
use super::alvo::EmProcessos;
use super::erro_busca::ErroBuscaDeProcesso;
use super::filtros::FiltrosDaPesquisa;
use super::formulario_pesquisa::{
    ErroPesquisaProtocolo, derivar_pagina_em_filtrada, derivar_pesquisa_em_filtrada, falha_do_envio,
};
use super::link_da_pesquisa::link_da_pesquisa;
use super::paginacao::{
    ErroPaginacaoDaPesquisa, conferir_contagem, conferir_protocolo_servido, planejar_pagina_por,
};
use super::redirecionamento::UrlDeTrabalho;
use super::resposta::RespostaDaPesquisa;
use crate::colheita_do_controle::Colheita;
use crate::controle_reconhecido::unidade_do_controle;
use crate::formulario_web::{
    RespostaDoEnvio, SubmissaoDerivada, TelaServida, enviar_submissao_via,
};
use crate::pagina_do_controle::codificacao_da_tela;
use crate::transporte::contagem::ContagemDeRequisicoes;
use crate::transporte::decodificar_resposta;
use crate::transporte::envio_post::{TelaDoControle, ler_tela_via};
use crate::transporte::sessao::CookieDaSessao;
use crate::visao_do_controle::{EnvioDoControle, TelaAtual};
use std::cell::RefCell;

/// Arquivos privados da sessão já lidos: URL assinada do Controle e origem.
pub(crate) struct SessaoDaBusca<'a> {
    pub(crate) url_do_controle: &'a str,
    pub(crate) origem: &'a str,
    pub(crate) cookie: &'a CookieDaSessao,
}

/// O que o usuário pediu: termo (já validado pela borda), página opcional
/// e `--unidade` conferida.
pub(crate) struct PedidoDeBusca<'a> {
    pub(crate) termo: &'a str,
    pub(crate) pagina: Option<u32>,
    pub(crate) unidade: Option<&'a str>,
}

/// Resposta observada, unidade do Controle e requisições feitas.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BuscaRealizada<R = RespostaDaPesquisa> {
    pub(crate) unidade: String,
    pub(crate) resposta: R,
    pub(crate) requisicoes: u8,
    /// URL de `procedimento_trabalhar` do 302, conferida e nunca seguida
    /// aqui; `processo buscar` a descarta, `processo NUP` a abre.
    pub(crate) trabalho: Option<UrlDeTrabalho>,
}

/// O GET do Controle já feito e a unidade que ele publica (conferida).
#[derive(Debug)]
pub(crate) struct ControleLido {
    pub(crate) tela: TelaDoControle,
    pub(crate) unidade: String,
}

/// Requisição 1: GET do Controle (origem conferida antes de ler o cookie),
/// unidade reconhecida e comparada com `--unidade`.
pub(crate) fn ler_controle<V: EnvioDoControle>(
    sessao: &SessaoDaBusca<'_>,
    unidade_pedida: Option<&str>,
    envio: &mut V,
) -> Result<ControleLido, ErroBuscaDeProcesso> {
    ler_controle_interno(sessao, unidade_pedida, None, envio)
}

fn ler_controle_com_contagem<V: EnvioDoControle>(
    sessao: &SessaoDaBusca<'_>,
    unidade_pedida: Option<&str>,
    requisicoes: &mut ContagemDeRequisicoes,
    envio: &mut V,
) -> Result<ControleLido, ErroBuscaDeProcesso> {
    ler_controle_interno(sessao, unidade_pedida, Some(requisicoes), envio)
}

fn ler_controle_interno<V: EnvioDoControle>(
    sessao: &SessaoDaBusca<'_>,
    unidade_pedida: Option<&str>,
    mut requisicoes: Option<&mut ContagemDeRequisicoes>,
    envio: &mut V,
) -> Result<ControleLido, ErroBuscaDeProcesso> {
    let tela = ler_tela_via(
        sessao.url_do_controle,
        sessao.cookie,
        sessao.origem,
        |url, cookie| {
            if let Some(contagem) = requisicoes.take() {
                contagem.registrar();
            }
            envio.get(url, cookie)
        },
    )?;
    let unidade = unidade_do_controle(&tela.html)?;
    if unidade_pedida.is_some_and(|s| s.split_whitespace().collect::<Vec<_>>().join(" ") != unidade)
    {
        return Err(ErroBuscaDeProcesso::UnidadeDoControleDiferenteDaSolicitada);
    }
    Ok(ControleLido { tela, unidade })
}

/// Orçamento: página 1 = GET + GET + POST; página N = mais um POST.
#[cfg(test)]
pub(crate) fn buscar_processo<V: EnvioDoControle>(
    sessao: &SessaoDaBusca<'_>,
    pedido: &PedidoDeBusca<'_>,
    envio: &mut V,
) -> Result<BuscaRealizada, ErroBuscaDeProcesso> {
    buscar_colhendo::<EmProcessos, V>(sessao, pedido, &Colheita::nenhuma(), envio)
}

/// [`buscar_processo`] em `A` (processos ou documentos), com a colheita do
/// Controle lido (nenhuma requisição a mais).
#[cfg(test)]
pub(crate) fn buscar_colhendo<A: AlvoDaPesquisa, V: EnvioDoControle>(
    sessao: &SessaoDaBusca<'_>,
    pedido: &PedidoDeBusca<'_>,
    colheita: &Colheita<'_>,
    envio: &mut V,
) -> Result<BuscaRealizada<A::Resposta>, ErroBuscaDeProcesso> {
    buscar_colhendo_filtrado::<A, V>(
        sessao,
        pedido,
        FiltrosDaPesquisa::default(),
        colheita,
        envio,
    )
}

pub(crate) fn buscar_colhendo_filtrado<A: AlvoDaPesquisa, V: EnvioDoControle>(
    sessao: &SessaoDaBusca<'_>,
    pedido: &PedidoDeBusca<'_>,
    filtros: FiltrosDaPesquisa<'_>,
    colheita: &Colheita<'_>,
    envio: &mut V,
) -> Result<BuscaRealizada<A::Resposta>, ErroBuscaDeProcesso> {
    let mut requisicoes = ContagemDeRequisicoes::default();
    buscar_colhendo_contado_filtrado::<A, V>(
        sessao,
        pedido,
        filtros,
        colheita,
        &mut requisicoes,
        envio,
    )
}

/// Versão que deixa a contagem disponível também quando uma tentativa
/// falha, para que o chamador e os testes observem o custo real do fluxo.
#[cfg(test)]
pub(crate) fn buscar_colhendo_contado<A: AlvoDaPesquisa, V: EnvioDoControle>(
    sessao: &SessaoDaBusca<'_>,
    pedido: &PedidoDeBusca<'_>,
    colheita: &Colheita<'_>,
    requisicoes: &mut ContagemDeRequisicoes,
    envio: &mut V,
) -> Result<BuscaRealizada<A::Resposta>, ErroBuscaDeProcesso> {
    buscar_colhendo_contado_filtrado::<A, V>(
        sessao,
        pedido,
        FiltrosDaPesquisa::default(),
        colheita,
        requisicoes,
        envio,
    )
}

pub(crate) fn buscar_colhendo_contado_filtrado<A: AlvoDaPesquisa, V: EnvioDoControle>(
    sessao: &SessaoDaBusca<'_>,
    pedido: &PedidoDeBusca<'_>,
    filtros: FiltrosDaPesquisa<'_>,
    colheita: &Colheita<'_>,
    requisicoes: &mut ContagemDeRequisicoes,
    envio: &mut V,
) -> Result<BuscaRealizada<A::Resposta>, ErroBuscaDeProcesso> {
    let controle = ler_controle_com_contagem(sessao, pedido.unidade, requisicoes, envio)?;
    colheita.colher(&controle.tela.html, Some(&controle.tela.mime));
    pesquisar_a_partir_do_controle_filtrado::<A, V>(
        &controle,
        sessao.cookie,
        pedido,
        filtros,
        requisicoes,
        envio,
    )
}

/// Requisições 2 em diante, a partir do Controle já lido (que conta como 1).
pub(crate) fn pesquisar_a_partir_do_controle_filtrado<A: AlvoDaPesquisa, V: EnvioDoControle>(
    controle: &ControleLido,
    cookie: &CookieDaSessao,
    pedido: &PedidoDeBusca<'_>,
    filtros: FiltrosDaPesquisa<'_>,
    requisicoes: &mut ContagemDeRequisicoes,
    envio: &mut V,
) -> Result<BuscaRealizada<A::Resposta>, ErroBuscaDeProcesso> {
    let unidade = &controle.unidade;
    let controle = &controle.tela;
    let instalacao = &controle.instalacao;
    let url_da_tela = link_da_pesquisa(instalacao, &controle.url, &controle.html)?;
    // 2. GET da tela de pesquisa pelo link publicado.
    let tela = obter_tela(cookie, &url_da_tela, requisicoes, envio)?;
    // 3. POST da pesquisa.
    let termo = pedido.termo;
    let submissao = derivar_pesquisa_em_filtrada(
        A::EM,
        instalacao,
        &servida(&tela)?,
        (unidade, termo),
        filtros,
    )?;
    let (enviada, resultado) = enviar(cookie, envio, requisicoes, submissao)?;
    let interpretado = A::interpretar(enviada, instalacao)?;
    let (resposta, html) = (interpretado.resposta, interpretado.html);
    let Some(solicitada) = pedido.pagina else {
        return Ok(BuscaRealizada {
            unidade: unidade.clone(),
            resposta,
            requisicoes: requisicoes.total_u8(),
            trabalho: interpretado.trabalho,
        });
    };
    let contagem = A::contagem(&resposta);
    let Some(pagina) = planejar_pagina_por(contagem, html.as_deref(), solicitada)? else {
        return Ok(BuscaRealizada {
            unidade: unidade.clone(),
            resposta,
            requisicoes: requisicoes.total_u8(),
            trabalho: interpretado.trabalho,
        });
    };
    // 4. POST de `navegar(inicio)` derivado da tela de resultado.
    let html = html.ok_or(ErroBuscaDeProcesso::Paginacao(
        ErroPaginacaoDaPesquisa::HtmlDaRespostaParaPaginarAusente,
    ))?;
    conferir_protocolo_servido(&html, termo)?;
    let tela_do_resultado = TelaAtual {
        url: resultado.url,
        html,
        mime: resultado.mime,
    };
    let submissao = derivar_pagina_em_filtrada(
        A::EM,
        instalacao,
        &servida(&tela_do_resultado)?,
        (unidade, termo),
        pagina.inicio,
        filtros,
    )?;
    let resposta = match enviar(cookie, envio, requisicoes, submissao)?.0 {
        RespostaDoEnvio::Redirecionamento(_) => {
            return Err(ErroBuscaDeProcesso::Paginacao(
                ErroPaginacaoDaPesquisa::RedirecionamentoAoPaginarAPesquisa,
            ));
        }
        RespostaDoEnvio::Html(html) => A::ler_pagina(&html, (pagina.esperada, pagina.numero))
            .map_err(|erro| ErroBuscaDeProcesso::Pesquisa(ErroPesquisaProtocolo::Resposta(erro)))?,
    };
    conferir_contagem(&pagina, A::contagem(&resposta))?;
    Ok(BuscaRealizada {
        unidade: unidade.clone(),
        resposta,
        requisicoes: requisicoes.total_u8(),
        trabalho: None,
    })
}

/// URL e `Content-Type` da resposta de um POST, para derivar o seguinte.
struct Resultado {
    url: String,
    mime: String,
}

fn servida(tela: &TelaAtual) -> Result<TelaServida<'_>, ErroBuscaDeProcesso> {
    let codificacao =
        codificacao_da_tela(&tela.mime).ok_or(ErroBuscaDeProcesso::CharsetDaTelaNaoSuportado)?;
    Ok(TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao,
    })
}

/// Um GET de URL já autorizada (link publicado pelo Controle).
fn obter_tela<V: EnvioDoControle>(
    cookie: &CookieDaSessao,
    url: &str,
    requisicoes: &mut ContagemDeRequisicoes,
    envio: &mut V,
) -> Result<TelaAtual, ErroBuscaDeProcesso> {
    let cookie = cookie.valor()?;
    requisicoes.registrar();
    let resposta = envio.get(url, cookie)?;
    let mime = resposta.mime.clone();
    let html = decodificar_resposta(resposta)?;
    Ok(TelaAtual {
        url: url.to_owned(),
        html,
        mime,
    })
}

/// Um POST, sem retry; o 302 volta como dado, nunca seguido.
fn enviar<V: EnvioDoControle>(
    cookie: &CookieDaSessao,
    envio: &mut V,
    requisicoes: &mut ContagemDeRequisicoes,
    submissao: SubmissaoDerivada,
) -> Result<(RespostaDoEnvio, Resultado), ErroBuscaDeProcesso> {
    let url = submissao.url().to_owned();
    let mime = RefCell::new(String::new());
    let resposta = enviar_submissao_via(submissao, cookie, |s, c| {
        requisicoes.registrar();
        let resposta = envio.post(s, c)?;
        mime.replace(resposta.resposta.mime.clone());
        Ok(resposta)
    })
    .map_err(|erro| ErroBuscaDeProcesso::Pesquisa(falha_do_envio(erro)))?;
    Ok((
        resposta,
        Resultado {
            url,
            mime: mime.into_inner(),
        },
    ))
}

#[cfg(test)]
#[path = "tests_fluxo.rs"]
pub(crate) mod tests;
