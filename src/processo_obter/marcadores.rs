//! Marcadores depois das propriedades (contracts/cli/processo-obter-marcadores.md).
//! A barra da árvore recém-lida diz se a tela `andamento_marcador_gerenciar`
//! é oferecida; sem ela, nada é pedido. No caminho 1 o link guardado vem
//! primeiro; login, redirect ou página que não é a tela deste processo o
//! apagam e o link da barra segue. Status de falha: erro de transporte, sem
//! retry. Um GET; a escrita da tela (adicionar, remover, alterar) nunca é
//! tocada.
use super::abertura::{ArvoreAberta, Contexto, Lida, Rede, fora_de_palpite};
use super::{EnvioDoProcesso, ErroProcessoObter as E, Marcadores};
use crate::marcadores_processo::{ACAO, TelaDosMarcadores, ler_marcadores};

pub(super) fn ler<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    aberta: &ArvoreAberta,
    pedidos: bool,
    guardado_primeiro: bool,
) -> Result<Marcadores, E> {
    if !pedidos {
        return Ok(Marcadores::Omitidos);
    }
    let endereco = match &aberta.marcadores {
        Ok(Some(endereco)) => endereco,
        Ok(None) => return Ok(Marcadores::NaoPublicadosNaBarra),
        Err(erro) => return Err(E::Marcadores(*erro)),
    };
    let id = aberta.arvore.processo.id.as_str();
    let guardada = if guardado_primeiro {
        pelo_guardado(contexto, rede, id)?
    } else {
        None
    };
    let (url, tela) = if let Some(lida) = guardada {
        lida
    } else {
        let url = endereco.as_str();
        (url.to_owned(), pelo_link_da_barra(contexto, rede, url, id)?)
    };
    contexto.aprender_link(ACAO, id, &url);
    Ok(match tela {
        TelaDosMarcadores::Lidos(lidos) => Marcadores::Lidos(lidos),
        TelaDosMarcadores::SemTabela => Marcadores::TabelaNaoPublicada,
    })
}

/// Link guardado nesta geração e unidade. `Ok(None)`: não havia, ou o
/// palpite errou e foi esquecido.
fn pelo_guardado<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    id: &str,
) -> Result<Option<(String, TelaDosMarcadores)>, E> {
    let Some(link) = contexto
        .chave(ACAO, id)
        .and_then(|chave| contexto.indice.link(&contexto.instalacao, chave))
    else {
        return Ok(None);
    };
    let url = link.url().to_owned();
    match rede.get(&url)? {
        Lida::Html { html, .. } => match ler_marcadores(&html, contexto.nup, id) {
            Ok(tela) => Ok(Some((url, tela))),
            Err(erro) if erro.link_envelhecido() => {
                contexto.esquecer_link(ACAO, id);
                Ok(None)
            }
            Err(erro) => Err(E::Marcadores(erro)),
        },
        Lida::Login | Lida::Redirecionamento(_) => {
            contexto.esquecer_link(ACAO, id);
            Ok(None)
        }
        Lida::Falha(falha) => Err(E::Transporte(falha)),
    }
}

fn pelo_link_da_barra<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    url: &str,
    id: &str,
) -> Result<TelaDosMarcadores, E> {
    match rede.get(url)? {
        Lida::Html { html, .. } => ler_marcadores(&html, contexto.nup, id).map_err(E::Marcadores),
        Lida::Redirecionamento(local) => Err(fora_de_palpite(local.as_deref())),
        Lida::Login => Err(E::SessaoEncerrada),
        Lida::Falha(falha) => Err(E::Transporte(falha)),
    }
}
