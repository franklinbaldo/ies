//! Despacho local das consultas de unidades e caminho do índice (#244).
//! Sem requisição Web quando há snapshot.
use crate::indice_caminho_cli;
use crate::{Cli, Domain, Indice, IndiceAcao};

pub(super) fn tentar_consulta_local(cli: &Cli) -> Option<anyhow::Result<()>> {
    // Despacho: só `ies indice caminho`; qualquer outro comando é de outro.
    let Domain::Indice(Indice {
        acao: Some(IndiceAcao::Caminho),
    }) = &cli.command
    else {
        return None;
    };
    Some(indice_caminho_cli::mostrar(cli.json))
}
