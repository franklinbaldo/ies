//! Recusas tipadas de documento cancelar. Nenhum tipo armazena URL ou segredo.
use super::ErroLinkCancelar;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_obter::ErroDocumentoObter;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FalhaDoCancelamento {
    Envio(ErroSubmissaoFormulario),
    RespostaSemRedirecionamento,
    RedirecionamentoNaoReconhecido,
}
impl FalhaDoCancelamento {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Envio(erro) => erro.subcodigo(),
            Self::RespostaSemRedirecionamento => "resposta_do_cancelamento_sem_redirecionamento",
            Self::RedirecionamentoNaoReconhecido => {
                "redirecionamento_do_cancelamento_nao_reconhecido"
            }
        }
    }
    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Envio(erro) => erro.status_http().cloned(),
            Self::RedirecionamentoNaoReconhecido => Some(StatusHttpRecusado::sem_mensagem(302)),
            Self::RespostaSemRedirecionamento => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VerificacaoDoCancelamento {
    Lida {
        cancelado: Option<bool>,
        motivo: Option<String>,
    },
    LeituraFalhou(ErroDocumentoObter),
}
impl VerificacaoDoCancelamento {
    pub(crate) fn motivo(&self) -> Option<&'static str> {
        match self {
            Self::Lida {
                cancelado: None, ..
            } => Some("estado_de_cancelamento_nao_observado"),
            Self::Lida { .. } => None,
            Self::LeituraFalhou(erro) => Some(erro.subcodigo()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroCancelar {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(CookieIlegivel),
    Abertura(ErroDocumentoObter),
    Link(ErroLinkCancelar),
    CancelamentoNaoOferecido,
    DocumentoJaCancelado,
    EstadoAntesNaoObservado,
    Tela(FalhaTransporte),
    TelaRedirecionada,
    SessaoEncerrada,
    CharsetNaoSuportado,
    Formulario(ErroSubmissaoFormulario),
    FormularioDeOutroDocumento,
    MotivoNaoPublicadoNaTela,
    /// A leitura nova confirma ausência inequívoca do cancelamento.
    CancelamentoNaoAplicado {
        falha_do_envio: Option<FalhaDoCancelamento>,
    },
    /// Depois da escrita houve mudança ambígua ou a leitura não pôde decidir.
    EfeitoIncerto {
        falha_do_envio: Option<FalhaDoCancelamento>,
        verificacao: Box<VerificacaoDoCancelamento>,
    },
}
impl ErroCancelar {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Cookie(e) => crate::erro_publico::folha(e),
            Self::Abertura(erro) => erro.subcodigo(),
            Self::Link(erro) => erro.subcodigo(),
            Self::CancelamentoNaoOferecido => "cancelamento_nao_oferecido_na_barra",
            Self::DocumentoJaCancelado => "documento_ja_cancelado",
            Self::EstadoAntesNaoObservado => "estado_antes_do_cancelamento_nao_observado",
            Self::Tela(falha) => falha.subcodigo(),
            Self::TelaRedirecionada => "tela_do_cancelamento_redirecionada",
            Self::SessaoEncerrada => "sessao_encerrada_rode_ies_login",
            Self::CharsetNaoSuportado => "charset_da_tela_de_cancelamento_nao_suportado",
            Self::Formulario(erro) => erro.subcodigo(),
            Self::FormularioDeOutroDocumento => "formulario_de_cancelamento_de_outro_documento",
            Self::MotivoNaoPublicadoNaTela => "campo_de_motivo_do_cancelamento_ausente",
            Self::CancelamentoNaoAplicado { .. } => "cancelamento_nao_aplicado",
            Self::EfeitoIncerto { .. } => "efeito_incerto_no_cancelamento",
        }
    }
    pub(crate) fn codigo(&self) -> &'static str {
        match self {
            Self::CancelamentoNaoAplicado { .. } => "escrita_nao_aplicada",
            Self::EfeitoIncerto { .. } => "efeito_incerto",
            _ => "fonte_invalida",
        }
    }
    pub(crate) fn saida(&self) -> u8 {
        match self {
            Self::EfeitoIncerto { .. } => 3,
            _ => 1,
        }
    }
    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Abertura(e) => e.status_http().cloned(),
            Self::Tela(e) => e.status_http().cloned(),
            Self::TelaRedirecionada => Some(StatusHttpRecusado::sem_mensagem(302)),
            Self::Formulario(e) => e.status_http().cloned(),
            Self::CancelamentoNaoAplicado { falha_do_envio }
            | Self::EfeitoIncerto { falha_do_envio, .. } => falha_do_envio
                .as_ref()
                .and_then(FalhaDoCancelamento::status_http),
            _ => None,
        }
    }
    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Cookie(e) => e.mensagem(),
            Self::Abertura(e) => e.mensagem(),
            Self::Link(_) => {
                "a ação de cancelar na barra está fora da forma observada; nada foi enviado"
            }
            Self::CancelamentoNaoOferecido => {
                "a barra do documento não oferece cancelar; nada foi enviado"
            }
            Self::DocumentoJaCancelado => "o documento já está cancelado; nada foi enviado",
            Self::EstadoAntesNaoObservado => {
                "a árvore não prova que o documento esteja ativo; nada foi enviado"
            }
            Self::Tela(_) | Self::TelaRedirecionada => {
                "a tela de cancelamento não pôde ser aberta; nada foi enviado"
            }
            Self::SessaoEncerrada => {
                "sessão encerrada; rode ies login, nenhum cancelamento foi enviado"
            }
            Self::CharsetNaoSuportado => {
                "codificação da tela de cancelamento não suportada; nada foi enviado"
            }
            Self::Formulario(_)
            | Self::FormularioDeOutroDocumento
            | Self::MotivoNaoPublicadoNaTela => {
                "o formulário de cancelamento não está na forma observada; nada foi enviado"
            }
            Self::CancelamentoNaoAplicado { .. } => {
                "a árvore relida não mostra o documento cancelado; nenhum retry foi feito"
            }
            Self::EfeitoIncerto { .. } => {
                "não é possível comprovar o cancelamento; confira a árvore antes de qualquer nova ação"
            }
        }
    }
}

impl From<CookieIlegivel> for ErroCancelar {
    fn from(erro: CookieIlegivel) -> Self {
        Self::Cookie(erro)
    }
}
