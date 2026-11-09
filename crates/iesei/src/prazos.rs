//! Estado de prazo explicitamente observado no SQLite descartável do anti-IES.
//! Nenhuma linha não equivale a "sem prazo" e nenhuma escrita no SEI é feita.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EstadoDoPrazo {
    SemPrazo,
    Data(String),
    Dias(u16),
    Concluido,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    EstadoInvalido,
}

impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}

/// Persiste uma observação de prazo tipada, isolada por instalação e cenário.
///
/// # Errors
/// Recusa identidade vazia ou estado inválido e preserva erro do SQLite.
pub fn guardar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    estado: &EstadoDoPrazo,
) -> Result<(), Falha> {
    if cenario.is_empty() || id.is_empty() {
        return Err(Falha::EstadoInvalido);
    }
    let (vence_em, concluido, dias): (Option<&str>, bool, Option<i64>) = match estado {
        EstadoDoPrazo::SemPrazo => (None, false, None),
        EstadoDoPrazo::Data(data) if !data.is_empty() => (Some(data), false, None),
        EstadoDoPrazo::Dias(dias) if (1..=999).contains(dias) => {
            (None, false, Some(i64::from(*dias)))
        }
        EstadoDoPrazo::Data(_) | EstadoDoPrazo::Dias(_) => return Err(Falha::EstadoInvalido),
        EstadoDoPrazo::Concluido => (None, true, None),
    };
    banco.execute(
        "INSERT INTO prazo_processo
         (origem, cenario, id_procedimento, vence_em, concluido, dias)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(origem, cenario, id_procedimento) DO UPDATE SET
         vence_em=excluded.vence_em, concluido=excluded.concluido,
         dias=excluded.dias",
        params![
            instalacao.url_base(),
            cenario,
            id,
            vence_em,
            concluido,
            dias
        ],
    )?;
    Ok(())
}

/// Lê uma observação sem inferir ausência de prazo de linha inexistente.
///
/// # Errors
/// Propaga falha do SQLite ou estado incongruente da observação.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Option<EstadoDoPrazo>, Falha> {
    let linha: Option<(Option<String>, bool, Option<i64>)> = banco
        .query_row(
            "SELECT vence_em, concluido, dias FROM prazo_processo
             WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
            params![instalacao.url_base(), cenario, id],
            |linha| Ok((linha.get(0)?, linha.get(1)?, linha.get(2)?)),
        )
        .optional()?;
    linha
        .map(|(data, concluido, dias)| match (data, concluido, dias) {
            (None, false, None) => Ok(EstadoDoPrazo::SemPrazo),
            (Some(data), false, None) if !data.is_empty() => Ok(EstadoDoPrazo::Data(data)),
            (None, false, Some(dias @ 1..=999)) => u16::try_from(dias)
                .map(EstadoDoPrazo::Dias)
                .map_err(|_| Falha::EstadoInvalido),
            (None, true, None) => Ok(EstadoDoPrazo::Concluido),
            _ => Err(Falha::EstadoInvalido),
        })
        .transpose()
}
