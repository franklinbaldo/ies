//! Entidades HTML de atributos e `application/x-www-form-urlencoded`.
use super::CodificacaoDaTela;

/// Valor de atributo com as entidades HTML decodificadas; `None` se houver
/// entidade que o HTML não conhece (não reenviar `&foo;` literal ao SEI).
pub(crate) fn decodificar_atributo(bruto: &str) -> Option<String> {
    let desconhecida = bruto.match_indices('&').any(|(i, _)| {
        let (_, resto) = bruto.split_at(i);
        let Some((antes, _)) = resto.split_once(';') else {
            return false;
        };
        let Some(nome) = antes.strip_prefix('&') else {
            return false;
        };
        let entidade = format!("{antes};");
        !nome.is_empty()
            && nome.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'#')
            && html_escape::decode_html_entities(&entidade) == entidade.as_str()
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
