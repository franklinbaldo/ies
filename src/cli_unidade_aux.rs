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

