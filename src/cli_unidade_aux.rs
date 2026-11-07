//! Adaptadores da CLI para listas de unidades.
//! Extraídos de main.rs para manter o limite de 20 KB após integrações
//! concorrentes; a capacidade mantém seu próprio fluxo de domínio.
use crate::{Cli, ListaDaUnidade};
use std::process::ExitCode;

/// `ies unidade recebidos|gerados`: uma lista por categoria.
pub(crate) fn listar(
    cli: &Cli,
    lista: &ListaDaUnidade,
    categoria: crate::unidade_categoria::CategoriaLista,
) -> ExitCode {
    crate::unidade_categoria::listar_categoria(
        cli.unidade.as_deref(),
        cli.json,
        lista.saida.as_deref(),
        categoria,
        lista.pagina.map(|n| n as usize),
    )
}

/// Exibe colunas ou encaminha sua definição, preservando o fluxo original.
pub(crate) fn colunas(cli: &Cli, definir: Option<&str>, confirmar: bool) -> ExitCode {
    match definir {
        None => crate::unidade_colunas_cli::mostrar_colunas(cli.unidade.as_deref(), cli.json),
        Some(definir) => crate::unidade_colunas_definir_cli::definir_colunas(
            cli.unidade.as_deref(),
            cli.json,
            definir,
            confirmar,
        ),
    }
}
