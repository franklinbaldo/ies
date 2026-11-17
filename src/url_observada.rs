//! Decomposição sintática de URLs servidas; nenhuma política de operação.
//! Não segue redirects, não normaliza caminhos, não toca rede nem cookies.
use crate::instalacao::host_valido;

/// URL HTTPS absoluta separada em fatias emprestadas da fonte.
pub(crate) struct UrlHttps<'a> {
    pub(crate) host: &'a str,
    /// Caminho sem a primeira barra, para manter a convenção dos leitores.
    pub(crate) caminho: &'a str,
    pub(crate) consulta: Option<&'a str>,
}

impl<'a> UrlHttps<'a> {
    /// Forma exigida pelos formulários: bytes ASCII imprimíveis, sem fragmentos.
    pub(crate) fn interpretar(url: &'a str) -> Option<Self> {
        if !url.bytes().all(|b| (33..=126).contains(&b))
            || url.contains('#')
            || url.as_bytes().contains(&92)
        {
            return None;
        }
        partes_https(url)
    }
}

pub(crate) fn partes_https(url: &str) -> Option<UrlHttps<'_>> {
    let (host, resto) = url.strip_prefix("https://")?.split_once('/')?;
    if !host_valido(host) {
        return None;
    }
    let (caminho, consulta) = match resto.split_once('?') {
        Some((caminho, consulta)) => (caminho, Some(consulta)),
        None => (resto, None),
    };
    Some(UrlHttps {
        host,
        caminho,
        consulta,
    })
}

/// Declara um esquema RFC lexical na primeira componente da referência.
pub(crate) fn tem_esquema(referencia: &str) -> bool {
    primeira_componente(referencia)
        .split_once(':')
        .is_some_and(|(esquema, _)| {
            !esquema.is_empty()
                && esquema
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
        })
}

/// Para login, qualquer prefixo não vazio terminado em ':' era recusado;
/// não mudar essa regra histórica quando o token não é RFC.
pub(crate) fn tem_prefixo_de_esquema(referencia: &str) -> bool {
    primeira_componente(referencia)
        .split_once(':')
        .is_some_and(|(esquema, _)| !esquema.is_empty())
}

/// Até a primeira `/` ou `?` (a referência inteira se não houver).
fn primeira_componente(referencia: &str) -> &str {
    referencia
        .split_once(['/', '?'])
        .map_or(referencia, |(antes, _)| antes)
}

pub(crate) fn esquema_e_resto(referencia: &str) -> Option<(&str, &str)> {
    referencia.split_once("://")
}

pub(crate) fn resto_http_ou_https(referencia: &str) -> Option<&str> {
    let (esquema, resto) = esquema_e_resto(referencia)?;
    (esquema.eq_ignore_ascii_case("https") || esquema.eq_ignore_ascii_case("http")).then_some(resto)
}

/// Caminho com a barra inicial; não remove segmentos nem resolve '..'.
pub(crate) fn caminho_absoluto(url: &str) -> Option<&str> {
    let resto = url.strip_prefix("https://")?;
    let (_, caminho) = resto.split_at(resto.find('/')?);
    Some(caminho.split_once('?').map_or(caminho, |(antes, _)| antes))
}

/// Diretório sintático da URL servida, sem normalizar a referência.
pub(crate) fn pasta(caminho: &str) -> Option<&str> {
    caminho.rsplit_once('/').map(|(pasta, _)| pasta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompostos_sem_normalizar_referencia() {
        let p = partes_https("https://exemplo.test/sei/controlador.php?acao=x").unwrap();
        assert_eq!(
            (p.host, p.caminho, p.consulta),
            ("exemplo.test", "sei/controlador.php", Some("acao=x"))
        );
        assert_eq!(pasta(p.caminho), Some("sei"));
        assert_eq!(
            caminho_absoluto("https://exemplo.test/sei/a?b=c"),
            Some("/sei/a")
        );
        assert!(partes_https("https://user@exemplo.test/sei/a").is_none());
        assert!(partes_https("https://exemplo.test:444/sei/a").is_none());
    }

    #[test]
    fn esquema_e_referencias_continuam_separados() {
        assert_eq!(
            resto_http_ou_https("HTTP://EXEMPLO.TEST/sei"),
            Some("EXEMPLO.TEST/sei")
        );
        assert!(tem_esquema("mailto:alguem"));
        assert!(tem_prefixo_de_esquema("a!:b"));
        assert!(!tem_esquema("a!:b"));
        assert!(!tem_esquema("//outro.test/sei"));
        assert!(resto_http_ou_https("//outro.test").is_none());
        assert_eq!(pasta("sei/../documento"), Some("sei/.."));
        assert_eq!(pasta("sei/a%2Fb"), Some("sei"));
        assert_eq!(
            caminho_absoluto("https://exemplo.test/sei/a#frag"),
            Some("/sei/a#frag")
        );
    }
}
