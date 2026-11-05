//! Conferência local da oferta de exclusão do Acompanhamento Especial (#254).
//! Nenhum JavaScript é executado. Apenas estrutura observada é aceita.
use super::ErroAcompanhar;
use crate::instalacao::Instalacao;

/// Confere formulário, função e link de exclusão da linha selecionada.
/// Não faz rede nem autoriza POST nesta fatia.
pub(super) fn conferir_oferta_exclusao(
    _instalacao: &Instalacao,
    _html: &str,
    _id_procedimento: &str,
    _id_alvo: &str,
) -> Result<(), ErroAcompanhar> {
    Err(ErroAcompanhar::ExclusaoNaoOferecida)
}

#[cfg(test)]
#[path = "tests_exclusao.rs"]
mod tests;
