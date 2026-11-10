//! Recusas tipadas de documento cancelar. Nenhum tipo armazena URL ou segredo.
use super::ErroLinkCancelar;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_obter::ErroDocumentoObter;
use crate::transporte::FalhaTransporte;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FalhaDoCancelamento {
    Envio(ErroSubmissaoFormulario),
    RespostaSemRedirecionamento,
    RedirecionamentoNaoReconhecido,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VerificacaoDoCancelamento {
    Lida {
        cancelado: Option<bool>,
        motivo: Option<String>,
    },
    LeituraFalhou(ErroDocumentoObter),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
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
    NaoConfirmado {
        falha_do_envio: Option<FalhaDoCancelamento>,
        verificacao: Box<VerificacaoDoCancelamento>,
    },
}

impl From<CookieIlegivel> for ErroCancelar {
    fn from(erro: CookieIlegivel) -> Self {
        Self::Cookie(erro)
    }
}
