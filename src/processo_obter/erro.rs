//! Falhas de `ies processo NUP` (sem `--resumo`): uma variante por fato
//! verificável, com subcódigo estável (contracts/cli/processo-obter-arvore.md).
use crate::arvore_processo::{ErroArvore, ErroTelaDoProcesso};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso;
use crate::pesquisa_web::redirecionamento::ErroRedirecionamentoDaPesquisa;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroProcessoObter {
    /// Cookie, URL privada, rede ou status de uma requisição.
    Transporte(FalhaTransporte),
    /// GET do Controle (sessão, reconhecimento, unidade): os de `processo buscar`.
    Controle(ErroBuscaDeProcesso),
    /// A entrada por id respondeu a recusa característica ("Link sem
    /// assinatura" no `login.php`); a sessão morreu.
    EntradaPorIdRecusada,
    /// Tela de login (200 ou 302 sem a mensagem de recusa) fora de palpite.
    SessaoEncerrada,
    /// A entrada por id respondeu outra coisa que o 302 observado.
    RespostaDaEntradaPorIdNaoReconhecida(StatusHttpRecusado),
    /// A pesquisa rápida não respondeu o 302 para `procedimento_trabalhar`.
    PesquisaRapidaSemUrlDeTrabalho(StatusHttpRecusado),
    /// 302 fora das formas observadas (nem trabalho, nem login).
    RedirecionamentoNaoReconhecido(StatusHttpRecusado),
    /// O 302 para `procedimento_trabalhar` não passou na conferência.
    Redirecionamento(ErroRedirecionamentoDaPesquisa),
    ProcessoRepetidoNoControle,
    LinksDoProcessoRepetidosNoControle,
    LinkDoProcessoNoControleNaoReconhecido(ErroRedirecionamentoDaPesquisa),
    CharsetDoControleNaoSuportado,
    /// Derivação ou envio do formulário da pesquisa rápida.
    Formulario(ErroSubmissaoFormulario),
    Tela(ErroTelaDoProcesso),
    Arvore(ErroArvore),
}

impl ErroProcessoObter {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Transporte(erro) => erro.subcodigo(),
            Self::Controle(erro) => erro.subcodigo(),
            Self::EntradaPorIdRecusada => "entrada_por_id_recusada_rode_ies_login",
            Self::SessaoEncerrada => "sessao_encerrada_rode_ies_login",
            Self::RespostaDaEntradaPorIdNaoReconhecida(_) => {
                "resposta_da_entrada_por_id_nao_reconhecida"
            }
            Self::PesquisaRapidaSemUrlDeTrabalho(_) => "pesquisa_rapida_sem_url_de_trabalho",
            Self::RedirecionamentoNaoReconhecido(_) => "redirecionamento_nao_reconhecido",
            Self::Redirecionamento(erro) => erro.subcodigo(),
            Self::ProcessoRepetidoNoControle => "processo_repetido_no_controle",
            Self::LinksDoProcessoRepetidosNoControle => "links_do_processo_repetidos_no_controle",
            Self::LinkDoProcessoNoControleNaoReconhecido(_) => {
                "link_do_processo_no_controle_nao_reconhecido"
            }
            Self::CharsetDoControleNaoSuportado => "charset_do_controle_nao_suportado",
            Self::Formulario(erro) => erro.subcodigo(),
            Self::Tela(erro) => erro.subcodigo(),
            Self::Arvore(erro) => erro.subcodigo(),
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Transporte(erro) => erro.mensagem(),
            Self::Controle(erro) => erro.mensagem(),
            Self::EntradaPorIdRecusada => {
                "a instalação recusou a entrada sem assinatura e encerrou a sessão; rode ies login"
            }
            Self::SessaoEncerrada => "o SEI devolveu a tela de login; rode ies login",
            Self::RespostaDaEntradaPorIdNaoReconhecida(_) => {
                "a entrada por id não respondeu como observado"
            }
            Self::PesquisaRapidaSemUrlDeTrabalho(_) => {
                "a pesquisa rápida não abriu o processo (resultado não é lido)"
            }
            Self::RedirecionamentoNaoReconhecido(_) => "o SEI redirecionou para fora do observado",
            Self::Redirecionamento(_) => "o endereço do processo não foi reconhecido",
            Self::ProcessoRepetidoNoControle | Self::LinksDoProcessoRepetidosNoControle => {
                "o número aparece mais de uma vez no Controle"
            }
            Self::LinkDoProcessoNoControleNaoReconhecido(_) => {
                "o link do processo no Controle não foi reconhecido"
            }
            Self::CharsetDoControleNaoSuportado => "o charset do Controle não é suportado",
            Self::Formulario(_) => "a pesquisa rápida não foi enviada como observado",
            Self::Tela(ErroTelaDoProcesso::UnidadeDaTelaDiferenteDaSolicitada) => {
                "a unidade da tela do processo difere da solicitada"
            }
            Self::Tela(_) => "a tela do processo não foi reconhecida",
            Self::Arvore(ErroArvore::IdentidadeDoProcessoDiferenteDaPedida) => {
                "o processo aberto não tem o número pedido"
            }
            Self::Arvore(_) => "a árvore do processo não foi reconhecida",
        }
    }

    /// Status e mensagem do SEI, da requisição recusada.
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(erro) => erro.status_http(),
            Self::Controle(erro) => erro.status_http(),
            Self::RespostaDaEntradaPorIdNaoReconhecida(status)
            | Self::PesquisaRapidaSemUrlDeTrabalho(status)
            | Self::RedirecionamentoNaoReconhecido(status) => Some(status),
            Self::Formulario(erro) => erro.status_http(),
            _ => None,
        }
    }
}
