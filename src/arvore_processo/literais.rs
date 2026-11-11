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
/// Na `UnB` 4.0.9 a primeira `PASTA` vem na linha do `.html` do nó anterior:
/// `Nos[0].html = '…';Nos[1] = new infraArvoreNo("PASTA",…);`.
const HTML_ANTES: &str = ".html = ";
const BARRA: &str = ".acoes = ";
const SRC: &str = ".src = ";
const PROTOCOLOS: &str = "['protocolos'] = ";

/// `Ok(None)`: a linha não declara nó (nem contém `new infraArvoreNo(`).
/// `Ok(Some((i, argumentos)))`: a linha inteira é
/// `Nos[i] = new infraArvoreNo(<literais separados por vírgula>);`, ou essa
/// declaração logo depois de `Nos[i-1].html = '<texto>';`.
pub(super) fn ler_declaracao(linha: &str) -> Result<Option<(usize, Vec<Literal>)>, ErroArvore> {
    let linha = linha.strip_suffix('\r').unwrap_or(linha);
    if !linha.contains(CONSTRUTOR) {
        return Ok(None);
    }
    let fora = || ErroArvore::DeclaracaoDeNoForaDaFormaObservada;
    let (indice, resto) = indice_entre(linha, "Nos[").ok_or_else(fora)?;
    if let Some(resto) = resto.strip_prefix(HTML_ANTES) {
        let (Literal::Texto(_), depois) = ler_literal(resto)? else {
            return Err(fora());
        };
        let declaracao = depois.strip_prefix(';').ok_or_else(fora)?;
        let (seguinte, argumentos) = ler_no(declaracao)?;
        if seguinte != indice + 1 {
            return Err(fora());
        }
        return Ok(Some((seguinte, argumentos)));
    }
    ler_no(linha).map(Some)
}

/// `Nos[i] = new infraArvoreNo(…);` exata, sem nada antes nem depois.
fn ler_no(linha: &str) -> Result<(usize, Vec<Literal>), ErroArvore> {
    let fora = || ErroArvore::DeclaracaoDeNoForaDaFormaObservada;
    let (indice, resto) = indice_entre(linha, "Nos[").ok_or_else(fora)?;
    let resto = resto.strip_prefix(ABERTURA).ok_or_else(fora)?;
    let (argumentos, depois) = ler_argumentos(resto)?;
    if depois != ";" {
        return Err(fora());
    }
    Ok((indice, argumentos))
}

/// `<variavel><decimal>]` no começo da linha: o índice e o que sobra.
fn indice_entre<'a>(linha: &'a str, variavel: &str) -> Option<(usize, &'a str)> {
    let resto = linha.strip_prefix(variavel)?;
    let (indice, resto) = resto.split_once(']')?;
    if indice.is_empty() || indice.len() > 6 || !indice.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((indice.parse().ok()?, resto))
}

/// O `i` de uma linha `Nos[i].acoes = …`, sem ler o resto.
pub(super) fn indice_da_barra(linha: &str) -> Option<usize> {
    let (indice, resto) = indice_entre(linha, "Nos[")?;
    resto.starts_with(BARRA).then_some(indice)
}

/// O `i` da linha inteira `Nos[i].acoes = Nos[i].acoes.concat('');`, servida
/// na UFRJ e em GO depois do `.html` de cada documento (só o literal vazio
/// foi observado: com conteúdo, a linha é lida por [`ler_barra`] e recusada).
pub(super) fn concatenacao_vazia_da_barra(linha: &str) -> Option<usize> {
    let linha = linha.strip_suffix('\r').unwrap_or(linha);
    let (indice, resto) = indice_entre(linha, "Nos[")?;
    let resto = resto.strip_prefix(BARRA)?;
    let (mesmo, resto) = indice_entre(resto, "Nos[")?;
    (mesmo == indice && resto == ".acoes.concat('');").then_some(indice)
}

/// `Ok(Some((i, texto)))` para a linha inteira `Nos[i].acoes = '<texto>';`;
/// `Ok(None)` se a linha não atribui `.acoes` a um nó.
pub(super) fn ler_barra(linha: &str) -> Result<Option<(usize, String)>, ErroArvore> {
    let linha = linha.strip_suffix('\r').unwrap_or(linha);
    let Some((indice, resto)) = indice_entre(linha, "Nos[") else {
        return Ok(None);
    };
    let Some(resto) = resto.strip_prefix(BARRA) else {
        return Ok(None);
    };
    let fora = || ErroArvore::BarraDeNoForaDaFormaObservada;
    match ler_literal(resto).map_err(|e| ErroArvore::BarraDeNoIlegivel(Box::new(e)))? {
        (Literal::Texto(texto), ";") => Ok(Some((indice, texto))),
        _ => Err(fora()),
    }
}

/// O `i` de uma linha `Nos[i].src = …`, sem ler o resto.
pub(super) fn indice_do_src(linha: &str) -> Option<usize> {
    let (indice, resto) = indice_entre(linha, "Nos[")?;
    resto.starts_with(SRC).then_some(indice)
}

