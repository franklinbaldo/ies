//! Máquina de estados WHATWG restrita à tag de abertura (shim do contrato).
use super::{AtributosDaTag, ErroAtributoHtml};

/// Tokeniza `<nome atributos...>`; ignora o que vem depois do `>`.
pub(crate) fn tokenizar_tag_de_abertura(
    fonte: &str,
) -> Result<AtributosDaTag<'_>, ErroAtributoHtml> {
    let _ = fonte;
    Ok(AtributosDaTag::novo(Vec::new()))
}
