//! Os quatro passos da resolução (contracts/cli/processo-obter-arvore.md).
//! Um passo é tentado só se o anterior não se aplica ou falhou por palpite
//! do índice; o palpite errado é esquecido e o comando segue.
use super::abertura::{
    ArvoreAberta, CAMPO, Contexto, Lida, PESQUISA_RAPIDA, Rede, concluir, ler_arvore_aberta,
};
use super::controle::link_do_processo_no_controle;
use super::login::{DestinoDoRedirecionamento, destino};
use super::pastas::{self, Guardado};
use super::{
    EnvioDoProcesso, ErroProcessoObter as E, PedidoDoProcesso, ProcessoAberto, ResolvidoPor,
    SessaoDoProcesso,
};
use crate::arvore_processo::ErroArvore;
use crate::formulario_web::{
    ErroSubmissaoFormulario, TelaServida, derivar_submissao, derivar_submissao_guardada,
};
use crate::indice_local::{IndiceLocal, url_da_entrada_por_id};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso;
use crate::pesquisa_web::fluxo::{SessaoDaBusca, ler_controle};
use crate::pesquisa_web::redirecionamento::{
    ErroRedirecionamentoDaPesquisa, OrigemDoTrabalho, conferir_trabalho,
};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado, arquivo_privado, validar_cookie};

/// Resultado de um passo: aberto, ou "não se aplica / palpite esquecido".
type Passo = Result<Option<(ArvoreAberta, Option<String>, ResolvidoPor)>, E>;

/// `infra_unidade_atual` único e decimal na URL privada do Controle.
fn unidade_da_url(url: &str) -> Option<String> {
    let consulta = url.split_once('?')?.1;
    let mut valores = consulta
        .split('&')
        .filter_map(|par| par.strip_prefix("infra_unidade_atual="));
    let unidade = valores.next()?;
    let decimal = !unidade.is_empty() && unidade.bytes().all(|b| b.is_ascii_digit());
    (decimal && valores.next().is_none()).then(|| unidade.to_owned())
}

pub(crate) fn obter_processo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoProcesso<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoAberto, E> {
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(E::Transporte(FalhaTransporte::OrigemNaoAutorizada))?;
    let texto = arquivo_privado(sessao.cookie_file, 8_192).map_err(E::Transporte)?;
    let cookie = texto.trim().to_owned();
    validar_cookie(&cookie).map_err(E::Transporte)?;
    let contexto = Contexto {
        instalacao,
        nup: pedido.nup.as_str(),
        unidade_pedida: pedido.unidade,
        geracao: crate::sessao_login::geracao_da_sessao(&cookie),
        unidade_id: unidade_da_url(sessao.url_do_controle),
        indice,
        cookie_file: sessao.cookie_file,
    };
    let mut rede = Rede {
        envio,
        cookie,
        requisicoes: 0,
    };
    let aberto = match link_da_arvore(&contexto, &mut rede)? {
        Some(aberto) => aberto,
        None => match entrada_por_id(&contexto, &mut rede)? {
            Some(aberto) => aberto,
            None => match pesquisa_guardada(&contexto, sessao, &mut rede)? {
                Some(aberto) => aberto,
                None => pelo_controle(&contexto, sessao, &mut rede)?,
            },
        },
    };
    let (arvore_lida, unidade, resolvido_por) = aberto;
    let andamentos = super::andamentos::ler(
        &contexto,
        &mut rede,
        &arvore_lida,
        &super::andamentos::Leitura {
            pedido: pedido.andamentos,
            guardado_primeiro: resolvido_por == ResolvidoPor::IndiceLink,
            unidade: unidade.as_deref(),
        },
    )?;
    Ok(ProcessoAberto {
        arvore: arvore_lida.arvore,
        unidade,
        resolvido_por,
        andamentos,
        requisicoes: rede.requisicoes,
    })
}

