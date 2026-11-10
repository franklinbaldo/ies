//! Links da árvore e da tela de trabalho guardados no índice local.
//! Um índice velho pode ser esquecido; uma exigência de senha não pode.
use super::{ErroProcessoObter, Passo};
use crate::arvore_processo::{ErroArvore, ErroTelaDoProcesso, ler_tela_do_processo};
use crate::processo_obter::abertura::{Contexto, Lida, Rede, concluir, ler_arvore_aberta};
use crate::processo_obter::pastas::{self, Guardado};
use crate::processo_obter::{EnvioDoProcesso, ResolvidoPor};

/// Passo 1, link guardado nesta geração e unidade: o de "Abrir todas as
/// Pastas", se houver, e senão o da árvore; um GET. A árvore não publica a
/// unidade; com `--unidade` o passo é pulado.
pub(in crate::processo_obter) fn link_da_arvore<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
) -> Passo {
    const ACAO: &str = "procedimento_visualizar";
    if contexto.unidade_pedida.is_some() {
        return Ok(None);
    }
    let instalacao = &contexto.instalacao;
    let Some(id) = contexto.indice.id_por_nup(instalacao, contexto.nup) else {
        return Ok(None);
    };
    match pastas::pelo_link_guardado(contexto, rede, &id)? {
        Guardado::Aberta(aberta) => return Ok(Some((*aberta, None, ResolvidoPor::IndiceLink))),
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
            Err(erro) => return Err(ErroProcessoObter::Arvore(erro)),
        }
    }
    contexto.esquecer_link(ACAO, &id);
    Ok(None)
}

/// Passo 1b, link `procedimento_trabalhar` guardado nesta geração e
/// unidade (colhido de uma linha do Controle ou de uma abertura anterior):
/// GET da tela do processo e GET da árvore.
pub(in crate::processo_obter) fn link_de_trabalho<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
) -> Passo {
    const ACAO: &str = "procedimento_trabalhar";
    let instalacao = &contexto.instalacao;
    let Some(id) = contexto.indice.id_por_nup(instalacao, contexto.nup) else {
        return Ok(None);
    };
    let Some(link) = contexto
        .chave(ACAO, &id)
        .and_then(|chave| contexto.indice.link(instalacao, chave))
    else {
        return Ok(None);
    };
    // Login, redirect ou outro status: o link envelheceu.
    let Lida::Html { html, mime } = rede.get(link.url())? else {
        contexto.esquecer_link(ACAO, &id);
        return Ok(None);
    };
    let tela = match ler_tela_do_processo(instalacao, &html, &id, contexto.unidade_pedida) {
        Ok(tela) => tela,
        // A modal também pode surgir no link já colhido no Controle:
        // validar nesse mesmo link, uma vez, e reler o trabalho e a árvore.
        Err(ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso) => {
            let (arvore, unidade) = crate::processo_obter::validacao_acesso::abrir_trabalho(
                rede,
                contexto,
                link.url(),
                &id,
                (&html, &mime),
            )?;
            return Ok(Some((arvore, Some(unidade), ResolvidoPor::IndiceTrabalhar)));
        }
        // Unidade distinta não é link envelhecido; recusa sem fallback.
        Err(erro @ ErroTelaDoProcesso::UnidadeDaTelaDiferenteDaSolicitada) => {
            return Err(ErroProcessoObter::Tela(erro));
        }
        // Não é a tela deste processo: o link envelheceu.
        Err(_) => {
            contexto.esquecer_link(ACAO, &id);
            return Ok(None);
        }
    };
    match rede.abrir_da_tela(contexto, link.url(), (&html, &mime), tela, &id) {
        Ok((arvore, unidade)) => Ok(Some((arvore, Some(unidade), ResolvidoPor::IndiceTrabalhar))),
        Err(ErroProcessoObter::Arvore(erro)) if erro.diverge_do_pedido() => {
            contexto.indice.esquecer_processo(instalacao, contexto.nup);
            contexto.esquecer_link(ACAO, &id);
            Ok(None)
        }
        Err(erro) => Err(erro),
    }
}
