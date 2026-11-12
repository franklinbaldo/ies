//! O HTML servido interpretado num ponto só (#751). Nenhum outro módulo de
//! produção chama o parser: trocá-lo (#676) mexe só aqui.

use crate::transporte::corpo::{ErroCorpoHttp, legivel};

mod documento;
pub(crate) use documento::{Documento, Elemento, arvore};

/// Bytes UTF-8 que um corpo de até `MAX_HTML_BYTES` bytes rende: em
/// ISO-8859-1 cada byte vira um caractere de até dois bytes.
const LIMITE_DO_TEXTO: usize = 2 * crate::MAX_HTML_BYTES;

/// Texto de uma tela servida, com no máximo `LIMITE_DO_TEXTO` bytes, abaixo
/// de `u32::MAX`: o único erro do parser (`InvalidLength`) não ocorre.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HtmlServido(String);

/// O texto passa de `LIMITE_DO_TEXTO` bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct HtmlAcimaDoLimite;

impl HtmlServido {
    /// Texto que não chegou como corpo HTTP (trecho decodificado da tela,
    /// arquivo, corpo de recusa ou de anexo).
    pub(crate) fn novo(html: String) -> Result<Self, HtmlAcimaDoLimite> {
        if html.len() > LIMITE_DO_TEXTO {
            return Err(HtmlAcimaDoLimite);
        }
        Ok(Self(html))
    }

    /// Corpo de uma resposta HTML/XML: até `MAX_HTML_BYTES` bytes,
    /// ISO-8859-1 se o `Content-Type` o declara, senão UTF-8 estrito.
    pub(crate) fn do_corpo(bytes: &[u8], content_type: &str) -> Result<Self, ErroCorpoHttp> {
        if bytes.len() > crate::MAX_HTML_BYTES {
            return Err(ErroCorpoHttp::HtmlAcimaDoLimite);
        }
        let enc = if content_type.to_ascii_lowercase().contains("8859-1") {
            "iso-8859-1"
        } else {
            "utf-8"
        };
        legivel(bytes, Some(enc)).map(Self)
    }

    /// O texto, para quem lê a tela sem o parser.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// Literal de teste, que cabe no limite.
    #[cfg(test)]
    pub(crate) fn de_teste(html: &str) -> Self {
        Self::novo(html.to_owned()).unwrap()
    }
}

impl std::ops::Deref for HtmlServido {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

/// A árvore do HTML servido.
pub(crate) fn interpretar(html: &HtmlServido) -> tl::VDom<'_> {
    match tl::parse(html, tl::ParserOptions::default()) {
        Ok(dom) => dom,
        Err(tl::ParseError::InvalidLength) => {
            unreachable!("HtmlServido tem no máximo 2 × MAX_HTML_BYTES, abaixo de u32::MAX")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HtmlAcimaDoLimite, HtmlServido, LIMITE_DO_TEXTO};
    use crate::transporte::corpo::ErroCorpoHttp;

    #[test]
    fn limite_do_texto_cabe_no_parser() {
        assert!(u32::try_from(LIMITE_DO_TEXTO).is_ok());
    }

    #[test]
    fn novo_aceita_no_limite_e_recusa_acima() {
        let limite = "a".repeat(LIMITE_DO_TEXTO);
        assert_eq!(HtmlServido::novo(limite).unwrap().len(), LIMITE_DO_TEXTO);
        let acima = "a".repeat(LIMITE_DO_TEXTO + 1);
        assert_eq!(HtmlServido::novo(acima), Err(HtmlAcimaDoLimite));
    }

    #[test]
    fn iso_8859_1_declarado_mapeia_cada_byte() {
        let corpo = b"a\xe7\xe3o";
        let html = HtmlServido::do_corpo(corpo, "text/html; charset=ISO-8859-1").unwrap();
        assert_eq!(html.as_str(), "ação");
    }

    #[test]
    fn sem_charset_latino_exige_utf8_valido() {
        let html = HtmlServido::do_corpo("ação".as_bytes(), "text/html").unwrap();
        assert_eq!(html.as_str(), "ação");
        assert!(matches!(
            HtmlServido::do_corpo(b"a\xe7\xe3o", "text/html; charset=utf-8"),
            Err(ErroCorpoHttp::Utf8Invalido(_))
        ));
        assert!(matches!(
            HtmlServido::do_corpo(b"a\xe7\xe3o", ""),
            Err(ErroCorpoHttp::Utf8Invalido(_))
        ));
    }

    #[test]
    fn corpo_acima_do_limite_e_recusado_antes_de_decodificar() {
        let grande = vec![0xFF; crate::MAX_HTML_BYTES + 1];
        assert!(matches!(
            HtmlServido::do_corpo(&grande, "text/html; charset=utf-8"),
            Err(ErroCorpoHttp::HtmlAcimaDoLimite)
        ));
        let limite = vec![b'a'; crate::MAX_HTML_BYTES];
        assert!(HtmlServido::do_corpo(&limite, "text/html; charset=utf-8").is_ok());
    }

    #[test]
    fn corpo_iso_8859_1_no_limite_rende_ate_o_dobro() {
        let corpo = vec![0xE7; crate::MAX_HTML_BYTES];
        let texto = HtmlServido::do_corpo(&corpo, "text/html; charset=ISO-8859-1").unwrap();
        assert_eq!(texto.len(), LIMITE_DO_TEXTO);
    }
}
