//! Recusas tipadas de `ies unidade trocar` (#205), uma por fato. Antes do
//! POST nada mudou (saída 1); depois dele, todo desvio é efeito incerto
//! (saída 3), com a etapa e o fato.
use super::tela::UnidadeOferecida;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

/// Onde, depois do POST enviado, a sequência observada se desfez.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EtapaDaTroca {
    Envio,
    PrimeiroRedirecionamento,
    SegundoRedirecionamento,
    ControleDaUnidadeNova,
}

impl EtapaDaTroca {
    pub(crate) fn nome(self) -> &'static str {
        match self {
            Self::Envio => "envio",
            Self::PrimeiroRedirecionamento => "primeiro_redirecionamento",
            Self::SegundoRedirecionamento => "segundo_redirecionamento",
            Self::ControleDaUnidadeNova => "controle_da_unidade_nova",
        }
    }
}

/// O fato observado depois do POST, sem URL nem corpo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FalhaDepoisDoEnvio {
    Envio(ErroSubmissaoFormulario),
    Transporte(FalhaTransporte),
    /// HTML 200 onde o observado é um 302.
    RespostaSemRedirecionamento,
    /// 302 sem `Location`.
    RedirecionamentoSemLocalizacao,
    /// `Location` fora do host, da pasta ou da forma observada.
    LocalizacaoForaDaFormaObservada,
    /// `Location` para outra unidade que a escolhida.
    LocalizacaoDeOutraUnidade,
    /// Status fora de 200 e 302 (recusa do transporte com o status).
    Status(i32),
    ControleNaoReconhecido(ErroCaixa),
    /// O Controle novo mostra outra sigla que a pedida.
    UnidadeDoControleDiferenteDaPedida,
}

impl FalhaDepoisDoEnvio {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Envio(erro) => erro.subcodigo(),
            Self::Transporte(falha) => falha.subcodigo(),
            Self::RespostaSemRedirecionamento => "resposta_sem_redirecionamento",
            Self::RedirecionamentoSemLocalizacao => "redirecionamento_da_troca_sem_localizacao",
            Self::LocalizacaoForaDaFormaObservada => "localizacao_fora_da_forma_observada",
            Self::LocalizacaoDeOutraUnidade => "localizacao_de_outra_unidade",
            Self::Status(_) => "status_inesperado",
            Self::ControleNaoReconhecido(erro) => erro.subcodigo(),
            Self::UnidadeDoControleDiferenteDaPedida => "unidade_do_controle_diferente_da_pedida",
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Envio(erro) => erro.status_http(),
            Self::Transporte(falha) => falha.status_http(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroTroca {
    /// GET do Controle ou da tela de troca, antes do POST.
    Transporte(FalhaTransporte),
    ControleNaoReconhecido(ErroCaixa),
    UnidadeExibidaDiferenteDaSolicitada,
    LinkDeTrocaAusente,
    LinksDeTrocaDivergentes,
    LinkDeTrocaForaDaFormaObservada,
    TelaDeTrocaNaoReconhecida,
    ListaDeUnidadesIncompleta,
    LinhaDeUnidadeForaDaFormaObservada,
    UnidadeMarcadaDiferenteDaAtiva,
    SelecaoDeUnidadePorScriptNaoPublicada,
    UnidadeNaoOferecida {
        oferecidas: Vec<UnidadeOferecida>,
    },
    SiglaDeUnidadeAmbigua {
        candidatas: Vec<UnidadeOferecida>,
    },
    Submissao(ErroSubmissaoFormulario),
    EfeitoIncerto {
        etapa: EtapaDaTroca,
        falha: FalhaDepoisDoEnvio,
    },
}

impl ErroTroca {
    pub(crate) fn codigo(&self) -> &'static str {
        match self {
            Self::EfeitoIncerto { .. } => "efeito_incerto",
            _ => "fonte_invalida",
        }
    }

    /// 3 só para efeito incerto; as demais recusas saem com 1.
    pub(crate) fn saida(&self) -> u8 {
        match self {
            Self::EfeitoIncerto { .. } => 3,
            _ => 1,
        }
    }

    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Transporte(falha) => falha.subcodigo(),
            Self::ControleNaoReconhecido(erro) => erro.subcodigo(),
            Self::UnidadeExibidaDiferenteDaSolicitada => "unidade_exibida_diferente_da_solicitada",
            Self::LinkDeTrocaAusente => "link_de_troca_de_unidade_ausente",
            Self::LinksDeTrocaDivergentes => "links_de_troca_de_unidade_divergentes",
            Self::LinkDeTrocaForaDaFormaObservada => {
                "link_de_troca_de_unidade_fora_da_forma_observada"
            }
            Self::TelaDeTrocaNaoReconhecida => "tela_de_troca_de_unidade_nao_reconhecida",
            Self::ListaDeUnidadesIncompleta => "lista_de_unidades_incompleta",
            Self::LinhaDeUnidadeForaDaFormaObservada => "linha_de_unidade_fora_da_forma_observada",
            Self::UnidadeMarcadaDiferenteDaAtiva => "unidade_marcada_diferente_da_ativa",
            Self::SelecaoDeUnidadePorScriptNaoPublicada => {
                "selecao_de_unidade_por_script_nao_publicada"
            }
            Self::UnidadeNaoOferecida { .. } => "unidade_nao_oferecida",
            Self::SiglaDeUnidadeAmbigua { .. } => "sigla_de_unidade_ambigua",
            Self::Submissao(erro) => erro.subcodigo(),
            Self::EfeitoIncerto { .. } => "efeito_incerto_na_troca_de_unidade",
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Transporte(falha) => falha.mensagem(),
            Self::ControleNaoReconhecido(_) => {
                "resposta não contém Controle de Processos verificável"
            }
            Self::UnidadeExibidaDiferenteDaSolicitada => {
                "a unidade mostrada pelo SEI não corresponde à solicitada"
            }
            Self::LinkDeTrocaAusente => "o Controle não publica o link de troca de unidade",
            Self::LinksDeTrocaDivergentes => {
                "o Controle publica links de troca de unidade diferentes"
            }
            Self::LinkDeTrocaForaDaFormaObservada => {
                "o link de troca de unidade não tem a forma observada"
            }
            Self::TelaDeTrocaNaoReconhecida => "a tela de troca de unidade não foi reconhecida",
            Self::ListaDeUnidadesIncompleta => {
                "a lista de unidades não tem o total que a legenda declara"
            }
            Self::LinhaDeUnidadeForaDaFormaObservada => {
                "uma linha da lista de unidades não tem a forma observada"
            }
            Self::UnidadeMarcadaDiferenteDaAtiva => {
                "a unidade marcada na tela de troca não é a ativa no Controle"
            }
            Self::SelecaoDeUnidadePorScriptNaoPublicada => {
                "a tela não publica selecionarUnidade() com selInfraUnidades"
            }
            Self::UnidadeNaoOferecida { .. } => "a tela de troca não oferece a unidade pedida",
            Self::SiglaDeUnidadeAmbigua { .. } => {
                "a sigla pedida aparece em mais de uma unidade da tela de troca"
            }
            Self::Submissao(_) => "o formulário de troca de unidade não pôde ser derivado",
            Self::EfeitoIncerto { .. } => {
                "a troca foi enviada e a resposta não é a observada: a unidade da sessão pode ter mudado; não repita, confira com ies login --forcar"
            }
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(falha) => falha.status_http(),
            Self::Submissao(erro) => erro.status_http(),
            Self::EfeitoIncerto { falha, .. } => falha.status_http(),
            _ => None,
        }
    }
}
