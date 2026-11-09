//! Estado de prazo explicitamente observado no SQLite descartável do anti-IES.
//! Nenhuma linha não equivale a "sem prazo" e nenhuma escrita no SEI é feita.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EstadoDoPrazo {
    SemPrazo,
    Data(String),
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
    let (vence_em, concluido): (Option<&str>, bool) = match estado {
        EstadoDoPrazo::SemPrazo => (None, false),
        EstadoDoPrazo::Data(data) if !data.is_empty() => (Some(data), false),
        EstadoDoPrazo::Data(_) => return Err(Falha::EstadoInvalido),
        EstadoDoPrazo::Concluido => (None, true),
    };
    banco.execute(
        "INSERT INTO prazo_processo
         (origem, cenario, id_procedimento, vence_em, concluido)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(origem, cenario, id_procedimento) DO UPDATE SET
         vence_em=excluded.vence_em, concluido=excluded.concluido",
        params![instalacao.url_base(), cenario, id, vence_em, concluido],
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
    let linha: Option<(Option<String>, bool)> = banco
        .query_row(
            "SELECT vence_em, concluido FROM prazo_processo
             WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
            params![instalacao.url_base(), cenario, id],
            |linha| Ok((linha.get(0)?, linha.get(1)?)),
        )
        .optional()?;
    linha
        .map(|(data, concluido)| match (data, concluido) {
            (None, false) => Ok(EstadoDoPrazo::SemPrazo),
            (Some(data), false) if !data.is_empty() => Ok(EstadoDoPrazo::Data(data)),
            (None, true) => Ok(EstadoDoPrazo::Concluido),
            _ => Err(Falha::EstadoInvalido),
        })
        .transpose()
}
