//! Uma pasta com carregamento não comprovado no anti-IES descartável.
//! Hipótese de falha para BDD-342-01; não é transcrição de cassette.

use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PastaPendente {
    pub indice: u32,
    pub protocolo_declarado: String,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    DadoInvalido,
}

impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}

fn id_numerico(valor: &str) -> bool {
    !valor.is_empty() && valor.bytes().all(|c| c.is_ascii_digit())
}

fn valida(pasta: &PastaPendente, id: &str, cenario: &str) -> bool {
    pasta.indice > 0
        && id_numerico(id)
        && !cenario.is_empty()
        && id_numerico(&pasta.protocolo_declarado)
}

/// Acrescenta uma hipótese de pasta pendente ao processo conhecido.
/// Não grava HTML, tokens ou qualquer estado do SEI operacional.
///
/// # Errors
/// Recusa identidade inválida e propaga falhas SQLite.
pub fn guardar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
    pasta: &PastaPendente,
) -> Result<(), Falha> {
    if !valida(pasta, id_procedimento, cenario) {
        return Err(Falha::DadoInvalido);
    }
    banco.execute(
        "INSERT INTO sim_pasta_pendente
         (origem, cenario, id_procedimento, indice, protocolo_declarado)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(origem, cenario, id_procedimento) DO UPDATE SET
         indice=excluded.indice, protocolo_declarado=excluded.protocolo_declarado",
        params![
            instalacao.url_base(),
            cenario,
            id_procedimento,
            i64::from(pasta.indice),
            pasta.protocolo_declarado
        ],
    )?;
    Ok(())
}

/// Distingue ausência da hipótese (None) de pasta sabidamente pendente.
///
/// # Errors
/// Preserva erros SQLite e não interpreta linhas inválidas como ausência.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
) -> Result<Option<PastaPendente>, Falha> {
    let linha: Option<(i64, String)> = banco
        .query_row(
            "SELECT indice, protocolo_declarado
             FROM sim_pasta_pendente
             WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
            params![instalacao.url_base(), cenario, id_procedimento],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    linha
        .map(|(indice, protocolo_declarado)| {
            let pasta = PastaPendente {
                indice: u32::try_from(indice).map_err(|_| Falha::DadoInvalido)?,
                protocolo_declarado,
            };
            if !valida(&pasta, id_procedimento, cenario) {
                return Err(Falha::DadoInvalido);
            }
            Ok(pasta)
        })
        .transpose()
}
