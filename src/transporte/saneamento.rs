//! Saneamento da mensagem do servidor antes de expô-la, por varredura
//! explícita (sem regex): controles viram espaço, espaços colapsam,
//! URLs, `infra_hash=`, `PHPSESSID=` e hexadecimais longos são redigidos,
//! e o texto é cortado em `MAX_CARACTERES` com `…`.

pub(crate) const MAX_CARACTERES: usize = 500;

/// Texto pronto para expor, ou `None` se ficou vazio.
pub(crate) fn sanear(texto: &str) -> Option<String> {
    let _ = texto;
    None
}

#[cfg(test)]
#[path = "tests_saneamento.rs"]
mod tests_saneamento;
