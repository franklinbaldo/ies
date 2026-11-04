//! Natureza de um `DOCUMENTO`: o ícone (a7 a a9) e o cancelamento (ícone
//! `documento_cancelado` e dica a6 `Documento Cancelado: <motivo>`), Refs
//! #59, contracts/cli/processo-obter-situacao.md. A dica em si não é
//! guardada: em e-mails traz remetente e destinatários.
use super::Literal;

/// `icone`: radical opaco de `svg/<radical>.svg`; `cancelado`: `None` com
/// só uma das duas marcas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NaturezaDoDocumento {
    pub(crate) icone: Option<String>,
    pub(crate) cancelado: Option<bool>,
    pub(crate) motivo_do_cancelamento: Option<String>,
}

/// Lê a6 a a9 de um `DOCUMENTO` já com a aridade conferida.
pub(super) fn ler_natureza(_a: &[Literal; super::ARGUMENTOS]) -> NaturezaDoDocumento {
    NaturezaDoDocumento {
        icone: None,
        cancelado: None,
        motivo_do_cancelamento: None,
    }
}
