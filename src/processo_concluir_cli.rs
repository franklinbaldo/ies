//! `ies processo concluir NUP [--confirmar]` (contracts/cli/processo-concluir.md).
use crate::Cli;
use std::process::ExitCode;

/// `Some` só para `ies processo concluir`.
pub(crate) fn tentar_concluir(cli: &Cli) -> Option<ExitCode> {
    let _ = cli;
    None
}
