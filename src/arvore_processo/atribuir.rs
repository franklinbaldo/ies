// Link de atribuir publicado na barra do nó PROCESSO (UnB, GO).
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

const ACAO: &str = "procedimento_atribuicao_cadastrar";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroAtribuirNaBarra {
    BarraForaDaForma,
    BarrasRepetidas,
    LinksRepetidos,
    EnderecoNaoReconhecido,
    EnderecoDeOutroProcesso,
}
impl ErroAtribuirNaBarra {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::BarraForaDaForma => "barra_da_atribuicao_fora_da_forma",
            Self::BarrasRepetidas => "barras_da_atribuicao_repetidas",
            Self::LinksRepetidos => "links_de_atribuir_repetidos_na_barra",
            Self::EnderecoNaoReconhecido => "endereco_da_atribuicao_nao_reconhecido",
            Self::EnderecoDeOutroProcesso => "endereco_da_atribuicao_de_outro_processo",
        }
    }
}

/// Apenas um href assinado e relativo, preso ao ID do processo.
pub(crate) fn ler_atribuir_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAtribuirNaBarra> {
    use ErroAtribuirNaBarra as E;
    let ancoras = ancoras_da_barra(html).map_err(|falha| match falha {
        FalhaDaBarra::Fora => E::BarraForaDaForma,
        FalhaDaBarra::Repetidas => E::BarrasRepetidas,
    })?;
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
                EnderecoRecusado::NaoReconhecido => E::EnderecoNaoReconhecido,
                EnderecoRecusado::DeOutroProcesso => E::EnderecoDeOutroProcesso,
            }),
        _ => Err(E::LinksRepetidos),
    }
}
