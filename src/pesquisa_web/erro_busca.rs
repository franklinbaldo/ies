//! Falhas de `ies processo buscar TERMO`: uma variante por etapa do fluxo,
//! cada uma com o subcódigo estável da etapa que falhou.
use super::formulario_pesquisa::ErroPesquisaProtocolo;
use super::link_da_pesquisa::ErroLinkDaPesquisa;
use super::paginacao::ErroPaginacaoDaPesquisa;
use super::resposta::ErroRespostaDaPesquisa;
use crate::erro_web::ErroCaixa;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroBuscaDeProcesso {
    /// GET do Controle ou da tela de pesquisa (inclui sessão vencida).
    Transporte(FalhaTransporte),
    /// O Controle não foi reconhecido em nenhuma visão.
    Controle(ErroCaixa),
    UnidadeDoControleDiferenteDaSolicitada,
    CharsetDaTelaNaoSuportado,
    Link(ErroLinkDaPesquisa),
    Pesquisa(ErroPesquisaProtocolo),
    Paginacao(ErroPaginacaoDaPesquisa),
}

impl ErroBuscaDeProcesso {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Transporte(erro) => erro.subcodigo(),
            Self::Controle(erro) => erro.subcodigo(),
            Self::UnidadeDoControleDiferenteDaSolicitada => {
                "unidade_da_tela_diferente_da_solicitada"
            }
            Self::CharsetDaTelaNaoSuportado => "charset_da_tela_da_pesquisa_nao_suportado",
            Self::Link(erro) => erro.subcodigo(),
            Self::Pesquisa(erro) => erro.subcodigo(),
            Self::Paginacao(erro) => erro.subcodigo(),
        }
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
