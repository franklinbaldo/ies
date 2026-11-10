//! Fronteira pública dos erros ([contrato](../contracts/cli/erros.md)).
//!
//! Cada erro é declarado no módulo que o levanta, com `thiserror`:
//! `#[from]` compõe pelo `?` e `#[source]` liga a causa. Fatos como "sessão
//! encerrada" ou o status HTTP do SEI são decididos por `match` nas
//! variantes de cada tipo, nunca por `downcast` na cadeia `source()`.

/// A sessão do SEI acabou: decide o relogin das leituras pelo tipo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("a sessão do SEI foi encerrada")]
pub struct SessaoEncerrada;

#[cfg(test)]
#[path = "erro_publico/tests.rs"]
mod tests;

/// Rótulo estável de um erro tipado para replays e testes: o `Debug` da variante.
#[cfg(test)]
pub(crate) trait Rotulo {
    fn rotulo(&self) -> &'static str;
}

#[cfg(test)]
impl<T: std::fmt::Debug> Rotulo for T {
    fn rotulo(&self) -> &'static str {
        Box::leak(format!("{self:?}").into_boxed_str())
    }
}

#[cfg(test)]
pub(crate) fn rotulo<T: std::fmt::Debug>(erro: &T) -> &'static str {
    erro.rotulo()
}
