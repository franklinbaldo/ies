//! Entidades HTML de atributos e `application/x-www-form-urlencoded`.
use super::CodificacaoDaTela;

/// Valor de atributo com as entidades HTML decodificadas. Entidade que o
/// HTML não conhece fica literal, como no navegador (#676).
pub(crate) fn decodificar_atributo(bruto: &str) -> String {
    html_escape::decode_html_entities(bruto).into_owned()
}

/// Acrescenta `valor` codificado; `None` se um caractere não existir no charset.
pub(super) fn codificar(
    valor: &str,
    codificacao: CodificacaoDaTela,
    saida: &mut String,
) -> Option<()> {
    for caractere in valor.chars() {
        let mut buffer = [0u8; 4];
        let bytes: &[u8] = match codificacao {
            CodificacaoDaTela::Iso88591 => {
                buffer[0] = u8::try_from(u32::from(caractere)).ok()?;
                &buffer[..1]
            }
            CodificacaoDaTela::Utf8 => caractere.encode_utf8(&mut buffer).as_bytes(),
        };
        saida.extend(form_urlencoded::byte_serialize(bytes));
    }
    Some(())
}
