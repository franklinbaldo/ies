//! Conferência da entrada antes de gravar, e a URL assinada redigida.
use std::fmt;

/// Entrada recusada antes de tocar no banco. Sem dados: nunca carrega a URL,
/// o hash, a geração ou o NUP. O chamador pode ignorá-la e seguir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RegistroRecusado {
    NupInvalido,
    IdProcedimentoInvalido,
    GeracaoInvalida,
    UnidadeInvalida,
    AcaoInvalida,
    ObjetoInvalido,
    LinkNaoHttps,
    LinkComCaractereProibido,
    LinkForaDaInstalacao,
    LinkComConsultaNaoCanonica,
    LinkSemAcaoUnicaIgual,
    LinkSemInfraHashUnico,
    LinkComAcaoAjax,
    LinkDeOutraUnidade,
}

/// URL assinada como a tela publicou. `Debug` não a mostra; não há `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct UrlAssinada(String);

impl UrlAssinada {
    pub(crate) fn url(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for UrlAssinada {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UrlAssinada(<redigida>)")
    }
}
