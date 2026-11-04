//! `ies processo buscar TERMO [--pagina N]` (contracts/cli/processo-buscar.md):
//! GET do Controle → link do menu `protocolo_pesquisar` → GET da tela →
//! POST da pesquisa → (N > 1) POST de `navegar(inicio)`. Cada chamada do
//! envio é uma requisição real única, sem retry nem redirect seguido; a
//! sessão fica na página/filtro em que a pesquisa a deixou.
use super::erro_busca::ErroBuscaDeProcesso;
use super::formulario_pesquisa::{
    ErroPesquisaProtocolo, derivar_pagina_da_pesquisa, derivar_pesquisa_de_protocolo,
    falha_do_envio, interpretar_envio,
};
use super::link_da_pesquisa::link_da_pesquisa;
use super::paginacao::{
    ErroPaginacaoDaPesquisa, conferir_pagina, conferir_protocolo_servido, planejar_pagina,
};
use super::redirecionamento::UrlDeTrabalho;
use super::resposta::{RespostaDaPesquisa, ler_resposta_html};
use crate::colheita_do_controle::Colheita;
use crate::controle_reconhecido::unidade_do_controle;
use crate::formulario_web::{
    RespostaDoEnvio, SubmissaoDerivada, TelaServida, enviar_submissao_via,
};
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::transporte::envio_post::{TelaDoControle, ler_tela_via};
use crate::transporte::decodificar_resposta;
use crate::visao_do_controle::{EnvioDoControle, TelaAtual};
use std::cell::RefCell;
use std::path::Path;

/// Arquivos privados da sessão já lidos: URL assinada do Controle e origem.
pub(crate) struct SessaoDaBusca<'a> {
    pub(crate) url_do_controle: &'a str,
    pub(crate) origem: &'a str,
    pub(crate) cookie_file: &'a Path,
}

/// O que o usuário pediu: termo, página opcional e `--unidade` conferida.
pub(crate) struct PedidoDeBusca<'a> {
    pub(crate) termo: &'a NumeroProcesso,
    pub(crate) pagina: Option<u32>,
    pub(crate) unidade: Option<&'a str>,
}

/// Resposta observada, unidade do Controle e requisições feitas.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BuscaRealizada {
    pub(crate) unidade: String,
    pub(crate) resposta: RespostaDaPesquisa,
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
    use ErroBuscaDeProcesso as E;
    let tela = ler_tela_via(
        sessao.url_do_controle,
        sessao.cookie_file,
        sessao.origem,
        |url, cookie| envio.get(url, cookie),
    )
    .map_err(E::Transporte)?;
    let unidade = unidade_do_controle(&tela.html).map_err(E::Controle)?;
    if unidade_pedida.is_some_and(|s| s.split_whitespace().collect::<Vec<_>>().join(" ") != unidade)
    {
        return Err(E::UnidadeDoControleDiferenteDaSolicitada);
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
    buscar_processo_colhendo(sessao, pedido, &Colheita::nenhuma(), envio)
}

/// [`buscar_processo`] com a colheita do Controle lido (nenhuma
/// requisição a mais).
pub(crate) fn buscar_processo_colhendo<V: EnvioDoControle>(
    sessao: &SessaoDaBusca<'_>,
    pedido: &PedidoDeBusca<'_>,
    colheita: &Colheita<'_>,
    envio: &mut V,
) -> Result<BuscaRealizada, ErroBuscaDeProcesso> {
    let controle = ler_controle(sessao, pedido.unidade, envio)?;
    colheita.colher(&controle.tela.html, Some(&controle.tela.mime));
    pesquisar_a_partir_do_controle(&controle, sessao.cookie_file, pedido, envio)
}

