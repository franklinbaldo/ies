//! Os passos da resolução (1, 1b, 2, 3 e 4) (contracts/cli/processo-obter-arvore.md).
//! Um passo é tentado só se o anterior não se aplica ou falhou por palpite
//! do índice; o palpite errado é esquecido e o comando segue.
use super::abertura::{ArvoreAberta, CAMPO, Contexto, Lida, PESQUISA_RAPIDA, Rede};
pub(crate) use super::acoes::AcoesDeEstado;
use super::controle::link_do_processo_no_controle;
use super::login::{DestinoDoRedirecionamento, destino};
use super::{
    EnvioDoProcesso, ErroProcessoObter, PedidoDoProcesso, ProcessoAberto, ResolvidoPor,
    SessaoDoProcesso,
};
use crate::colheita_do_controle::Colheita;
use crate::formulario_web::{
    ErroSubmissaoFormulario, TelaServida, derivar_submissao, derivar_submissao_guardada,
};
use crate::indice_local::{IndiceLocal, unidade_dos_links, url_da_entrada_por_id};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso;
use crate::pesquisa_web::fluxo::{ControleLido, SessaoDaBusca, ler_controle};
use crate::pesquisa_web::redirecionamento::{
    ErroRedirecionamentoDaPesquisa, OrigemDoTrabalho, ProcessoRedirecionado, conferir_trabalho,
};
use crate::transporte::FalhaTransporte;
use crate::transporte::contagem::ContagemDeRequisicoes;

mod links_guardados;
pub(super) use links_guardados::{link_da_arvore, link_de_trabalho};

/// Resultado de um passo: aberto, ou "não se aplica / palpite esquecido".
type Passo = Result<Option<(ArvoreAberta, Option<String>, ResolvidoPor)>, ErroProcessoObter>;

pub(crate) fn obter_processo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoProcesso<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoAberto, ErroProcessoObter> {
    abrir_para_escrever(sessao, pedido, indice, envio).map(|(aberto, _)| aberto)
}

/// Como [`obter_processo`], devolvendo também as ações de concluir e de
/// reabrir da barra da árvore lida.
pub(crate) fn abrir_para_escrever<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoProcesso<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<(ProcessoAberto, AcoesDeEstado), ErroProcessoObter> {
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(
        ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
    )?;
    let cookie = sessao.cookie.valor()?.to_owned();
    let contexto = Contexto {
        instalacao,
        nup: pedido.nup.as_str(),
        unidade_pedida: pedido.unidade,
        geracao: crate::sessao_login::geracao_da_sessao(&cookie),
        unidade_id: unidade_dos_links(sessao.url_do_controle),
        indice,
        cookie: sessao.cookie,
        documento_procurado: None,
        permitir_leitura_parcial: matches!(
            pedido.leitura_das_pastas,
            super::LeituraDasPastas::Informativa
        ),
    };
    let mut rede = Rede {
        envio,
        cookie,
        requisicoes: ContagemDeRequisicoes::default(),
    };
    let (arvore_lida, unidade, resolvido_por) =
        abrir_processo(&contexto, sessao, &mut rede, pedido.unidade_exigida)?;
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
    let propriedades = super::propriedades::ler(
        &contexto,
        &mut rede,
        &arvore_lida,
        pedido.propriedades,
        resolvido_por == ResolvidoPor::IndiceLink,
    )?;
    let marcadores = super::marcadores::ler(
        &contexto,
        &mut rede,
        &arvore_lida,
        pedido.marcadores,
        resolvido_por == ResolvidoPor::IndiceLink,
    )?;
    let aberto = ProcessoAberto {
        arvore: arvore_lida.arvore,
        unidade,
        resolvido_por,
        andamentos,
        propriedades,
        marcadores,
        requisicoes: rede.requisicoes.total_u8(),
    };
    let acoes = AcoesDeEstado {
        incluir: arvore_lida.incluir,
        marcadores: arvore_lida.marcadores,
        prazo: arvore_lida.prazo,
        concluir: arvore_lida.concluir,
        reabrir: arvore_lida.reabrir,
        sobrestar: arvore_lida.sobrestar,
        retomar: arvore_lida.retomar,
        enviar: arvore_lida.enviar,
        anotacoes: arvore_lida.anotacoes,
        comentarios: arvore_lida.comentarios,
        ciencia: arvore_lida.ciencia,
        lista_ciencias: arvore_lida.lista_ciencias,
        anexar: arvore_lida.anexar,
        atribuir: arvore_lida.atribuir,
        andamento: arvore_lida.andamento,
        relacionar: arvore_lida.relacionar,
        acompanhar: arvore_lida.acompanhar,
    };
    Ok((aberto, acoes))
}

