//! Mensagem de validação do framework Infra do SEI: depois de um POST
//! recusado, a página volta com HTTP 200 e o texto num
//! `<textarea id="txaInfraValidacao" style="display:none">`, que um script
//! mostra por `alert(document.getElementById('txaInfraValidacao').value)`.
//! Observado no login recusado da UnB 4.0.9 (02/11/2026); ver
//! docs/como-o-sei-funciona/validacao-da-infra.md. Leitura com `tl` e
//! `html_atributos` (sem regex).

/// `id` do contêiner da mensagem.
pub(crate) const ID_DA_VALIDACAO: &str = "txaInfraValidacao";

/// Texto bruto (entidades decodificadas, ainda não saneado) do primeiro
/// `#txaInfraValidacao` não vazio do documento.
pub(crate) fn texto_da_validacao(_dom: &tl::VDom<'_>) -> Option<String> {
    None
}

/// Mensagem saneada (e limitada) da validação da Infra, se a página a traz.
/// Para os fluxos de POST classificarem uma resposta 200 que é recusa.
pub(crate) fn mensagem_de_validacao_da_infra(_html: &str) -> Option<String> {
    None
}

#[cfg(test)]
#[path = "tests_validacao_da_infra.rs"]
mod tests_validacao_da_infra;
