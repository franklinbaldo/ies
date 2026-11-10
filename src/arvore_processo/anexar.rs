//! Link observado da ação `procedimento_anexar` na barra do PROCESSO.
//! Endereço assinado validado pela mesma peça das outras escritas.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::erro_html::ErroHtml;
use crate::instalacao::Instalacao;

pub(crate) const ACAO_DE_ANEXAR: &str = "procedimento_anexar";

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAnexarNaBarra {
    EnderecoDeOutroProcesso,
    Html(#[from] ErroHtml),
}
pub(crate) fn ler_anexar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAnexarNaBarra> {
    let ancoras = ancoras_da_barra(html).map_err(|erro| match erro {
        FalhaDaBarra::Fora => {
            ErroAnexarNaBarra::Html(ErroHtml::valor("barra_do_processo_fora_da_forma_observada"))
        }
        FalhaDaBarra::Repetidas => {
            ErroAnexarNaBarra::Html(ErroHtml::repetido("barras_do_processo_repetidas_na_arvore"))
        }
    })?;
    let links: Vec<_> = ancoras
        .iter()
        .filter_map(|ancora| ancora.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(ACAO_DE_ANEXAR))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => conferir_endereco_relativo(instalacao, href, ACAO_DE_ANEXAR, id)
            .map(|url| Some(EnderecoDeEscrita::novo(url)))
            .map_err(|recusa| match recusa {
                EnderecoRecusado::NaoReconhecido => {
                    ErroAnexarNaBarra::Html(ErroHtml::valor("endereco_de_anexar_nao_reconhecido"))
                }
                EnderecoRecusado::DeOutroProcesso => ErroAnexarNaBarra::EnderecoDeOutroProcesso,
            }),
        _ => Err(ErroHtml::repetido("links_de_anexar_repetidos_na_barra").into()),
    }
}
