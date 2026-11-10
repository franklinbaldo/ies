//! Falhas de `ies processo buscar TERMO`: uma variante por etapa do fluxo,
//! cada uma com o subcódigo estável da etapa que falhou.
use super::formulario_pesquisa::ErroPesquisaProtocolo;
use super::link_da_pesquisa::ErroLinkDaPesquisa;
use super::paginacao::ErroPaginacaoDaPesquisa;
use super::resposta::ErroRespostaDaPesquisa;
use crate::erro_web::ErroCaixa;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroBuscaDeProcesso {
    /// GET do Controle ou da tela de pesquisa (inclui sessão vencida).
    #[causa(interna)]
    Transporte(FalhaTransporte),
    /// O Controle não foi reconhecido em nenhuma visão.
    #[causa(interna)]
    Controle(ErroCaixa),
    UnidadeDoControleDiferenteDaSolicitada,
    CharsetDaTelaNaoSuportado,
    #[causa(interna)]
    Link(ErroLinkDaPesquisa),
    #[causa(interna)]
    Pesquisa(ErroPesquisaProtocolo),
    #[causa(interna)]
    Paginacao(ErroPaginacaoDaPesquisa),
}

impl ErroBuscaDeProcesso {
    pub(crate) fn subcodigo(&self) -> &'static str {
        crate::erro_publico::folha(self)
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Transporte(erro) => erro.mensagem(),
            Self::Controle(_) => "Controle de Processos não reconhecido",
            Self::UnidadeDoControleDiferenteDaSolicitada => {
                "a unidade do Controle difere da solicitada"
            }
            Self::CharsetDaTelaNaoSuportado | Self::Link(_) => {
                "a tela de pesquisa não foi obtida como observado"
            }
            Self::Pesquisa(ErroPesquisaProtocolo::Submissao(
                crate::formulario_web::ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(_),
            )) => "o SEI recusou a pesquisa",
            Self::Pesquisa(ErroPesquisaProtocolo::Resposta(
                ErroRespostaDaPesquisa::RespostaDaPesquisaNaoReconhecida,
            )) => "a resposta da pesquisa não foi reconhecida",
            Self::Pesquisa(_) => "a pesquisa não foi concluída como observado",
            Self::Paginacao(ErroPaginacaoDaPesquisa::PaginaForaDoIntervalo { .. }) => {
                "a página pedida não existe no resultado da pesquisa"
            }
            Self::Paginacao(_) => "a página pedida da pesquisa não foi obtida",
        }
    }

    /// Status e mensagem do SEI, da requisição recusada.
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(erro) => erro.status_http(),
            Self::Pesquisa(erro) => erro.status_http(),
            _ => None,
        }
    }
}
