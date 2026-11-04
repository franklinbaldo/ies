//! Entidades de caractere no **texto** de um documento HTML (Refs #60): as
//! cinco de marcação, as numéricas e as nomeadas do Latin-1 (HTML 4, U+00A0
//! a U+00FF), as que o editor do SEI serve no corpo (`&aacute;`, `&nbsp;`,
//! `&ordm;`, observadas ao vivo na `UnB`), e algumas tipográficas. Uma
//! entidade desconhecida fica como está. Sem regex.

/// Nomes de U+00A0 a U+00FF, na ordem do código.
const LATIN1: [&str; 96] = [
    "nbsp", "iexcl", "cent", "pound", "curren", "yen", "brvbar", "sect", "uml", "copy", "ordf",
    "laquo", "not", "shy", "reg", "macr", "deg", "plusmn", "sup2", "sup3", "acute", "micro",
    "para", "middot", "cedil", "sup1", "ordm", "raquo", "frac14", "frac12", "frac34", "iquest",
    "Agrave", "Aacute", "Acirc", "Atilde", "Auml", "Aring", "AElig", "Ccedil", "Egrave", "Eacute",
    "Ecirc", "Euml", "Igrave", "Iacute", "Icirc", "Iuml", "ETH", "Ntilde", "Ograve", "Oacute",
    "Ocirc", "Otilde", "Ouml", "times", "Oslash", "Ugrave", "Uacute", "Ucirc", "Uuml", "Yacute",
    "THORN", "szlig", "agrave", "aacute", "acirc", "atilde", "auml", "aring", "aelig", "ccedil",
    "egrave", "eacute", "ecirc", "euml", "igrave", "iacute", "icirc", "iuml", "eth", "ntilde",
    "ograve", "oacute", "ocirc", "otilde", "ouml", "divide", "oslash", "ugrave", "uacute", "ucirc",
    "uuml", "yacute", "thorn", "yuml",
];

const OUTRAS: [(&str, char); 15] = [
    ("amp", '&'),
    ("lt", '<'),
    ("gt", '>'),
    ("quot", '"'),
    ("apos", '\''),
    ("ndash", '\u{2013}'),
    ("mdash", '\u{2014}'),
    ("lsquo", '\u{2018}'),
    ("rsquo", '\u{2019}'),
    ("ldquo", '\u{201C}'),
    ("rdquo", '\u{201D}'),
    ("bull", '\u{2022}'),
    ("hellip", '\u{2026}'),
    ("euro", '\u{20AC}'),
    ("trade", '\u{2122}'),
];

/// O texto com as entidades conhecidas trocadas pelo caractere.
#[must_use]
pub fn decodificar_texto(bruto: &str) -> String {
    let mut saida = String::with_capacity(bruto.len());
    let mut resto = bruto;
    while let Some(pos) = resto.find('&') {
        saida.push_str(&resto[..pos]);
        let depois = &resto[pos + 1..];
        let fim = depois.find(|c: char| !(c.is_ascii_alphanumeric() || c == '#'));
        let decodificado = match fim {
            Some(n) if n > 0 && n <= 10 && depois[n..].starts_with(';') => {
                entidade(&depois[..n]).map(|c| (c, n))
            }
            _ => None,
        };
        if let Some((caractere, n)) = decodificado {
            saida.push(caractere);
            resto = &depois[n + 1..];
        } else {
            saida.push('&');
            resto = depois;
        }
    }
    saida.push_str(resto);
    saida
}

fn entidade(nome: &str) -> Option<char> {
    if let Some(numero) = nome.strip_prefix('#') {
        let codigo = match numero.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => numero.parse().ok()?,
        };
        return char::from_u32(codigo).filter(|c| !c.is_control() || c.is_whitespace());
    }
    if let Some(i) = LATIN1.iter().position(|n| *n == nome) {
        return char::from_u32(0xA0 + u32::try_from(i).ok()?);
    }
    OUTRAS.iter().find(|(n, _)| *n == nome).map(|(_, c)| *c)
}

#[cfg(test)]
mod tests {
    use super::decodificar_texto;

    #[test]
    fn latin1_numericas_e_desconhecidas() {
        assert_eq!(
            decodificar_texto("&Aacute;&yuml;&nbsp;&#233;&#xE7;"),
            "Áÿ\u{a0}éç"
        );
        assert_eq!(decodificar_texto("a & b &x; &#0; &;"), "a & b &x; &#0; &;");
        assert_eq!(
            decodificar_texto("&ldquo;ok&rdquo; &amp;amp;"),
            "\u{201C}ok\u{201D} &amp;"
        );
    }
}