/// Requisições 2 em diante, a partir do Controle já lido (que conta como 1).
pub(crate) fn pesquisar_a_partir_do_controle<V: EnvioDoControle>(
    controle: &ControleLido,
    cookie_file: &Path,
    pedido: &PedidoDeBusca<'_>,
    envio: &mut V,
) -> Result<BuscaRealizada, ErroBuscaDeProcesso> {
    use ErroBuscaDeProcesso as E;
    let unidade = &controle.unidade;
    let controle = &controle.tela;
    let instalacao = &controle.instalacao;
    let url_da_tela =
        link_da_pesquisa(instalacao, &controle.url, &controle.html).map_err(E::Link)?;
    // 2. GET da tela de pesquisa pelo link publicado.
    let tela = obter_tela(cookie_file, &url_da_tela, envio)?;
    // 3. POST da pesquisa.
    let termo = pedido.termo.as_str();
    let submissao = derivar_pesquisa_de_protocolo(instalacao, &servida(&tela)?, unidade, termo)
        .map_err(E::Pesquisa)?;
    let (enviada, resultado) = enviar(cookie_file, envio, submissao)?;
    let interpretado = interpretar_envio(enviada, instalacao).map_err(E::Pesquisa)?;
    let (resposta, html) = (interpretado.resposta, interpretado.html);
    let pronta = |resposta, requisicoes, trabalho| BuscaRealizada {
        unidade: unidade.clone(),
        resposta,
        requisicoes,
        trabalho,
    };
    let Some(solicitada) = pedido.pagina else {
        return Ok(pronta(resposta, 3, interpretado.trabalho));
    };
    let Some(pagina) =
        planejar_pagina(&resposta, html.as_deref(), solicitada).map_err(E::Paginacao)?
    else {
        return Ok(pronta(resposta, 3, interpretado.trabalho));
    };
    // 4. POST de `navegar(inicio)` derivado da tela de resultado.
    let html = html.ok_or(E::Paginacao(
        ErroPaginacaoDaPesquisa::RegraDeNavegacaoDaPesquisaNaoObservada,
    ))?;
    conferir_protocolo_servido(&html, termo).map_err(E::Paginacao)?;
    let tela_do_resultado = TelaAtual {
        url: resultado.url,
        html,
        mime: resultado.mime,
    };
    let submissao = derivar_pagina_da_pesquisa(
        instalacao,
        &servida(&tela_do_resultado)?,
        unidade,
        termo,
        pagina.inicio,
    )
    .map_err(E::Pesquisa)?;
    let resposta = match enviar(cookie_file, envio, submissao)?.0 {
        RespostaDoEnvio::Redirecionamento(_) => {
            return Err(E::Paginacao(
                ErroPaginacaoDaPesquisa::RedirecionamentoAoPaginarAPesquisa,
            ));
        }
        RespostaDoEnvio::Html(html) => ler_resposta_html(&html)
            .map_err(|erro| E::Pesquisa(ErroPesquisaProtocolo::Resposta(erro)))?,
    };
    let lista = conferir_pagina(&pagina, resposta).map_err(E::Paginacao)?;
    Ok(pronta(RespostaDaPesquisa::Lista(lista), 4, None))
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
    cookie_file: &Path,
    url: &str,
    envio: &mut V,
) -> Result<TelaAtual, ErroBuscaDeProcesso> {
    use ErroBuscaDeProcesso as E;
    let cookie = crate::transporte::sessao::cookie_privado_validado(cookie_file)
        .map_err(E::Transporte)?;
    let resposta = envio.get(url, &cookie).map_err(E::Transporte)?;
    let mime = resposta.mime.clone();
    let html = decodificar_resposta(resposta).map_err(E::Transporte)?;
    Ok(TelaAtual {
        url: url.to_owned(),
        html,
        mime,
    })
}

/// Um POST, sem retry; o 302 volta como dado, nunca seguido.
fn enviar<V: EnvioDoControle>(
    cookie_file: &Path,
    envio: &mut V,
    submissao: SubmissaoDerivada,
) -> Result<(RespostaDoEnvio, Resultado), ErroBuscaDeProcesso> {
    let url = submissao.url().to_owned();
    let mime = RefCell::new(String::new());
    let resposta = enviar_submissao_via(submissao, cookie_file, |s, c| {
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
mod tests;
