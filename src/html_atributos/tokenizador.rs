//! Máquina de estados WHATWG restrita à tag de abertura: antes do nome,
//! nome, depois do nome, antes do valor, valor entre aspas duplas, entre
//! aspas simples ou sem aspas, e depois do valor entre aspas. Opera sobre
//! bytes ASCII; os cortes caem sempre em fronteiras de caractere UTF-8.
use super::{AtributosDaTag, ErroAtributoHtml};

#[derive(Clone, Copy)]
enum Estado {
    AntesDoNome,
    Nome { inicio: usize },
    DepoisDoNome,
    AntesDoValor,
    ValorEntreAspas { aspa: u8, inicio: usize },
    ValorSemAspas { inicio: usize },
    DepoisDoValorEntreAspas,
}

fn espaco(b: u8) -> bool {
    matches!(b, b'\t' | b'\n' | 0x0c | b'\r' | b' ')
}

/// Tokeniza `<nome atributos...>`; ignora o que vem depois do `>`.
pub(crate) fn tokenizar_tag_de_abertura(
    fonte: &str,
) -> Result<AtributosDaTag<'_>, ErroAtributoHtml> {
    let bytes = fonte.as_bytes();
    if bytes.first() != Some(&b'<') || !bytes.get(1).is_some_and(u8::is_ascii_alphabetic) {
        return Err(ErroAtributoHtml::InicioDaTagDeAberturaAusente);
    }
    // Estado "nome da tag": até espaço, `/` ou `>`.
    let mut i = 1;
    while i < bytes.len() && !espaco(bytes[i]) && !matches!(bytes[i], b'/' | b'>') {
        i += 1;
    }
    let mut pares: Vec<(String, Option<&str>)> = Vec::new();
    let mut estado = Estado::AntesDoNome;
    while let Some(&b) = bytes.get(i) {
        estado = match estado {
            Estado::AntesDoNome if espaco(b) || b == b'/' => Estado::AntesDoNome,
            Estado::AntesDoNome if b == b'>' => return Ok(AtributosDaTag::novo(pares)),
            // `=` no início do nome faz parte do nome (erro de parse WHATWG).
            Estado::AntesDoNome => Estado::Nome { inicio: i },
            Estado::Nome { inicio } => {
                if !espaco(b) && !matches!(b, b'/' | b'>' | b'=') {
                    i += 1;
                    continue;
                }
                pares.push((fonte[inicio..i].to_ascii_lowercase(), None));
                match b {
                    b'=' => Estado::AntesDoValor,
                    b'>' => return Ok(AtributosDaTag::novo(pares)),
                    b'/' => Estado::AntesDoNome,
                    _ => Estado::DepoisDoNome,
                }
            }
            Estado::DepoisDoNome => match b {
                _ if espaco(b) => estado,
                b'=' => Estado::AntesDoValor,
                b'>' => return Ok(AtributosDaTag::novo(pares)),
                b'/' => Estado::AntesDoNome,
                _ => Estado::Nome { inicio: i },
            },
            Estado::AntesDoValor => match b {
                _ if espaco(b) => estado,
                b'"' | b'\'' => Estado::ValorEntreAspas {
                    aspa: b,
                    inicio: i + 1,
                },
                // `nome=>`: valor ausente vira vazio e a tag termina.
                b'>' => {
                    definir_valor(&mut pares, "");
                    return Ok(AtributosDaTag::novo(pares));
                }
                _ => Estado::ValorSemAspas { inicio: i },
            },
            Estado::ValorEntreAspas { aspa, inicio } => {
                if b != aspa {
                    i += 1;
                    continue;
                }
                definir_valor(&mut pares, &fonte[inicio..i]);
                Estado::DepoisDoValorEntreAspas
            }
            Estado::ValorSemAspas { inicio } => {
                if !espaco(b) && b != b'>' {
                    i += 1;
                    continue;
                }
                definir_valor(&mut pares, &fonte[inicio..i]);
                if b == b'>' {
                    return Ok(AtributosDaTag::novo(pares));
                }
                Estado::AntesDoNome
            }
            Estado::DepoisDoValorEntreAspas => match b {
                b'>' => return Ok(AtributosDaTag::novo(pares)),
                _ if espaco(b) || b == b'/' => Estado::AntesDoNome,
                // Falta de espaço entre atributos: reconsome como novo nome.
                _ => Estado::Nome { inicio: i },
            },
        };
        i += 1;
    }
    Err(match estado {
        Estado::ValorEntreAspas { .. } => ErroAtributoHtml::ValorEntreAspasSemFechamento,
        Estado::AntesDoNome
        | Estado::Nome { .. }
        | Estado::DepoisDoNome
        | Estado::AntesDoValor
        | Estado::ValorSemAspas { .. }
        | Estado::DepoisDoValorEntreAspas => ErroAtributoHtml::FimDaTagDeAberturaAusente,
    })
}

fn definir_valor<'a>(pares: &mut [(String, Option<&'a str>)], valor: &'a str) {
    if let Some((_, slot)) = pares.last_mut() {
        *slot = Some(valor);
    }
}
