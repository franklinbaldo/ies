//! Shim do RED: nenhuma leitura ainda.
use super::Registro;

pub(super) const TERMO_DE_PROVA: &str = "0";

pub(super) fn replay(_card: &str, _instalacao: &str, _json: &serde_json::Value) -> Vec<Registro> {
    Vec::new()
}
