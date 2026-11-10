//! Falhas de `ies processo buscar TERMO`: uma variante por etapa do fluxo,
//! cada uma com o subcódigo estável da etapa que falhou.
use super::formulario_pesquisa::ErroPesquisaProtocolo;
use super::link_da_pesquisa::ErroLinkDaPesquisa;
use super::paginacao::ErroPaginacaoDaPesquisa;
use crate::erro_web::ErroCaixa;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroBuscaDeProcesso {
    /// GET do Controle ou da tela de pesquisa (inclui sessão vencida).
    Transporte(#[from] FalhaTransporte),
    /// O Controle não foi reconhecido em nenhuma visão.
    Controle(#[from] ErroCaixa),
    UnidadeDoControleDiferenteDaSolicitada,
    CharsetDaTelaNaoSuportado,
    Link(#[from] ErroLinkDaPesquisa),
    Pesquisa(#[from] ErroPesquisaProtocolo),
    Paginacao(#[from] ErroPaginacaoDaPesquisa),
}

impl ErroBuscaDeProcesso {
    /// Status e mensagem do SEI, da requisição recusada.
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(erro) => erro.status_http(),
            Self::Pesquisa(erro) => erro.status_http(),
            _ => None,
        }
    }
}
