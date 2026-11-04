//! Shim do RED: relatório vazio.
use super::Registro;

pub(super) fn markdown(_cassettes: usize, _registros: &[Registro]) -> String {
    String::new()
}
