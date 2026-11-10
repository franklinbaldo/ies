//! Fronteira pública dos erros ([contrato](../contracts/cli/erros.md)).
//!
//! Cada erro é declarado no módulo que o levanta, com `thiserror`:
//! `#[from]` compõe pelo `?` e `#[source]` liga a causa. O status HTTP do
//! SEI e o fato "sessão encerrada" são achados percorrendo a cadeia
//! `source()`, sem derive próprio. A saída publica a mensagem; classifica-se
//! por `match` nas variantes.
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};
use std::error::Error;

/// A sessão do SEI acabou: decide o relogin das leituras pelo tipo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("a sessão do SEI foi encerrada")]
pub struct SessaoEncerrada;

/// O que se sabe de qualquer erro pela cadeia `source()`.
pub(crate) trait Causa {
    fn status_http(&self) -> Option<&StatusHttpRecusado>;
    fn sessao_encerrada(&self) -> bool;
}

impl<T: Error + 'static> Causa for T {
    fn status_http(&self) -> Option<&StatusHttpRecusado> {
        cadeia(self).find_map(|e| e.downcast_ref::<StatusHttpRecusado>())
    }

    fn sessao_encerrada(&self) -> bool {
        cadeia(self).any(|e| {
            e.is::<SessaoEncerrada>()
                || matches!(
                    e.downcast_ref::<FalhaTransporte>(),
                    Some(
                        FalhaTransporte::PaginaDeLoginRecebidaNoControle
                            | FalhaTransporte::ControleRedirecionadoParaOLogin(_)
                    )
                )
        })
    }
}

fn cadeia<'a>(erro: &'a (dyn Error + 'static)) -> impl Iterator<Item = &'a (dyn Error + 'static)> {
    std::iter::successors(Some(erro), |&e| e.source())
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
