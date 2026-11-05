// Link de atribuir publicado na barra do nó PROCESSO (UnB, GO).
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

const ACAO: &str = "procedimento_atribuicao_cadastrar";

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroAtribuirNaBarra {
    #[causa(nome = "endereco_da_atribuicao_de_outro_processo")]
    EnderecoDeOutroProcesso,
    #[causa(interna)]
    Html(crate::erro_html::ErroHtml),
}
impl ErroAtribuirNaBarra {
    pub(crate) fn subcodigo(self) -> &'static str {
        crate::erro_publico::folha(&self)
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
        FalhaDaBarra::Fora => E::Html(crate::erro_html::ErroHtml::valor(
            "barra_da_atribuicao_fora_da_forma",
            "o valor não está na forma observada",
        )),
        FalhaDaBarra::Repetidas => E::Html(crate::erro_html::ErroHtml::repetido(
            "barras_da_atribuicao_repetidas",
            "o elemento aparece mais de uma vez",
        )),
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
                EnderecoRecusado::NaoReconhecido => E::Html(crate::erro_html::ErroHtml::valor(
                    "endereco_da_atribuicao_nao_reconhecido",
                    "o valor não está na forma observada",
                )),
                EnderecoRecusado::DeOutroProcesso => E::EnderecoDeOutroProcesso,
            }),
        _ => Err(E::Html(crate::erro_html::ErroHtml::repetido(
            "links_de_atribuir_repetidos_na_barra",
            "o elemento aparece mais de uma vez",
        ))),
    }
}
