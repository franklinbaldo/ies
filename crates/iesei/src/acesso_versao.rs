//! Acesso sintético a versões: negativa registrada por cenário e instalação.
//! Não armazena dados de autenticação nem altera permissão no SEI real.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};
use std::num::NonZeroU32;

/// Acesso à versão no cenário montado pelo ensaio.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcessoVersao {
    Acessivel,
    Recusado,
}

/// Marca uma versão como inacessível na sessão sintética.
///
/// # Errors
/// Propaga o erro SQLite.
pub fn restringir(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id_documento: &str,
    versao: NonZeroU32,
) -> rusqlite::Result<()> {
    banco.execute(
        "INSERT OR IGNORE INTO versao_inacessivel
         (origem, cenario, id_documento, versao) VALUES (?1, ?2, ?3, ?4)",
        params![
            instalacao.url_base(),
            cenario,
            id_documento,
            i64::from(versao.get())
        ],
    )?;
    Ok(())
}

/// Consulta restrição explicitamente definida. Versões não restritas
/// no snapshot continuam acessíveis, sem inferir permissões reais.
///
/// # Errors
/// Propaga o erro SQLite.
pub fn consultar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id_documento: &str,
    versao: NonZeroU32,
) -> rusqlite::Result<AcessoVersao> {
    let restrita = banco
        .query_row(
            "SELECT 1 FROM versao_inacessivel
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3 AND versao=?4",
            params![
                instalacao.url_base(),
                cenario,
                id_documento,
                i64::from(versao.get())
            ],
            |linha| linha.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    Ok(if restrita {
        AcessoVersao::Recusado
    } else {
        AcessoVersao::Acessivel
    })
}
