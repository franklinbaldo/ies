//! Marcadores depois das propriedades (contracts/cli/processo-obter-marcadores.md).
//! A barra da árvore recém-lida diz se a tela `andamento_marcador_gerenciar`
//! é oferecida; sem ela, nada é pedido. No caminho 1 o link guardado vem
//! primeiro; login, redirect ou página que não é a tela deste processo o
//! apagam e o link da barra segue. Status de falha: erro de transporte, sem
//! retry. Um GET; a escrita da tela (adicionar, remover, alterar) nunca é
//! tocada.
use super::abertura::{ArvoreAberta, Contexto, Lida, Rede, fora_de_palpite};
use super::{EnvioDoProcesso, ErroProcessoObter, Marcadores};
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::marcadores_processo::{MarcadoresLidos, TelaDosMarcadores, ler_marcadores};

/// 302 para `andamento_marcador_cadastrar` assinado do mesmo processo:
/// a tela não tem o que listar. Nunca é seguido.
fn sem_marcadores(contexto: &Contexto<'_>, local: Option<&str>, id: &str) -> bool {
    let Some(local) = local else { return false };
    let base = format!("{}/", contexto.instalacao.url_base());
    let relativo = local.strip_prefix(base.as_str()).unwrap_or(local);
    conferir_endereco_relativo(
        &contexto.instalacao,
        relativo,
        AcaoDoSei::AndamentoMarcadorCadastrar,
        id,
    )
    .is_ok()
}

fn vazia() -> TelaDosMarcadores {
    TelaDosMarcadores::Lidos(MarcadoresLidos {
        total_declarado: 0,
        itens: Vec::new(),
    })
}

pub(super) fn ler<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    aberta: &ArvoreAberta,
    pedidos: bool,
    guardado_primeiro: bool,
) -> Result<Marcadores, ErroProcessoObter> {
    if !pedidos {
        return Ok(Marcadores::Omitidos);
    }
    let endereco = match &aberta.marcadores {
        Ok(Some(endereco)) => endereco,
        Ok(None) => return Ok(Marcadores::NaoPublicadosNaBarra),
        Err(erro) => return Err(ErroProcessoObter::Marcadores(erro.clone())),
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
    contexto.aprender_link(AcaoDoSei::AndamentoMarcadorGerenciar, id, &url);
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
) -> Result<Option<(String, TelaDosMarcadores)>, ErroProcessoObter> {
    let Some(link) = contexto
        .chave(AcaoDoSei::AndamentoMarcadorGerenciar, id)
        .and_then(|chave| contexto.indice.link(&contexto.instalacao, chave))
    else {
        return Ok(None);
    };
    let url = link.url().to_owned();
    match rede.get(&url)? {
        Lida::Html { html, .. } => match ler_marcadores(&html, contexto.nup, id) {
            Ok(tela) => Ok(Some((url, tela))),
            Err(erro) if erro.link_envelhecido() => {
                contexto.esquecer_link(AcaoDoSei::AndamentoMarcadorGerenciar, id);
                Ok(None)
            }
            Err(erro) => Err(ErroProcessoObter::Marcadores(erro)),
        },
        Lida::Redirecionamento(local) if sem_marcadores(contexto, local.as_deref(), id) => {
            Ok(Some((url, vazia())))
        }
        Lida::Login | Lida::Redirecionamento(_) => {
            contexto.esquecer_link(AcaoDoSei::AndamentoMarcadorGerenciar, id);
            Ok(None)
        }
        Lida::Falha(falha) => Err(ErroProcessoObter::Transporte(falha)),
    }
}

fn pelo_link_da_barra<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    url: &str,
    id: &str,
) -> Result<TelaDosMarcadores, ErroProcessoObter> {
    match rede.get(url)? {
        Lida::Html { html, .. } => {
            ler_marcadores(&html, contexto.nup, id).map_err(ErroProcessoObter::Marcadores)
        }
        Lida::Redirecionamento(local) if sem_marcadores(contexto, local.as_deref(), id) => {
            Ok(vazia())
        }
        Lida::Redirecionamento(local) => Err(fora_de_palpite(local.as_deref())),
        Lida::Login => Err(ErroProcessoObter::SessaoEncerrada(
            crate::erro_publico::SessaoEncerrada,
        )),
        Lida::Falha(falha) => Err(ErroProcessoObter::Transporte(falha)),
    }
}
