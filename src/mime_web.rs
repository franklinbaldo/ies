//! Conferência estrita do tipo de mídia do Controle Web do SEI.
//! Nunca autoriza rede, apenas interpreta o cabeçalho recebido.
pub(crate) fn e_html(mime: &str) -> bool {
    principal(mime).eq_ignore_ascii_case("text/html")
}

/// O tipo principal: antes do primeiro `;` (o cabeçalho inteiro se não
/// houver), aparado.
pub(crate) fn principal(mime: &str) -> &str {
    mime.split_once(';').map_or(mime, |(tipo, _)| tipo).trim()
}

#[cfg(test)]
mod tests {
    use super::{e_html, principal};

    #[test]
    fn principal_e_o_texto_antes_do_primeiro_ponto_e_virgula_aparado() {
        for (mime, esperado) in [
            ("", ""),
            (";", ""),
            ("text/html", "text/html"),
            (" TEXT/HTML ; charset=x", "TEXT/HTML"),
            ("a;b;c", "a"),
            ("  ", ""),
        ] {
            assert_eq!(principal(mime), esperado, "{mime:?}");
        }
    }

    #[test]
    fn aceita_tipo_html_exato_com_charset_opcional() {
        for mime in [
            "text/html",
            "text/html; charset=UTF-8",
            " TEXT/HTML ; charset=ISO-8859-1",
        ] {
            assert!(e_html(mime), "{mime}");
        }
    }

    #[test]
    fn substring_em_parametros_de_outro_tipo_nao_autoriza_html() {
        for mime in [
            "application/json; profile=text/html",
            "text/plain; note=text/html",
            "application/octet-stream; name=text/html",
        ] {
            assert!(!e_html(mime), "{mime}");
        }
    }

    #[test]
    fn vazio_ou_subtipo_distinto_continua_sem_suporte() {
        for mime in ["", "text/html2", "application/xhtml+xml", "text/plain"] {
            assert!(!e_html(mime), "{mime}");
        }
    }
}
