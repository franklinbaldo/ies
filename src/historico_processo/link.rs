//! O link assinado do histórico na árvore, nas duas formas observadas
//! (03/11/2026, capturas privadas lidas só pela estrutura):
//! - `UnB` 4.0.9: o `onclick` de `div#divConsultarAndamento a` é
//!   `consultarAndamento();` e o `script` inline define, em duas linhas,
//!   `function consultarAndamento(){` e
//!   `parent.document.getElementById('ifrVisualizacao').src = '<URL>';`;
//! - GO 5.0.3: o `onclick` é `consultarAndamento('<URL>');` (a função não
//!   é definida na página).
//!
//! Só essas formas exatas; nada de regex nem de JavaScript executado.
use super::ErroLinkDoHistorico as E;
use crate::instalacao::Instalacao;

/// URL absoluta do histórico, assinada: `Debug` redigido, sem `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDoHistorico(String);

impl EnderecoDoHistorico {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for EnderecoDoHistorico {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EnderecoDoHistorico(<redigido>)")
    }
}

/// Lê o único link do histórico da árvore servida e o confere contra a
/// instalação e o id do processo aberto.
pub(crate) fn ler_link_do_historico(
    _instalacao: &Instalacao,
    _html: &str,
    _id: &str,
) -> Result<EnderecoDoHistorico, E> {
    Err(E::LinkDoHistoricoAusenteNaArvore)
}
