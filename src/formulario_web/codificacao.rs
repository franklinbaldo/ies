//! Entidades HTML de atributos e `application/x-www-form-urlencoded`.
use super::CodificacaoDaTela;
use std::fmt::Write as _;

/// Decodifica entidades conhecidas; `None` para entidade terminada em `;`
/// que o IES não reconhece (o navegador poderia produzir outro valor).
/// `&` sem `;` subsequente é literal, como `&acao_origem=` no `action`.
pub(super) fn decodificar_atributo(bruto: &str) -> Option<String> {
    let mut saida = String::with_capacity(bruto.len());
    let mut resto = bruto;
    while let Some(pos) = resto.find('&') {
        saida.push_str(&resto[..pos]);
        let depois = &resto[pos + 1..];
        let fim = depois.find(|c: char| !(c.is_ascii_alphanumeric() || c == '#'));
        match fim {
            Some(n) if n > 0 && depois[n..].starts_with(';') => {
                saida.push(entidade(&depois[..n])?);
                resto = &depois[n + 1..];
            }
            _ => {
                saida.push('&');
                resto = depois;
            }
        }
    }
    saida.push_str(resto);
    Some(saida)
}

fn entidade(nome: &str) -> Option<char> {
    match nome {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let numero = nome.strip_prefix('#')?;
            let codigo = match numero.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None if numero.bytes().all(|b| b.is_ascii_digit()) => numero.parse().ok()?,
                None => return None,
            };
            char::from_u32(codigo)
        }
    }
}

/// Acrescenta `valor` codificado; `None` se um caractere não existir no charset.
pub(super) fn codificar(
    valor: &str,
    codificacao: CodificacaoDaTela,
    saida: &mut String,
) -> Option<()> {
    for caractere in valor.chars() {
        let mut buffer = [0u8; 4];
        let bytes: &[u8] = match codificacao {
            CodificacaoDaTela::Iso88591 => {
                buffer[0] = u8::try_from(u32::from(caractere)).ok()?;
                &buffer[..1]
            }
            CodificacaoDaTela::Utf8 => caractere.encode_utf8(&mut buffer).as_bytes(),
        };
        for &byte in bytes {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => {
                    saida.push(char::from(byte));
                }
                b' ' => saida.push('+'),
                _ => {
                    let _ = write!(saida, "%{byte:02X}");
                }
            }
        }
    }
    Some(())
}
