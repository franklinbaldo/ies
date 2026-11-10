//! Recusas tipadas de `ies unidade trocar` (#205), uma por fato. Antes do
//! POST nada mudou (saída 1); depois dele, todo desvio é efeito incerto
//! (saída 3), com a etapa e o fato.
use super::tela::UnidadeOferecida;
use crate::erro_html::ErroHtml;
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
#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum FalhaDepoisDoEnvio {
    #[causa(interna)]
    Envio(ErroSubmissaoFormulario),
    #[causa(interna)]
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
    #[causa(interna)]
    ControleNaoReconhecido(ErroCaixa),
    /// O Controle novo mostra outra sigla que a pedida.
    UnidadeDoControleDiferenteDaPedida,
}

impl FalhaDepoisDoEnvio {
    pub(crate) fn subcodigo(&self) -> &'static str {
        crate::erro_publico::folha(self)
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Envio(erro) => erro.status_http(),
            Self::Transporte(falha) => falha.status_http(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroTroca {
    /// GET do Controle ou da tela de troca, antes do POST.
    #[causa(interna)]
    Transporte(FalhaTransporte),
    #[causa(interna)]
    ControleNaoReconhecido(ErroCaixa),
    UnidadeExibidaDiferenteDaSolicitada,
    LinksDeTrocaDivergentes,
    ListaDeUnidadesIncompleta,
    UnidadeMarcadaDiferenteDaAtiva,
    SelecaoDeUnidadePorScriptNaoPublicada,
    UnidadeNaoOferecida {
        oferecidas: Vec<UnidadeOferecida>,
    },
    SiglaDeUnidadeAmbigua {
        candidatas: Vec<UnidadeOferecida>,
    },
    #[causa(interna)]
    Submissao(ErroSubmissaoFormulario),
    EfeitoIncerto {
        etapa: EtapaDaTroca,
        falha: FalhaDepoisDoEnvio,
    },
    #[causa(transparente)]
    Html(ErroHtml),
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
        crate::erro_publico::folha(self)
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Html(e) => e.mensagem,
            Self::Transporte(falha) => falha.mensagem(),
            Self::ControleNaoReconhecido(_) => {
                "resposta não contém Controle de Processos verificável"
            }
            Self::UnidadeExibidaDiferenteDaSolicitada => {
                "a unidade mostrada pelo SEI não corresponde à solicitada"
            }
            Self::LinksDeTrocaDivergentes => {
                "o Controle publica links de troca de unidade diferentes"
            }
            Self::ListaDeUnidadesIncompleta => {
                "a lista de unidades não tem o total que a legenda declara"
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
