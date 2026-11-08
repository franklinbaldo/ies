//! Tipos de processo oferecidos pelo cadastro simulado, com identidade no SQLite.
//! Observação real e hipóteses sintéticas devem permanecer identificadas pelo cenário.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TipoDeProcesso {
    pub id: String,
    pub nome: String,
    pub id_assunto: String,
    pub assunto: String,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    DadoInvalido,
}
impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self { Self::Sqlite(erro) }
}

fn validar(tipo: &TipoDeProcesso) -> bool {
    [&tipo.id, &tipo.id_assunto].iter().all(|s| {
        !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
    }) && [&tipo.nome, &tipo.assunto].iter().all(|s| {
        !s.trim().is_empty() && !s.chars().any(char::is_control)
    })
}

/// Registra uma escolha de tipo que a instalação ofereceu, sem criar processo.
///
/// # Errors
/// Recusa dados inválidos e preserva falhas SQLite.
pub fn registrar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    tipo: &TipoDeProcesso,
) -> Result<(), Falha> {
    if cenario.is_empty() || !validar(tipo) {
        return Err(Falha::DadoInvalido);
    }
    banco.execute(
        "INSERT INTO tipo_processo_simulado
         (origem, cenario, id_tipo, nome, id_assunto, assunto)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(origem, cenario, id_tipo) DO UPDATE SET
         nome=excluded.nome, id_assunto=excluded.id_assunto,
         assunto=excluded.assunto",
        params![instalacao.url_base(), cenario, tipo.id, tipo.nome, tipo.id_assunto, tipo.assunto],
    )?;
    Ok(())
}

/// Nenhuma linha significa cenário de escolha não observado, não lista vazia.
///
/// # Errors
/// Propaga falhas de leitura do SQLite.
pub fn listar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> rusqlite::Result<Option<Vec<TipoDeProcesso>>> {
    let mut consulta = banco.prepare(
        "SELECT id_tipo, nome, id_assunto, assunto
         FROM tipo_processo_simulado WHERE origem=?1 AND cenario=?2
         ORDER BY id_tipo",
    )?;
    let tipos = consulta.query_map(params![instalacao.url_base(), cenario], |r| {
        Ok(TipoDeProcesso {
            id: r.get(0)?,
            nome: r.get(1)?,
            id_assunto: r.get(2)?,
            assunto: r.get(3)?,
        })
    })?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok((!tipos.is_empty()).then_some(tipos))
}

/// Identificador de tipo informado na URL não autoriza inventar uma escolha.
///
/// # Errors
/// Preserva erro SQLite.
pub fn consultar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> rusqlite::Result<Option<TipoDeProcesso>> {
    banco.query_row(
        "SELECT id_tipo, nome, id_assunto, assunto FROM tipo_processo_simulado
         WHERE origem=?1 AND cenario=?2 AND id_tipo=?3",
        params![instalacao.url_base(), cenario, id],
        |r| Ok(TipoDeProcesso {
            id: r.get(0)?, nome: r.get(1)?, id_assunto: r.get(2)?, assunto: r.get(3)?,
        }),
    ).optional()
}
