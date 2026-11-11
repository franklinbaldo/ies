//! Saneamento da mensagem do servidor antes de expô-la, por varredura
//! explícita (sem regex): controles viram espaço, espaços colapsam,
//! URLs, `infra_hash=`, `PHPSESSID=` e hexadecimais longos são redigidos,
//! e o texto é cortado em `MAX_CARACTERES` com `…`. Fora isso o texto sai
//! como o SEI o enviou e pode conter informação do servidor.

pub(crate) const MAX_CARACTERES: usize = 500;

/// Menor sequência de dígitos hexadecimais tratada como segredo.
const MIN_HEX: usize = 32;

/// Nomes cujo valor é capacidade de sessão; comparados sem caixa ASCII.
const NOMES_SECRETOS: [&str; 2] = ["infra_hash=", "phpsessid="];

const ESQUEMAS: [&str; 2] = ["https://", "http://"];

/// Texto pronto para expor, ou `None` se ficou vazio.
pub(crate) fn sanear(texto: &str) -> Option<String> {
    let colapsado = colapsar(texto);
    if colapsado.is_empty() {
        return None;
    }
    Some(cortar(&redigir(&colapsado)))
}

/// Controles (inclusive `\t`, `\r`, `\n`) e espaços Unicode viram um espaço.
fn colapsar(texto: &str) -> String {
    let mut saida = String::with_capacity(texto.len());
    let mut espaco_pendente = false;
    for c in texto.chars() {
        if c.is_whitespace() || c.is_control() {
            espaco_pendente = true;
            continue;
        }
        if espaco_pendente && !saida.is_empty() {
            saida.push(' ');
        }
        espaco_pendente = false;
        saida.push(c);
    }
    saida
}

fn comeca_com_sem_caixa(resto: &str, prefixo: &str) -> bool {
    resto
        .as_bytes()
        .get(..prefixo.len())
        .is_some_and(|inicio| inicio.eq_ignore_ascii_case(prefixo.as_bytes()))
}

/// Uma URL termina no primeiro espaço, aspas ou sinal de tag.
fn fim_de_url(c: char) -> bool {
    c == ' ' || matches!(c, '"' | '\'' | '<' | '>')
}

/// Um valor de parâmetro ou cookie termina num separador usual.
fn fim_de_valor(c: char) -> bool {
    fim_de_url(c) || matches!(c, '&' | ';' | ',' | ')')
}

/// Bytes do prefixo de `resto` cujos caracteres não satisfazem `fim`.
fn ate(resto: &str, fim: fn(char) -> bool) -> usize {
    resto.find(fim).unwrap_or(resto.len())
}

fn redigir(texto: &str) -> String {
    let mut saida = String::with_capacity(texto.len());
    let mut resto = texto;
    while let Some(c) = resto.chars().next() {
        if ESQUEMAS.iter().any(|e| comeca_com_sem_caixa(resto, e)) {
            saida.push_str("<url>");
            (_, resto) = resto.split_at(ate(resto, fim_de_url));
            continue;
        }
        if let Some(nome) = NOMES_SECRETOS
            .iter()
            .find(|nome| comeca_com_sem_caixa(resto, nome))
        {
            // O nome sai como veio; só o valor é redigido. O nome é ASCII:
            // sem fronteira de caractere, o resto inteiro é redigido.
            let Some((cru, valor)) = resto.split_at_checked(nome.len()) else {
                saida.push_str("<redigido>");
                break;
            };
            saida.push_str(cru);
            saida.push_str("<redigido>");
            (_, resto) = valor.split_at(ate(valor, fim_de_valor));
            continue;
        }
        if c.is_ascii_hexdigit() {
            let (sequencia, depois) = resto.split_at(ate(resto, |d| !d.is_ascii_hexdigit()));
            if sequencia.len() >= MIN_HEX {
                saida.push_str("<hex>");
            } else {
                saida.push_str(sequencia);
            }
            resto = depois;
            continue;
        }
        saida.push(c);
        (_, resto) = resto.split_at(c.len_utf8());
    }
    saida
}

/// No máximo `MAX_CARACTERES` caracteres, o último sendo `…` se cortou.
fn cortar(texto: &str) -> String {
    if texto.chars().count() <= MAX_CARACTERES {
        return texto.to_owned();
    }
    let mut saida: String = texto.chars().take(MAX_CARACTERES - 1).collect();
    saida.push('…');
    saida
}

#[cfg(test)]
#[path = "tests_saneamento.rs"]
mod tests_saneamento;