/// `Ok(Some((i, texto)))` para a linha inteira `Nos[i].src = '<texto>';`
/// (o link do conteúdo do nó, Refs #60); `Ok(None)` se a linha não atribui
/// `.src` a um nó.
pub(super) fn ler_src(linha: &str) -> Result<Option<(usize, String)>, ErroArvore> {
    let linha = linha.strip_suffix('\r').unwrap_or(linha);
    let Some((indice, resto)) = indice_entre(linha, "Nos[") else {
        return Ok(None);
    };
    let Some(resto) = resto.strip_prefix(SRC) else {
        return Ok(None);
    };
    let fora = || ErroArvore::SrcDeNoForaDaFormaObservada;
    match ler_literal(resto).map_err(|e| ErroArvore::SrcDeNoIlegivel(Box::new(e)))? {
        (Literal::Texto(texto), ";") => Ok(Some((indice, texto))),
        _ => Err(fora()),
    }
}

/// O `i` de uma linha `Nos[i].html = …`, sem ler o resto.
pub(super) fn indice_do_html(linha: &str) -> Option<usize> {
    let (indice, resto) = indice_entre(linha, "Nos[")?;
    resto.starts_with(HTML_ANTES).then_some(indice)
}

/// `Ok(Some((i, texto)))` para `Nos[i].html = '<texto>';`, sozinha ou
/// seguida da declaração seguinte (`';Nos[i+1] = new infraArvoreNo(…`, já
/// conferida por [`ler_declaracao`]); `Ok(None)` se a linha não atribui
/// `.html` a um nó.
pub(super) fn ler_html(linha: &str) -> Result<Option<(usize, String)>, ErroArvore> {
    let linha = linha.strip_suffix('\r').unwrap_or(linha);
    let Some((indice, resto)) = indice_entre(linha, "Nos[") else {
        return Ok(None);
    };
    let Some(resto) = resto.strip_prefix(HTML_ANTES) else {
        return Ok(None);
    };
    let fora = || ErroArvore::HtmlDoNoDoProcessoForaDaFormaObservada;
    match ler_literal(resto).map_err(|e| ErroArvore::HtmlDoNoDoProcessoIlegivel(Box::new(e)))? {
        (Literal::Texto(texto), depois) if depois == ";" || depois.starts_with(";Nos[") => {
            Ok(Some((indice, texto)))
        }
        _ => Err(fora()),
    }
}

/// `Ok(Some((k, texto)))` para `Pastas[k]['protocolos'] = '<texto>';`
/// (indentação tolerada); `Ok(None)` para as outras linhas, inclusive
/// `Pastas[k] = [];` e `Pastas[k]['link'] = …`.
pub(super) fn ler_protocolos(linha: &str) -> Result<Option<(usize, String)>, ErroArvore> {
    let linha = linha.trim();
    if !linha.starts_with("Pastas[") || !linha.contains(PROTOCOLOS) {
        return Ok(None);
    }
    let fora = || ErroArvore::ProtocolosDaPastaForaDaFormaObservada;
    let (indice, resto) = indice_entre(linha, "Pastas[").ok_or_else(fora)?;
    let resto = resto.strip_prefix(PROTOCOLOS).ok_or_else(fora)?;
    match ler_literal(resto).map_err(|e| ErroArvore::ProtocolosDaPastaIlegiveis(Box::new(e)))? {
        (Literal::Texto(texto), ";") => Ok(Some((indice, texto))),
        _ => Err(fora()),
    }
}

/// `<variavel>[j] = new <construtor>(<literais>);` (indentação tolerada):
/// os argumentos, ou `None` se a linha não tem essa forma.
pub(super) fn ler_chamada(linha: &str, variavel: &str, construtor: &str) -> Option<Vec<Literal>> {
    let linha = linha.trim();
    let (_, resto) = indice_entre(linha, variavel)?;
    let resto = resto.strip_prefix(" = new ")?.strip_prefix(construtor)?;
    let resto = resto.strip_prefix('(')?;
    match ler_argumentos(resto) {
        Ok((argumentos, ";")) => Some(argumentos),
        _ => None,
    }
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
        if let Some(seguinte) = depois.strip_prefix(',') {
            resto = seguinte;
        } else if let Some(seguinte) = depois.strip_prefix(')') {
            return Ok((argumentos, seguinte));
        } else if depois.is_empty() {
            return Err(ErroArvore::DeclaracaoDeNoForaDaFormaObservada);
        } else {
            return Err(ErroArvore::ArgumentoDeNoNaoLiteral);
        }
    }
}

fn ler_literal(resto: &str) -> Result<(Literal, &str), ErroArvore> {
    for aspa in ['"', '\''] {
        if let Some(texto) = resto.strip_prefix(aspa) {
            return ler_texto(texto, aspa);
        }
    }
    let fim = resto.find([',', ')', ';']).unwrap_or(resto.len());
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
    while let Some((_, caractere)) = caracteres.next() {
        if caractere == aspa {
            return Ok((Literal::Texto(texto), caracteres.as_str()));
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
