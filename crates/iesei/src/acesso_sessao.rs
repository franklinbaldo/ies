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

/// Registra a identidade conhecida de um usuário de ensaio por origem/cenário.
/// Não interpreta cookie, não lê login real e não infere autoria da revisão.
/// Mudar esta identidade em um ensaio não modifica autorias já registradas.
///
/// # Errors
/// Propaga a causa do SQLite; nome vazio é recusado pelo esquema.
pub fn identificar_usuario(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    usuario: &str,
) -> rusqlite::Result<()> {
    banco.execute(
        "INSERT INTO sessao_usuario (origem, cenario, usuario) VALUES (?1, ?2, ?3)
         ON CONFLICT(origem, cenario) DO UPDATE SET usuario=excluded.usuario",
        params![origem.url_base(), cenario, usuario],
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