/// A cadeia de passos até a árvore conferida (com as pastas), e o que ela
/// ensina ao índice sem requisição: anexados, pai, movidos e documentos.
/// Também a abertura de `documento DOC --processo NUP`. Com
/// `unidade_exigida` (`processo concluir`), o passo 1 é pulado: o link
/// guardado da árvore não publica a unidade.
pub(super) fn abrir_processo<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Rede<'_, V>,
    unidade_exigida: bool,
) -> Result<(ArvoreAberta, Option<String>, ResolvidoPor), ErroProcessoObter> {
    let primeiro = if unidade_exigida {
        None
    } else {
        link_da_arvore(contexto, rede)?
    };
    let aberto = match primeiro {
        Some(aberto) => aberto,
        None => match link_de_trabalho(contexto, rede)? {
            Some(aberto) => aberto,
            None => match entrada_por_id(contexto, rede)? {
                Some(aberto) => aberto,
                None => match pesquisa_guardada(contexto, sessao, rede)? {
                    Some(aberto) => aberto,
                    None => pelo_controle(contexto, sessao, rede)?,
                },
            },
        },
    };
    let (arvore_lida, unidade, resolvido_por) = aberto;
    // Cada anexado da árvore conferida: par NUP ↔ id aprendido sem requisição.
    for anexado in &arvore_lida.arvore.anexados {
        let _ = contexto
            .indice
            .registrar_processo(&contexto.instalacao, &anexado.nup, &anexado.id);
    }
    // Aberto um anexado: o pai e o link dele, para o passo 1b.
    if let Some(pai) = &arvore_lida.arvore.anexado_a {
        let _ = contexto.indice.registrar_processo(
            &contexto.instalacao,
            &pai.numero,
            &pai.id_procedimento,
        );
        contexto.aprender_link(
            "procedimento_trabalhar",
            &pai.id_procedimento,
            pai.trabalhar.as_str(),
        );
    }
    // Cada movido com destino conferido: o par e o link assinado do destino.
    let destinos = arvore_lida.arvore.documentos_movidos.iter();
    for destino in destinos.filter_map(|m| m.destino.as_ref()) {
        let _ = contexto.indice.registrar_processo(
            &contexto.instalacao,
            &destino.numero,
            &destino.id_procedimento,
        );
        contexto.aprender_link(
            "procedimento_trabalhar",
            &destino.id_procedimento,
            destino.trabalhar.as_str(),
        );
    }
    contexto.aprender_documentos(&arvore_lida.arvore);
    Ok((arvore_lida, unidade, resolvido_por))
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
        Lida::Login => {
            return Err(ErroProcessoObter::SessaoEncerrada(
                crate::erro_publico::SessaoEncerrada,
            ));
        }
        Lida::Html { .. } => return Err(ErroProcessoObter::RespostaDaEntradaPorIdNaoReconhecida),
        Lida::Falha(falha) => return Err(ErroProcessoObter::Transporte(falha)),
    };
    match destino(localizacao.as_deref()) {
        DestinoDoRedirecionamento::RecusaSemAssinatura => {
            // A recusa derrubou a sessão: registrar e parar, sem relogin.
            contexto.indice.registrar_entrada_por_id(instalacao, false);
            return Err(ErroProcessoObter::EntradaPorIdRecusada);
        }
        DestinoDoRedirecionamento::Login => {
            return Err(ErroProcessoObter::SessaoEncerrada(
                crate::erro_publico::SessaoEncerrada,
            ));
        }
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
        Err(erro) => return Err(ErroProcessoObter::Redirecionamento(erro)),
    };
    contexto.indice.registrar_entrada_por_id(instalacao, true);
    match rede.abrir(contexto, trabalho.as_str(), &id) {
        Ok((arvore, unidade)) => Ok(Some((arvore, Some(unidade), ResolvidoPor::IndiceEntrada))),
        Err(ErroProcessoObter::Arvore(erro)) if erro.diverge_do_pedido() => {
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
    let Some(redirecionado) = pesquisar_pela_guardada(contexto, sessao, rede, contexto.nup)? else {
        return Ok(None);
    };
    let (arvore, unidade) = rede.abrir(
        contexto,
        redirecionado.trabalho.as_str(),
        redirecionado.id.as_str(),
    )?;
    Ok(Some((arvore, Some(unidade), ResolvidoPor::PesquisaRapida)))
}

/// POST de `valor` (NUP ou número SEI) ao `action` da pesquisa rápida
/// guardado; `Ok(None)`: não há guardado ou ele envelheceu (esquecido).
pub(super) fn pesquisar_pela_guardada<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Rede<'_, V>,
    valor: &str,
) -> Result<Option<ProcessoRedirecionado>, ErroProcessoObter> {
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
        &[(CAMPO, valor)],
    ) {
        Ok(submissao) => submissao,
        // Valor fora do ASCII: o charset da tela não foi guardado.
        Err(ErroSubmissaoFormulario::ValorNaoRepresentavelNaCodificacao { .. }) => return Ok(None),
        Err(_) => {
            contexto.esquecer_link(acao, "");
            return Ok(None);
        }
    };
    let Some(redirecionado) = rede.pesquisar(contexto, submissao, valor)? else {
        contexto.esquecer_link(acao, "");
        return Ok(None);
    };
    Ok(Some(redirecionado))
}

