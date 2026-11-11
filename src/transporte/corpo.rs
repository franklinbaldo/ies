//! Corpo HTTP em texto: UTF-8 ou ISO-8859-1, conforme o charset declarado.
//! Usado pelo transporte ao vivo (caixa, login, autocompletar do envio) e,
//! nos testes, pela leitura das cassettes do acervo.

/// Por que o corpo não virou texto.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCorpoHttp {
    /// O corpo passa de `MAX_HTML_BYTES`.
    HtmlAcimaDoLimite,
    /// UTF-8 declarado (ou presumido), bytes inválidos.
    Utf8Invalido(#[source] std::str::Utf8Error),
    /// O charset declarado não é UTF-8 nem ISO-8859-1.
    CodificacaoNaoSuportada,
}

/// Decodifica bytes no `encoding` declarado. Sem codificação declarada
/// (`None`), cada byte vira o caractere de mesmo código, como em ISO-8859-1.
/// Outra codificação é recusada.
pub(crate) fn legivel(bytes: &[u8], encoding: Option<&str>) -> Result<String, ErroCorpoHttp> {
    let Some(enc) = encoding.map(str::to_ascii_lowercase) else {
        return Ok(bytes.iter().copied().map(char::from).collect());
    };
    if enc.contains("utf-8") || enc == "utf8" {
        return std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(ErroCorpoHttp::Utf8Invalido);
    }
    if enc.contains("8859-1") || enc.contains("latin1") {
        return Ok(bytes.iter().copied().map(char::from).collect());
    }
    Err(ErroCorpoHttp::CodificacaoNaoSuportada)
}

/// Corpo de uma resposta HTML/XML: até `MAX_HTML_BYTES`, ISO-8859-1 se o
/// `Content-Type` o declara, senão UTF-8 estrito.
pub(crate) fn decode_http_body(bytes: &[u8], content_type: &str) -> Result<String, ErroCorpoHttp> {
    if bytes.len() > crate::MAX_HTML_BYTES {
        return Err(ErroCorpoHttp::HtmlAcimaDoLimite);
    }
    let enc = if content_type.to_ascii_lowercase().contains("8859-1") {
        "iso-8859-1"
    } else {
        "utf-8"
    };
    legivel(bytes, Some(enc))
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
        assert!(legivel(b"abc", Some("shift_jis")).is_err());
        assert_eq!(legivel(b"\xe9", None).unwrap(), "é");
        assert_eq!(legivel(b"\xe9", Some("latin1")).unwrap(), "é");
    }
}
