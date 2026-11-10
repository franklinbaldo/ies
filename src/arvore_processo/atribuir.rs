// Link de atribuir publicado na barra do nó PROCESSO (UnB, GO).
use super::barra_do_processo::{acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

const ACAO: &str = "procedimento_atribuicao_cadastrar";

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAtribuirNaBarra {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    EnderecoDeOutroProcesso,
    EnderecoDaAtribuicaoNaoReconhecido,
    LinksDeAtribuirRepetidosNaBarra,
}

/// Apenas um href assinado e relativo, preso ao ID do processo.
pub(crate) fn ler_atribuir_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAtribuirNaBarra> {
    let ancoras = ancoras_da_barra(html)?;
    let links: Vec<&str> = ancoras
        .iter()
        .filter_map(|ancora| ancora.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(ACAO))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => conferir_endereco_relativo(instalacao, href, ACAO, id)
            .map(|url| Some(EnderecoDeEscrita::novo(url)))
            .map_err(|erro| match erro {
                EnderecoRecusado::NaoReconhecido => {
                    ErroAtribuirNaBarra::EnderecoDaAtribuicaoNaoReconhecido
                }
                EnderecoRecusado::DeOutroProcesso => ErroAtribuirNaBarra::EnderecoDeOutroProcesso,
            }),
        _ => Err(ErroAtribuirNaBarra::LinksDeAtribuirRepetidosNaBarra),
    }
}
