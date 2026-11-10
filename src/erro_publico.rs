//! Fronteira pública dos erros ([contrato](../contracts/cli/erros.md)).
//!
//! Cada erro é declarado no módulo que o levanta e deriva [`Causa`], que
//! gera o `From` das variantes `interna`, o status HTTP do SEI e o fato
//! "sessão encerrada" que decide o relogin. Não há nome em texto: a saída
//! publica a mensagem; classifica-se por `match` nas variantes.
use crate::transporte::StatusHttpRecusado;

pub(crate) use ies_derive::Causa;

/// Implementado por `#[derive(Causa)]`.
pub(crate) trait Causa {
    fn status_http(&self) -> Option<&StatusHttpRecusado>;
    /// A sessão do SEI acabou (`#[causa(sessao_encerrada)]`): decide o
    /// relogin das leituras pelo tipo.
    fn sessao_encerrada(&self) -> bool;
}

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
