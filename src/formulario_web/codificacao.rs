//! Entidades HTML de atributos e `application/x-www-form-urlencoded`.
use super::CodificacaoDaTela;

/// Valor de atributo com as entidades HTML decodificadas; `None` se houver
/// entidade que o HTML não conhece (não reenviar `&foo;` literal ao SEI).
pub(crate) fn decodificar_atributo(bruto: &str) -> Option<String> {
    let desconhecida = bruto.match_indices('&').any(|(i, _)| {
        let resto = &bruto[i..];
        resto
            .find(';')
            .filter(|&fim| {
                fim > 1
                    && resto[1..fim]
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'#')
            })
            .is_some_and(|fim| html_escape::decode_html_entities(&resto[..=fim]) == resto[..=fim])
    });
    (!desconhecida).then(|| html_escape::decode_html_entities(bruto).into_owned())
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
