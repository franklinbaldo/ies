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

const ABERTURA: &str = " = new infraArvoreNo(";
const CONSTRUTOR: &str = "new infraArvoreNo(";

/// `Ok(None)`: a linha não declara nó (nem contém `new infraArvoreNo(`).
/// `Ok(Some((i, argumentos)))`: a linha inteira é
/// `Nos[i] = new infraArvoreNo(<literais separados por vírgula>);`.
pub(super) fn ler_declaracao(linha: &str) -> Result<Option<(usize, Vec<Literal>)>, ErroArvore> {
    let linha = linha.strip_suffix('\r').unwrap_or(linha);
    if !linha.contains(CONSTRUTOR) {
        return Ok(None);
    }
    let fora = ErroArvore::DeclaracaoDeNoForaDaFormaObservada;
    let resto = linha.strip_prefix("Nos[").ok_or(fora)?;
    let (indice, resto) = resto.split_once(']').ok_or(fora)?;
    if indice.is_empty() || indice.len() > 6 || !indice.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fora);
    }
    let indice: usize = indice.parse().map_err(|_| fora)?;
    let resto = resto.strip_prefix(ABERTURA).ok_or(fora)?;
    let (argumentos, depois) = ler_argumentos(resto)?;
    if depois != ";" {
        return Err(fora);
    }
    Ok(Some((indice, argumentos)))
}

/// Literais separados por `,` (sem espaço, como servidos) até o `)`.
/// Devolve o que sobra depois do `)`.
fn ler_argumentos(mut resto: &str) -> Result<(Vec<Literal>, &str), ErroArvore> {
    let mut argumentos = Vec::new();
    if let Some(depois) = resto.strip_prefix(')') {
        return Ok((argumentos, depois));
    }
    loop {
        let (literal, depois) = ler_literal(resto)?;
        argumentos.push(literal);
        match depois.as_bytes().first() {
            Some(b',') => resto = &depois[1..],
            Some(b')') => return Ok((argumentos, &depois[1..])),
            None => return Err(ErroArvore::DeclaracaoDeNoForaDaFormaObservada),
            Some(_) => return Err(ErroArvore::ArgumentoDeNoNaoLiteral),
        }
    }
}

fn ler_literal(resto: &str) -> Result<(Literal, &str), ErroArvore> {
    if let Some(&aspa @ (b'"' | b'\'')) = resto.as_bytes().first() {
        return ler_texto(&resto[1..], char::from(aspa));
    }
    let fim = resto.find([',', ')']).unwrap_or(resto.len());
    let (palavra, depois) = resto.split_at(fim);
    let literal = match palavra {
        "null" => Literal::Nulo,
        "true" => Literal::Booleano(true),
        "false" => Literal::Booleano(false),
        _ if inteiro(palavra) => Literal::Inteiro(palavra.to_owned()),
        _ => return Err(ErroArvore::ArgumentoDeNoNaoLiteral),
    };
    Ok((literal, depois))
}

fn inteiro(palavra: &str) -> bool {
    let digitos = palavra.strip_prefix('-').unwrap_or(palavra);
    !digitos.is_empty() && digitos.len() <= 20 && digitos.bytes().all(|b| b.is_ascii_digit())
}

/// Texto até a aspa de fechamento, com os escapes observáveis num literal
/// simples. `\x`, `\u`, octais e continuação de linha são recusa.
fn ler_texto(resto: &str, aspa: char) -> Result<(Literal, &str), ErroArvore> {
    let mut texto = String::new();
    let mut caracteres = resto.char_indices();
    while let Some((posicao, caractere)) = caracteres.next() {
        if caractere == aspa {
            return Ok((Literal::Texto(texto), &resto[posicao + 1..]));
        }
        if caractere != '\\' {
            texto.push(caractere);
            continue;
        }
        let escapado = match caracteres.next() {
            None => return Err(ErroArvore::TextoDeNoSemFechamento),
            Some((_, c @ ('"' | '\'' | '\\' | '/'))) => c,
            Some((_, 'n')) => '\n',
            Some((_, 'r')) => '\r',
            Some((_, 't')) => '\t',
            Some(_) => return Err(ErroArvore::EscapeDeNoNaoReconhecido),
        };
        texto.push(escapado);
    }
    Err(ErroArvore::TextoDeNoSemFechamento)
}
