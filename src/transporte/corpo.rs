//! Corpo HTTP em texto: UTF-8 ou ISO-8859-1, conforme o charset declarado.
//! O corpo HTML/XML vira `HtmlServido` em `tela_html`, com o limite.
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

#[cfg(test)]
mod tests {
    use super::legivel;

    #[test]
    fn codificacao_desconhecida_e_recusada() {
        assert!(legivel(b"abc", Some("shift_jis")).is_err());
        assert_eq!(legivel(b"\xe9", None).unwrap(), "é");
        assert_eq!(legivel(b"\xe9", Some("latin1")).unwrap(), "é");
    }
}
