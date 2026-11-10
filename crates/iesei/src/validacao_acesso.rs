//! Revalidacao de acesso por senha a um processo no banco efemero do anti-IES.
//! A modal em GO e observada; forma de submissao e resultado sao hipoteticos.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Validacao {
    Exigida,
    Realizada,
}

impl Validacao {
    fn como_sql(self) -> &'static str {
        match self {
            Self::Exigida => "exigida",
            Self::Realizada => "realizada",
        }
    }
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    EstadoInvalido,
    IdentidadeInvalida,
}

impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())
}

/// Guarda um estado explicito, sem senha, token ou endereco assinado.
///
/// # Errors
/// Recusa identidade invalida e preserva falha SQLite.
pub fn guardar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    estado: Validacao,
) -> Result<(), Falha> {
    if cenario.is_empty() || !id_valido(id) {
        return Err(Falha::IdentidadeInvalida);
    }
    banco.execute(
        "INSERT INTO processo_validacao_acesso
         (origem, cenario, id_procedimento, situacao)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(origem, cenario, id_procedimento)
         DO UPDATE SET situacao=excluded.situacao",
        params![instalacao.url_base(), cenario, id, estado.como_sql()],
    )?;
    Ok(())
}

/// Sem linha: estado desconhecido, nunca confirmacao de senha validada.
///
/// # Errors
/// Propaga falha SQLite e recusa situacao nao tipada.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Option<Validacao>, Falha> {
    if cenario.is_empty() || !id_valido(id) {
        return Err(Falha::IdentidadeInvalida);
    }
    let estado: Option<String> = banco
        .query_row(
            "SELECT situacao FROM processo_validacao_acesso
             WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
            params![instalacao.url_base(), cenario, id],
            |linha| linha.get(0),
        )
        .optional()?;
    match estado.as_deref() {
        None => Ok(None),
        Some("exigida") => Ok(Some(Validacao::Exigida)),
        Some("realizada") => Ok(Some(Validacao::Realizada)),
        _ => Err(Falha::EstadoInvalido),
    }
}