/// Passo 1, link guardado nesta geração e unidade: o de "Abrir todas as
/// Pastas", se houver, e senão o da árvore; um GET. A árvore não publica a
/// unidade; com `--unidade` o passo é pulado.
fn link_da_arvore<V: EnvioDoProcesso>(contexto: &Contexto<'_>, rede: &mut Rede<'_, V>) -> Passo {
    const ACAO: &str = "procedimento_visualizar";
    if contexto.unidade_pedida.is_some() {
        return Ok(None);
    }
    let instalacao = &contexto.instalacao;
    let Some(id) = contexto.indice.id_por_nup(instalacao, contexto.nup) else {
        return Ok(None);
    };
    match pastas::pelo_link_guardado(contexto, rede, &id)? {
        Guardado::Aberta(aberta) => return Ok(Some((aberta, None, ResolvidoPor::IndiceLink))),
        Guardado::Divergiu => return Ok(None),
        Guardado::Seguir => {}
    }
    let Some(link) = contexto
        .chave(ACAO, &id)
        .and_then(|chave| contexto.indice.link(instalacao, chave))
    else {
        return Ok(None);
    };
    if let Lida::Html { html, .. } = rede.get(link.url())? {
        match ler_arvore_aberta(contexto, &html, &id) {
            Ok(aberta) => {
                concluir(contexto, &aberta.arvore, None, link.url());
                let aberta = pastas::completar(contexto, rede, &html, aberta)?;
                return Ok(Some((aberta, None, ResolvidoPor::IndiceLink)));
            }
            Err(erro) if erro.diverge_do_pedido() => {
                contexto.indice.esquecer_processo(instalacao, contexto.nup);
            }
            // Não é uma árvore: o link envelheceu.
            Err(ErroArvore::NoDoProcessoAusenteNaArvore) => {}
            Err(erro) => return Err(E::Arvore(erro)),
        }
    }
    contexto.esquecer_link(ACAO, &id);
    Ok(None)
}

/// Passo 2, entrada sem assinatura por id (a única URL que o IES monta): 302 para
/// o `procedimento_trabalhar` assinado do mesmo id, depois tela e árvore.
fn entrada_por_id<V: EnvioDoProcesso>(contexto: &Contexto<'_>, rede: &mut Rede<'_, V>) -> Passo {
    let instalacao = &contexto.instalacao;
    let Some(id) = contexto.indice.id_por_nup(instalacao, contexto.nup) else {
        return Ok(None);
    };
    if contexto.indice.entrada_por_id(instalacao) == Some(false) {
        return Ok(None);
    }
    let Some(url) = url_da_entrada_por_id(instalacao, &id) else {
        return Ok(None);
    };
    let localizacao = match rede.get(&url)? {
        Lida::Redirecionamento(localizacao) => localizacao,
        Lida::Login => return Err(E::SessaoEncerrada),
        Lida::Html { .. } => {
            return Err(E::RespostaDaEntradaPorIdNaoReconhecida(
                StatusHttpRecusado::sem_mensagem(200),
            ));
        }
        Lida::Falha(falha) => {
            let status = falha
                .status_http()
                .cloned()
                .ok_or(E::Transporte(falha.clone()))?;
            return Err(E::RespostaDaEntradaPorIdNaoReconhecida(status));
        }
    };
    match destino(localizacao.as_deref()) {
        DestinoDoRedirecionamento::RecusaSemAssinatura => {
            // A recusa derrubou a sessão: registrar e parar, sem relogin.
            contexto.indice.registrar_entrada_por_id(instalacao, false);
            return Err(E::EntradaPorIdRecusada);
        }
        DestinoDoRedirecionamento::Login => return Err(E::SessaoEncerrada),
        DestinoDoRedirecionamento::Outro => {}
    }
    let origem = OrigemDoTrabalho::EntradaPorId { id: &id };
    let trabalho = match conferir_trabalho(localizacao.as_deref(), &url, instalacao, origem) {
        Ok(redirecionado) => redirecionado.trabalho,
        Err(ErroRedirecionamentoDaPesquisa::IdProcedimentoDoRedirecionamentoDiferenteDoPedido) => {
            contexto.indice.registrar_entrada_por_id(instalacao, true);
            contexto.indice.esquecer_processo(instalacao, contexto.nup);
            return Ok(None);
        }
        Err(erro) => return Err(E::Redirecionamento(erro)),
    };
    contexto.indice.registrar_entrada_por_id(instalacao, true);
    match rede.abrir(contexto, trabalho.as_str(), &id) {
        Ok((arvore, unidade)) => Ok(Some((arvore, Some(unidade), ResolvidoPor::IndiceEntrada))),
        Err(E::Arvore(erro)) if erro.diverge_do_pedido() => {
            contexto.indice.esquecer_processo(instalacao, contexto.nup);
            Ok(None)
        }
        Err(erro) => Err(erro),
    }
}

