//! Mensagem de validação do framework Infra do SEI: depois de um POST
//! recusado, a página volta com HTTP 200 e o texto num
//! `<textarea id="txaInfraValidacao" style="display:none">`, que um script
//! mostra por `alert(document.getElementById('txaInfraValidacao').value)`.
//! Observado no login recusado da `UnB` 4.0.9 (02/11/2026); ver
//! docs/como-o-sei-funciona/validacao-da-infra.md. Leitura na árvore
//! html5ever (sem regex).
use super::saneamento::sanear;
use super::texto_de_recusa::texto_visivel;

/// `id` do contêiner da mensagem.
pub(crate) const ID_DA_VALIDACAO: &str = "txaInfraValidacao";

/// Texto bruto (entidades decodificadas, ainda não saneado) do primeiro
/// `#txaInfraValidacao` não vazio do documento.
pub(crate) fn texto_da_validacao(dom: &crate::tela_html::Documento) -> Option<String> {
    dom.com_id(ID_DA_VALIDACAO).find_map(|conteiner| {
        let texto = texto_visivel(conteiner);
        (!texto.trim().is_empty()).then_some(texto)
    })
}

/// Mensagem saneada (e limitada) da validação da Infra, se a página a traz.
/// Para os fluxos de POST classificarem uma resposta 200 que é recusa.
pub(crate) fn mensagem_de_validacao_da_infra(
    html: &crate::tela_html::HtmlServido,
) -> Option<String> {
    let dom = crate::tela_html::arvore(html);
    sanear(&texto_da_validacao(&dom)?)
}

#[cfg(test)]
#[path = "tests_validacao_da_infra.rs"]
mod tests_validacao_da_infra;
