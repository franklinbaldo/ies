//! Despacho local das consultas de unidades e caminho do índice (#244).
//! Sem requisição Web quando há snapshot.
use crate::indice_caminho_cli;
use crate::{Cli, Domain, Indice, IndiceAcao};

pub(super) fn tentar_consulta_local(cli: &Cli) -> Option<anyhow::Result<()>> {
    match &cli.command {
        Domain::Indice(Indice {
            acao: Some(IndiceAcao::Caminho),
        }) => Some(indice_caminho_cli::mostrar(cli.json)),
        Domain::Processo(_)
        | Domain::Documento(_)
        | Domain::Unidade(_)
        | Domain::Indice(_)
        | Domain::Bloco(_)
        | Domain::Status(_)
        | Domain::Setup(_)
        | Domain::Login(_)
        | Domain::Esquema => None,
    }
}
