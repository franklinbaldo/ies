//! Despacho local das consultas de unidades e caminho do índice (#244).
//! Sem requisição Web quando há snapshot.
use crate::indice_caminho_cli;
use crate::{Cli, Domain, Indice, IndiceAcao};
use std::process::ExitCode;

pub(super) fn tentar_consulta_local(cli: &Cli) -> Option<ExitCode> {
    match &cli.command {
        Domain::Indice(Indice {
            acao: Some(IndiceAcao::Caminho),
        }) => Some(indice_caminho_cli::mostrar(cli.json)),
        _ => None,
    }
}
