//! A linha do Controle cujo link do processo tem o texto **igual** ao NUP
//! (visões R e D: `tr#P{id}` com `a[href^=controlador.php?acao=procedimento_trabalhar]`).
use super::ErroProcessoObter;

/// `Ok(None)`: o NUP não está nesta página. `Ok(Some((id, href)))`: o id da
/// linha e o `href` com entidades decodificadas, ainda relativo.
pub(super) fn link_do_processo_no_controle(
    html: &str,
    nup: &str,
) -> Result<Option<(String, String)>, ErroProcessoObter> {
    let _ = (html, nup);
    Ok(None)
}
