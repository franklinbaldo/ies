//! Legenda de contagem das tabelas do SEI (`<caption>`): leitor único das
//! formas observadas no acervo público (issue #799).
//!
//! Formas aceitas, com ou sem rótulo antes do parêntese (o rótulo é do sítio):
//! - `(N registros):` e `N registros:`;
//! - `(N registros - I a F):` e `N registros - I a F:`;
//! - `(1 registro):` e `1 registro:`, sempre com total 1 e sem faixa;
//! - `N registro(s):`, sem rótulo, com qualquer N;
//! - números com separador de milhar no formato canônico (`1.234`,
//!   `12.345.678`), no total e na faixa.
//!
//! A ausência de contagem, a contagem ilegível e a faixa fora de `I a F`
//! são erros tipados; o sítio decide o que fazer com um total ou uma faixa.
use serde::Serialize;
use std::num::ParseIntError;

/// Intervalo `inicio a fim` que a legenda publica, 1-based.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Faixa {
    pub(crate) inicio: usize,
    pub(crate) fim: usize,
}

/// Total declarado pela legenda e, quando publicada, a faixa.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Legenda {
    pub(crate) total: usize,
    pub(crate) faixa: Option<Faixa>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLegenda {
    SemContagem,
    ContagemIlegivel(#[source] ParseIntError),
    FaixaSemSeparador,
    FaixaIlegivel(#[source] ParseIntError),
    SingularComTotalDiferenteDeUm { total: usize },
    SingularComFaixa,
    SeparadorDeMilharFora,
}

/// Lê a legenda inteira (com rótulo, se houver). Espaços, inclusive
/// `U+00A0`, são colapsados antes da leitura.
pub(crate) fn ler_legenda(texto: &str) -> Result<Legenda, ErroLegenda> {
    let normalizado = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    let sem_dois_pontos = normalizado
        .strip_suffix(':')
        .ok_or(ErroLegenda::SemContagem)?;
    let contagem = match sem_dois_pontos.split_once(" (") {
        Some((_rotulo, dentro)) => dentro.strip_suffix(')').ok_or(ErroLegenda::SemContagem)?,
        None => sem_dois_pontos,
    };
    let (corpo, faixa) = match contagem.split_once(" - ") {
        Some((corpo, intervalo)) => (corpo, Some(faixa(intervalo)?)),
        None => (contagem, None),
    };
    let (numero, palavra) = corpo.split_once(' ').ok_or(ErroLegenda::SemContagem)?;
    if !matches!(palavra, "registros" | "registro(s)" | "registro") {
        return Err(ErroLegenda::SemContagem);
    }
    let total = numero_com_milhar(numero, ErroLegenda::ContagemIlegivel)?;
    match palavra {
        "registro" if total != 1 => Err(ErroLegenda::SingularComTotalDiferenteDeUm { total }),
        "registro" if faixa.is_some() => Err(ErroLegenda::SingularComFaixa),
        _ => Ok(Legenda { total, faixa }),
    }
}

fn faixa(intervalo: &str) -> Result<Faixa, ErroLegenda> {
    let (inicio, fim) = intervalo
        .split_once(" a ")
        .ok_or(ErroLegenda::FaixaSemSeparador)?;
    Ok(Faixa {
        inicio: numero_com_milhar(inicio, ErroLegenda::FaixaIlegivel)?,
        fim: numero_com_milhar(fim, ErroLegenda::FaixaIlegivel)?,
    })
}

/// Um número sem separador ou com separador de milhar canônico: grupos de
/// três dígitos após o primeiro, que tem de um a três dígitos.
fn numero_com_milhar(
    texto: &str,
    erro: fn(ParseIntError) -> ErroLegenda,
) -> Result<usize, ErroLegenda> {
    if !texto.contains('.') {
        return texto.parse::<usize>().map_err(erro);
    }
    if !separador_canonico(texto) {
        return Err(ErroLegenda::SeparadorDeMilharFora);
    }
    texto.replace('.', "").parse::<usize>().map_err(erro)
}

fn separador_canonico(texto: &str) -> bool {
    let Some((primeiro, demais)) = texto.split_once('.') else {
        return false;
    };
    (1..=3).contains(&primeiro.len())
        && primeiro.bytes().all(|b| b.is_ascii_digit())
        && demais
            .split('.')
            .all(|g| g.len() == 3 && g.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn erro_de_numero(texto: &str) -> ParseIntError {
        texto.parse::<usize>().unwrap_err()
    }

    #[test]
    fn formas_observadas_sem_faixa() {
        assert_eq!(
            ler_legenda("Lista de Blocos (505 registros):"),
            Ok(Legenda {
                total: 505,
                faixa: None
            })
        );
        assert_eq!(
            ler_legenda("566 registros:"),
            Ok(Legenda {
                total: 566,
                faixa: None
            })
        );
        assert_eq!(
            ler_legenda("Lista de Textos Padrão da Unidade (221 registros):"),
            Ok(Legenda {
                total: 221,
                faixa: None
            })
        );
    }

    #[test]
    fn formas_observadas_com_faixa() {
        assert_eq!(
            ler_legenda("Lista de Blocos (718 registros - 1 a 50):"),
            Ok(Legenda {
                total: 718,
                faixa: Some(Faixa { inicio: 1, fim: 50 })
            })
        );
        assert_eq!(
            ler_legenda("100 registros - 1 a 10:"),
            Ok(Legenda {
                total: 100,
                faixa: Some(Faixa { inicio: 1, fim: 10 })
            })
        );
        assert_eq!(
            ler_legenda("Lista de Unidades (13671 registros - 1 a 50):"),
            Ok(Legenda {
                total: 13671,
                faixa: Some(Faixa { inicio: 1, fim: 50 })
            })
        );
    }

    #[test]
    fn singular_so_com_total_um_e_sem_faixa() {
        assert_eq!(
            ler_legenda("Lista de Ciências (1 registro):"),
            Ok(Legenda {
                total: 1,
                faixa: None
            })
        );
        assert_eq!(
            ler_legenda("1 registro:"),
            Ok(Legenda {
                total: 1,
                faixa: None
            })
        );
        assert_eq!(
            ler_legenda("Lista de Blocos (2 registro):"),
            Err(ErroLegenda::SingularComTotalDiferenteDeUm { total: 2 })
        );
        assert_eq!(
            ler_legenda("Lista (1 registro - 1 a 1):"),
            Err(ErroLegenda::SingularComFaixa)
        );
    }

    #[test]
    fn registro_s_aceita_qualquer_total() {
        assert_eq!(
            ler_legenda("5 registro(s):"),
            Ok(Legenda {
                total: 5,
                faixa: None
            })
        );
        assert_eq!(
            ler_legenda("0 registro(s):"),
            Ok(Legenda {
                total: 0,
                faixa: None
            })
        );
    }

    #[test]
    fn separador_de_milhar_canonico_e_aceito() {
        assert_eq!(
            ler_legenda("Lista de Blocos (1.234 registros):"),
            Ok(Legenda {
                total: 1234,
                faixa: None
            })
        );
        assert_eq!(
            ler_legenda("Processos recebidos (12.345.678 registros - 1.001 a 1.050):"),
            Ok(Legenda {
                total: 12_345_678,
                faixa: Some(Faixa {
                    inicio: 1001,
                    fim: 1050
                })
            })
        );
    }

    #[test]
    fn espacos_inclusive_nbsp_sao_colapsados() {
        assert_eq!(
            ler_legenda("Lista de Blocos (3\u{a0}registros):"),
            Ok(Legenda {
                total: 3,
                faixa: None
            })
        );
    }

    #[test]
    fn formas_vizinhas_sao_recusadas() {
        assert_eq!(
            ler_legenda("Lista de Blocos (3 registros)"),
            Err(ErroLegenda::SemContagem)
        );
        assert_eq!(
            ler_legenda("Lista de Blocos (3 itens):"),
            Err(ErroLegenda::SemContagem)
        );
        assert_eq!(
            ler_legenda("Lista de Blocos:"),
            Err(ErroLegenda::SemContagem)
        );
        assert_eq!(
            ler_legenda("Lista de Blocos (x registros):"),
            Err(ErroLegenda::ContagemIlegivel(erro_de_numero("x")))
        );
        assert_eq!(
            ler_legenda("Lista de Blocos (1.23 registros):"),
            Err(ErroLegenda::SeparadorDeMilharFora)
        );
        assert_eq!(
            ler_legenda("Lista de Blocos (.123 registros):"),
            Err(ErroLegenda::SeparadorDeMilharFora)
        );
        assert_eq!(
            ler_legenda("Lista de Blocos (1..234 registros):"),
            Err(ErroLegenda::SeparadorDeMilharFora)
        );
        assert_eq!(
            ler_legenda("Lista de Blocos (1234.5 registros):"),
            Err(ErroLegenda::SeparadorDeMilharFora)
        );
        assert_eq!(
            ler_legenda("Lista de Blocos (3 registros - 1.23 a 3):"),
            Err(ErroLegenda::SeparadorDeMilharFora)
        );
        assert_eq!(
            ler_legenda("Lista de Blocos (3 registros - 1 e 3):"),
            Err(ErroLegenda::FaixaSemSeparador)
        );
        assert_eq!(
            ler_legenda("Lista de Blocos (3 registros - x a 3):"),
            Err(ErroLegenda::FaixaIlegivel(erro_de_numero("x")))
        );
    }
}
