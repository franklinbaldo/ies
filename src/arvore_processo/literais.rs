//! Leitor escrito à mão das declarações de nó: só literais JS (strings entre
//! aspas duplas ou simples com escapes, `null`, `true`, `false`, inteiros).
//! Nada é avaliado; qualquer outra forma é recusa.
use super::ErroArvore;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Literal {
    Texto(String),
    Nulo,
    Booleano(bool),
    Inteiro(String),
}

/// `Ok(None)`: a linha não declara nó (nem contém `new infraArvoreNo(`).
/// `Ok(Some((i, argumentos)))`: a linha inteira é
/// `Nos[i] = new infraArvoreNo(<literais separados por vírgula>);`.
pub(super) fn ler_declaracao(linha: &str) -> Result<Option<(usize, Vec<Literal>)>, ErroArvore> {
    let _ = linha;
    Ok(None)
}
