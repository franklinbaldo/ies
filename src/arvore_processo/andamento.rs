//! "Atualizar Andamento" na barra do nó PROCESSO.
//! Caso: Todos src/todos/andamento.py @323eb01; cassete `UnB`
//! processo-andamento-registrar-unb. Nunca executa JS nem publica URL.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::instalacao::Instalacao;

pub(crate) const ACAO: &str = "procedimento_atualizar_andamento";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroAndamentoNaBarra {
    BarraForaDaForma,
    BarrasRepetidas,
    LinksRepetidos,
    EnderecoNaoReconhecido,
    OutroProcesso,
}

impl ErroAndamentoNaBarra {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::BarraForaDaForma => "barra_do_andamento_fora_da_forma",
            Self::BarrasRepetidas => "barras_do_processo_repetidas_na_arvore",
            Self::LinksRepetidos => "links_de_atualizar_andamento_repetidos",
            Self::EnderecoNaoReconhecido => "endereco_de_atualizar_andamento_nao_reconhecido",
            Self::OutroProcesso => "endereco_de_atualizar_andamento_de_outro_processo",
        }
    }
}

/// Só links publicados na barra conferida; nenhuma URL montada no IES.
pub(crate) fn ler_andamento_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAndamentoNaBarra> {
    use ErroAndamentoNaBarra as E;
    let ancoras = ancoras_da_barra(html).map_err(|f| match f {
        FalhaDaBarra::Fora => E::BarraForaDaForma,
        FalhaDaBarra::Repetidas => E::BarrasRepetidas,
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
                EnderecoRecusado::NaoReconhecido => E::EnderecoNaoReconhecido,
                EnderecoRecusado::DeOutroProcesso => E::OutroProcesso,
            }),
        _ => Err(E::LinksRepetidos),
    }
}
