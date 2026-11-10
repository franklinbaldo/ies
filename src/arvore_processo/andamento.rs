//! "Atualizar Andamento" na barra do nó PROCESSO.
//! Caso: Todos src/todos/andamento.py @323eb01; cassete `UnB`
//! processo-andamento-registrar-unb. Nunca executa JS nem publica URL.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

pub(crate) const ACAO: &str = "procedimento_atualizar_andamento";

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAndamentoNaBarra {
    OutroProcesso,
    BarraDoAndamentoForaDaForma,
    BarrasDoProcessoRepetidasNaArvore,
    EnderecoDeAtualizarAndamentoNaoReconhecido,
    LinksDeAtualizarAndamentoRepetidos,
}

/// Só links publicados na barra conferida; nenhuma URL montada no IES.
pub(crate) fn ler_andamento_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAndamentoNaBarra> {
    let ancoras = ancoras_da_barra(html).map_err(|f| match f {
        FalhaDaBarra::Fora => ErroAndamentoNaBarra::BarraDoAndamentoForaDaForma,
        FalhaDaBarra::Repetidas => ErroAndamentoNaBarra::BarrasDoProcessoRepetidasNaArvore,
    })?;
    let links: Vec<&str> = ancoras
        .iter()
        .filter_map(|a| a.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(ACAO))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => conferir_endereco_relativo(instalacao, href, ACAO, id)
            .map(|url| Some(EnderecoDeEscrita::novo(url)))
            .map_err(|e| match e {
                EnderecoRecusado::NaoReconhecido => {
                    ErroAndamentoNaBarra::EnderecoDeAtualizarAndamentoNaoReconhecido
                }
                EnderecoRecusado::DeOutroProcesso => ErroAndamentoNaBarra::OutroProcesso,
            }),
        _ => Err(ErroAndamentoNaBarra::LinksDeAtualizarAndamentoRepetidos),
    }
}
