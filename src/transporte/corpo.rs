//! Corpo HTTP em texto: UTF-8 ou ISO-8859-1, conforme o charset declarado.
//! Usado pelo transporte ao vivo (caixa, login, autocompletar do envio) e,
//! nos testes, pela leitura das cassettes do acervo.

/// Decodifica bytes no `encoding` declarado; ISO-8859-1 (ou nenhum) mapeia
/// cada byte ao caractere de mesmo código. Outra codificação é recusada.
pub(crate) fn legivel(bytes: &[u8], encoding: &str) -> Result<String, String> {
    let enc = encoding.to_ascii_lowercase();
    if enc.contains("utf-8") || enc == "utf8" {
        return String::from_utf8(bytes.to_vec())
            .map_err(|_| "resposta não é UTF-8 válido".to_owned());
    }
    if enc.contains("8859-1") || enc.contains("latin1") || enc.is_empty() {
        return Ok(bytes.iter().copied().map(char::from).collect());
    }
    Err("codificação de resposta ainda não suportada".into())
}

/// Corpo de uma resposta HTML/XML: até `MAX_HTML_BYTES`, ISO-8859-1 se o
/// `Content-Type` o declara, senão UTF-8 estrito.
pub(crate) fn decode_http_body(bytes: &[u8], content_type: &str) -> Result<String, String> {
    if bytes.len() > crate::MAX_HTML_BYTES {
        return Err("HTML maior que 8 MB".into());
    }
    let enc = if content_type.to_ascii_lowercase().contains("8859-1") {
        "iso-8859-1"
    } else {
        "utf-8"
    };
    legivel(bytes, enc)
}

#[cfg(test)]
mod tests {
    use super::{decode_http_body, legivel};

    #[test]
    fn iso_8859_1_declarado_mapeia_cada_byte() {
        let corpo = b"a\xe7\xe3o";
        assert_eq!(
            decode_http_body(corpo, "text/html; charset=ISO-8859-1").unwrap(),
            "ação"
        );
    }

    #[test]
    fn sem_charset_latino_exige_utf8_valido() {
        assert_eq!(
            decode_http_body("ação".as_bytes(), "text/html").unwrap(),
            "ação"
        );
        assert!(decode_http_body(b"a\xe7\xe3o", "text/html; charset=utf-8").is_err());
        assert!(decode_http_body(b"a\xe7\xe3o", "").is_err());
    }

    #[test]
    fn corpo_acima_do_limite_e_recusado_antes_de_decodificar() {
        let grande = vec![b'a'; crate::MAX_HTML_BYTES + 1];
        assert!(decode_http_body(&grande, "text/html; charset=utf-8").is_err());
        let limite = vec![b'a'; crate::MAX_HTML_BYTES];
        assert!(decode_http_body(&limite, "text/html; charset=utf-8").is_ok());
    }

    #[test]
    fn codificacao_desconhecida_e_recusada() {
        assert!(legivel(b"abc", "shift_jis").is_err());
        assert_eq!(legivel(b"\xe9", "").unwrap(), "é");
        assert_eq!(legivel(b"\xe9", "latin1").unwrap(), "é");
    }
}
