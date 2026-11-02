//! Interpretador explícito de `Location` do login, sem regex e sem
//! normalizar caminhos. Só o host da instalação é aceito. Regra observada
//! (UFRJ e GO, 02/11/2026): `http://` no mesmo host é promovido a
//! `https://`; a URL HTTP nunca é pedida.
use super::ErroLogin;

/// Próxima URL HTTPS de um salto: `localizacao` resolvida contra `atual`
/// (URL HTTPS absoluta já aceita) e conferida contra `host`.
pub(crate) fn resolver_no_host(
    atual: &str,
    localizacao: &str,
    host: &str,
) -> Result<String, ErroLogin> {
    let _ = (atual, localizacao, host);
    todo!("CONTRACT: resolver Location")
}

/// Caminho (com `/` inicial) de uma URL HTTPS absoluta, sem a consulta.
pub(crate) fn caminho_da_url(url: &str) -> Option<&str> {
    let _ = url;
    todo!("CONTRACT: caminho da URL")
}