/// Passo 3, `action` da pesquisa rápida guardado: POST, 302, tela e árvore.
fn pesquisa_guardada<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Rede<'_, V>,
) -> Passo {
    let acao = PESQUISA_RAPIDA.acao;
    let Some(link) = contexto
        .chave(acao, "")
        .and_then(|chave| contexto.indice.link(&contexto.instalacao, chave))
    else {
        return Ok(None);
    };
    let submissao = match derivar_submissao_guardada(
        &PESQUISA_RAPIDA,
        &contexto.instalacao,
        link.url(),
        sessao.url_do_controle,
        &[(CAMPO, contexto.nup)],
    ) {
        Ok(submissao) => submissao,
        // NUP fora do ASCII: o charset da tela não foi guardado.
        Err(ErroSubmissaoFormulario::ValorNaoRepresentavelNaCodificacao { .. }) => return Ok(None),
        Err(_) => {
            contexto.esquecer_link(acao, "");
            return Ok(None);
        }
    };
    let Some(redirecionado) = rede.pesquisar(contexto, submissao)? else {
        contexto.esquecer_link(acao, "");
        return Ok(None);
    };
    let (arvore, unidade) = rede.abrir(
        contexto,
        redirecionado.trabalho.as_str(),
        redirecionado.id.as_str(),
    )?;
    Ok(Some((arvore, Some(unidade), ResolvidoPor::PesquisaRapida)))
}

/// Passo 4, GET do Controle: (a) a linha do NUP exato; (b) senão, a pesquisa
/// rápida do formulário servido. A sessão morta aqui é erro, não palpite.
fn pelo_controle<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Rede<'_, V>,
) -> Result<(ArvoreAberta, Option<String>, ResolvidoPor), E> {
    let busca = SessaoDaBusca {
        url_do_controle: sessao.url_do_controle,
        origem: sessao.origem,
        cookie_file: sessao.cookie_file,
    };
    rede.requisicoes = rede.requisicoes.saturating_add(1);
    let controle = ler_controle(&busca, contexto.unidade_pedida, &mut *rede.envio).map_err(
        |erro| match erro {
            ErroBuscaDeProcesso::Transporte(FalhaTransporte::PaginaDeLoginRecebidaNoControle) => {
                E::SessaoEncerrada
            }
            outro => E::Controle(outro),
        },
    )?;
    let tela = &controle.tela;
    contexto.aprender_pesquisa_rapida(&tela.url, &tela.html, &tela.mime, &controle.unidade);
    if let Some((id, href)) = link_do_processo_no_controle(&tela.html, contexto.nup)? {
        let origem = OrigemDoTrabalho::Controle { id: &id };
        let redirecionado = conferir_trabalho(Some(&href), &tela.url, &contexto.instalacao, origem)
            .map_err(E::LinkDoProcessoNoControleNaoReconhecido)?;
        let (arvore, unidade) = rede.abrir(contexto, redirecionado.trabalho.as_str(), &id)?;
        return Ok((arvore, Some(unidade), ResolvidoPor::Caixa));
    }
    let codificacao = codificacao_da_tela(&tela.mime).ok_or(E::CharsetDoControleNaoSuportado)?;
    let servida = TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao,
    };
    let submissao = derivar_submissao(
        &PESQUISA_RAPIDA,
        &contexto.instalacao,
        &servida,
        &controle.unidade,
        &[(CAMPO, contexto.nup)],
    )
    .map_err(E::Formulario)?;
    let redirecionado = rede
        .pesquisar(contexto, submissao)?
        .ok_or(E::SessaoEncerrada)?;
    let (arvore, unidade) = rede.abrir(
        contexto,
        redirecionado.trabalho.as_str(),
        redirecionado.id.as_str(),
    )?;
    Ok((arvore, Some(unidade), ResolvidoPor::PesquisaRapida))
}
