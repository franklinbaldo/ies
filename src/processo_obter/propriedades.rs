//! Propriedades depois da árvore (contracts/cli/processo-obter-propriedades.md).
//! A barra da árvore recém-lida diz qual tela é oferecida
//! (`procedimento_alterar` ou `procedimento_consultar`); sem nenhuma, nada
//! é pedido. No caminho 1 o link guardado da mesma `acao` vem primeiro;
//! login, redirect ou página que não é o formulário deste processo o
//! apagam e o link da barra segue. Status de falha: erro de transporte,
//! sem retry. Um GET; o formulário nunca é enviado.
use super::abertura::{ArvoreAberta, Contexto, Lida, Rede, fora_de_palpite};
use super::{EnvioDoProcesso, ErroProcessoObter as E, Propriedades};
use crate::propriedades_processo::{FonteDasPropriedades, PropriedadesLidas, ler_propriedades};

pub(super) fn ler<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    aberta: &ArvoreAberta,
    pedidas: bool,
    guardado_primeiro: bool,
) -> Result<Propriedades, E> {
    if !pedidas {
        return Ok(Propriedades::Omitidas);
    }
    let endereco = match &aberta.propriedades {
        Ok(Some(endereco)) => endereco,
        Ok(None) => return Ok(Propriedades::NaoPublicadasNaBarra),
        Err(erro) => return Err(E::Propriedades(*erro)),
    };
    let id = aberta.arvore.processo.id.as_str();
    let fonte = endereco.fonte;
    let guardada = if guardado_primeiro {
        pelo_guardado(contexto, rede, id, fonte)?
    } else {
        None
    };
    let (url, lidas) = if let Some(lida) = guardada {
        lida
    } else {
        let url = endereco.as_str();
        (
            url.to_owned(),
            pelo_link_da_barra(contexto, rede, url, id, fonte)?,
        )
    };
    contexto.aprender_link(fonte.acao(), id, &url);
    Ok(Propriedades::Lidas(lidas))
}

/// Link guardado nesta geração e unidade. `Ok(None)`: não havia, ou o
/// palpite errou e foi esquecido.
fn pelo_guardado<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    id: &str,
    fonte: FonteDasPropriedades,
) -> Result<Option<(String, PropriedadesLidas)>, E> {
    let acao = fonte.acao();
    let Some(link) = contexto
        .chave(acao, id)
        .and_then(|chave| contexto.indice.link(&contexto.instalacao, chave))
    else {
        return Ok(None);
    };
    let url = link.url().to_owned();
    match rede.get(&url)? {
        Lida::Html { html, .. } => match ler_propriedades(&html, contexto.nup, id, fonte) {
            Ok(lidas) => Ok(Some((url, lidas))),
            Err(erro) if erro.link_envelhecido() => {
                contexto.esquecer_link(acao, id);
                Ok(None)
            }
            Err(erro) => Err(E::Propriedades(erro)),
        },
        Lida::Login | Lida::Redirecionamento(_) => {
            contexto.esquecer_link(acao, id);
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
    fonte: FonteDasPropriedades,
) -> Result<PropriedadesLidas, E> {
    match rede.get(url)? {
        Lida::Html { html, .. } => {
            ler_propriedades(&html, contexto.nup, id, fonte).map_err(E::Propriedades)
        }
        Lida::Redirecionamento(local) => Err(fora_de_palpite(local.as_deref())),
        Lida::Login => Err(E::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)),
        Lida::Falha(falha) => Err(E::Transporte(falha)),
    }
}
