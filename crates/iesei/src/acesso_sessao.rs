//! Acesso sintético à sessão inteira, isolado por origem e cenário.
//! Não guarda identidade, credenciais ou cookies de uma sessão real.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcessoSessao {
    Permitido,
    Recusado,
}

/// Simula uma recusa explícita da sessão no cenário, de forma idempotente.
///
/// # Errors
/// Propaga a causa SQLite original.
pub fn bloquear(banco: &Connection, origem: &Instalacao, cenario: &str) -> rusqlite::Result<()> {
    banco.execute(
        "INSERT OR IGNORE INTO sessao_bloqueada (origem, cenario) VALUES (?1, ?2)",
        params![origem.url_base(), cenario],
    )?;
    Ok(())
}

/// Uma sessão só é recusada quando há negativa explícita no SQLite.
/// Ausência dessa negativa não é comprovação de permissão em SEI real.
///
/// # Errors
/// Propaga a causa SQLite original.
pub fn consultar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> rusqlite::Result<AcessoSessao> {
    let negada = banco
        .query_row(
            "SELECT 1 FROM sessao_bloqueada WHERE origem=?1 AND cenario=?2",
            params![origem.url_base(), cenario],
            |linha| linha.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    Ok(if negada {
        AcessoSessao::Recusado
    } else {
        AcessoSessao::Permitido
    })
}
