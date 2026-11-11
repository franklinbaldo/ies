//! Falhas de `ies processo buscar TERMO`: um erro por etapa do fluxo,
//! cada uma com o erro da etapa que falhou.
use super::formulario_pesquisa::ErroPesquisaProtocolo;
use super::link_da_pesquisa::ErroLinkDaPesquisa;
use super::paginacao::ErroPaginacaoDaPesquisa;
use crate::erro_web::ErroCaixa;
use crate::transporte::FalhaTransporte;

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
    /// A sessão do SEI acabou (decide o relogin único): por `match` no tipo, sem
    /// percorrer a cadeia `source()`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Transporte(erro) => erro.sessao_encerrada(),
            Self::Pesquisa(erro) => erro.sessao_encerrada(),
            Self::Controle(_)
            | Self::UnidadeDoControleDiferenteDaSolicitada
            | Self::CharsetDaTelaNaoSuportado
            | Self::Link(_)
            | Self::Paginacao(_) => false,
        }
    }
}
