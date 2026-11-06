//! Conferência estrita do tipo de mídia do Controle Web do SEI.
//! Nunca autoriza rede, apenas interpreta o cabeçalho recebido.
pub(crate) fn e_html(mime: &str) -> bool {
    mime.split(';')
        .next()
        .is_some_and(|principal| principal.trim().eq_ignore_ascii_case("text/html"))
}

#[cfg(test)]
mod tests {
    use super::e_html;

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