/// Passo 4, GET do Controle: (a) a linha do NUP exato; (b) senão, a pesquisa
/// rápida do formulário servido. A sessão morta aqui é erro, não palpite.
fn pelo_controle<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Rede<'_, V>,
) -> Result<(ArvoreAberta, Option<String>, ResolvidoPor), ErroProcessoObter> {
    let controle = ler_controle_colhido(contexto, sessao, rede)?;
    let tela = &controle.tela;
    if let Some((id, href)) = link_do_processo_no_controle(&tela.html, contexto.nup)? {
        let origem = OrigemDoTrabalho::Controle { id: &id };
        let redirecionado = conferir_trabalho(Some(&href), &tela.url, &contexto.instalacao, origem)
            .map_err(ErroProcessoObter::LinkDoProcessoNoControleNaoReconhecido)?;
        let (arvore, unidade) = rede
            .abrir(contexto, redirecionado.trabalho.as_str(), &id)
            .inspect_err(|erro| {
                // A colheita deste GET gravou o par que a árvore desmentiu.
                if matches!(erro, ErroProcessoObter::Arvore(arvore) if arvore.diverge_do_pedido()) {
                    contexto
                        .indice
                        .esquecer_processo(&contexto.instalacao, contexto.nup);
                }
            })?;
        return Ok((arvore, Some(unidade), ResolvidoPor::Caixa));
    }
    let redirecionado = pesquisar_no_controle(contexto, &controle, rede, contexto.nup)?;
    let (arvore, unidade) = rede.abrir(
        contexto,
        redirecionado.trabalho.as_str(),
        redirecionado.id.as_str(),
    )?;
    Ok((arvore, Some(unidade), ResolvidoPor::PesquisaRapida))
}

/// GET do Controle; linhas, link da pesquisa e `action` da pesquisa rápida
/// colhidos no índice, sem requisição.
pub(super) fn ler_controle_colhido<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Rede<'_, V>,
) -> Result<ControleLido, ErroProcessoObter> {
    let busca = SessaoDaBusca {
        url_do_controle: sessao.url_do_controle,
        origem: sessao.origem,
        cookie: sessao.cookie,
    };
    rede.requisicoes.registrar();
    let controle = ler_controle(&busca, contexto.unidade_pedida, &mut *rede.envio).map_err(
        |erro| match erro {
            ErroBuscaDeProcesso::Transporte(FalhaTransporte::PaginaDeLoginRecebidaNoControle) => {
                ErroProcessoObter::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
            }
            outro => ErroProcessoObter::Controle(outro),
        },
    )?;
    Colheita::com_geracao(
        contexto.indice,
        &contexto.instalacao,
        &contexto.geracao,
        sessao.url_do_controle,
    )
    .colher(&controle.tela.html, Some(&controle.tela.mime));
    Ok(controle)
}

/// POST de `valor` pelo formulário rápido servido no Controle lido.
pub(super) fn pesquisar_no_controle<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    controle: &ControleLido,
    rede: &mut Rede<'_, V>,
    valor: &str,
) -> Result<ProcessoRedirecionado, ErroProcessoObter> {
    let tela = &controle.tela;
    let codificacao =
        codificacao_da_tela(&tela.mime).ok_or(ErroProcessoObter::CharsetDoControleNaoSuportado)?;
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
        &[(CAMPO, valor)],
    )?;
    rede.pesquisar(contexto, submissao, valor)?
        .ok_or(ErroProcessoObter::SessaoEncerrada(
            crate::erro_publico::SessaoEncerrada,
        ))
}
