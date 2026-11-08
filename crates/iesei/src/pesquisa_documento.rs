//! Resultado negativo explicitamente observado no cenário de busca documental.
//! Ausência de linha não comprova ausência de documentos no SEI real.
use crate::instalacao::Instalacao;
use crate::observacao::UnidadeExibida;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resultado {
    SemResultado,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    EntradaInvalida,
}
impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}

fn valido(cenario: &str, termo: &str) -> bool {
    !cenario.is_empty() && !termo.trim().is_empty() && !termo.chars().any(char::is_control)
}

/// Registra somente uma busca cuja resposta negativa foi observada no cenário.
///
/// # Errors
/// Retorna a causa SQLite ou a invalidez do termo/contexto.
pub fn registrar_sem_resultado(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    termo: &str,
) -> Result<(), Falha> {
    if !valido(cenario, termo) {
        return Err(Falha::EntradaInvalida);
    }
    banco.execute(
        "INSERT INTO pesquisa_documento_sem_resultado
         (origem, cenario, unidade, termo) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(origem, cenario, unidade, termo) DO NOTHING",
        params![instalacao.url_base(), cenario, unidade.as_str(), termo],
    )?;
    Ok(())
}

/// `None` significa desconhecido, não uma pesquisa que encontrou zero itens.
///
/// # Errors
/// Propaga erro SQL e recusa contexto inválido.
pub fn consultar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    termo: &str,
) -> Result<Option<Resultado>, Falha> {
    if !valido(cenario, termo) {
        return Err(Falha::EntradaInvalida);
    }
    let achado: Option<i64> = banco
        .query_row(
            "SELECT 1 FROM pesquisa_documento_sem_resultado
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND termo=?4",
            params![instalacao.url_base(), cenario, unidade.as_str(), termo],
            |linha| linha.get(0),
        )
        .optional()?;
    Ok(achado.map(|_| Resultado::SemResultado))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registro_idempotente_e_ausencia_nao_inferida() {
        let banco = Connection::open_in_memory().unwrap();
        crate::banco_cassettes::iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let unidade = UnidadeExibida::nova("TESTE").unwrap();
        assert!(
            consultar(&banco, &origem, "cenario", &unidade, "sem")
                .unwrap()
                .is_none()
        );
        registrar_sem_resultado(&banco, &origem, "cenario", &unidade, "sem").unwrap();
        registrar_sem_resultado(&banco, &origem, "cenario", &unidade, "sem").unwrap();
        assert_eq!(
            consultar(&banco, &origem, "cenario", &unidade, "sem").unwrap(),
            Some(Resultado::SemResultado)
        );
        assert!(
            consultar(&banco, &origem, "cenario", &unidade, "outro")
                .unwrap()
                .is_none()
        );
        assert!(
            consultar(
                &banco,
                &origem,
                "cenario",
                &UnidadeExibida::nova("OUTRA").unwrap(),
                "sem"
            )
            .unwrap()
            .is_none()
        );
    }
}
