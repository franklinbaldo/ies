//! Despacho local das consultas de unidades e caminho do índice (#244).
//! Sem requisição Web quando há snapshot.
use crate::{Cli, Domain, Indice, IndiceAcao, Unidade, UnidadeAcao};
use crate::{indice_caminho_cli, unidade_pesquisar_cli};
use std::process::ExitCode;

pub(super) fn tentar_consulta_local(cli: &Cli) -> Option<ExitCode> {
    match &cli.command {
        Domain::Indice(Indice {
            acao: Some(IndiceAcao::Caminho),
        }) => Some(indice_caminho_cli::mostrar(cli.json)),
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Listar { atualizar }),
        }) => Some(unidade_pesquisar_cli::listar(
            cli.json,
            cli.unidade.as_deref(),
            *atualizar,
        )),
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Pesquisar { termo }),
        }) => Some(unidade_pesquisar_cli::pesquisar(
            cli.json,
            cli.unidade.as_deref(),
            termo,
        )),
        _ => None,
    }
}
