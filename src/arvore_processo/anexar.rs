//! Link observado da ação procedimento_anexar na barra do PROCESSO.
//! Endereço assinado validado pela mesma peça das outras escritas.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

pub(crate) const ACAO_DE_ANEXAR: &str = "procedimento_anexar";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroAnexarNaBarra {
    BarraForaDaFormaObservada,
    BarrasRepetidas,
    LinksRepetidos,
    EnderecoNaoReconhecido,
    EnderecoDeOutroProcesso,
}
impl ErroAnexarNaBarra {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::BarraForaDaFormaObservada => "barra_do_processo_fora_da_forma_observada",
            Self::BarrasRepetidas => "barras_do_processo_repetidas_na_arvore",
            Self::LinksRepetidos => "links_de_anexar_repetidos_na_barra",
            Self::EnderecoNaoReconhecido => "endereco_de_anexar_nao_reconhecido",
            Self::EnderecoDeOutroProcesso => "endereco_de_anexar_de_outro_processo",
        }
    }
}
pub(crate) fn ler_anexar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAnexarNaBarra> {
    use ErroAnexarNaBarra as E;
    let ancoras = ancoras_da_barra(html).map_err(|erro| match erro {
        FalhaDaBarra::Fora => E::BarraForaDaFormaObservada,
        FalhaDaBarra::Repetidas => E::BarrasRepetidas,
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
                EnderecoRecusado::NaoReconhecido => E::EnderecoNaoReconhecido,
                EnderecoRecusado::DeOutroProcesso => E::EnderecoDeOutroProcesso,
            }),
        _ => Err(E::LinksRepetidos),
    }
}
