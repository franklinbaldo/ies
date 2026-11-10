//! Estado observado do histórico de processo no SQLite efêmero.
//! Ausência de observação não significa histórico vazio.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Linha {
    pub ocorrido_em: String,
    pub unidade: String,
    pub usuario: String,
    pub descricao: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Historico {
    pub total_declarado: u64,
    pub linhas: Vec<Linha>,
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

fn valido(id: &str, historico: &Historico) -> bool {
    !id.is_empty()
        && id.bytes().all(|b| b.is_ascii_digit())
        && !historico.linhas.is_empty()
        && usize::try_from(historico.total_declarado)
            .is_ok_and(|total| total >= historico.linhas.len())
        && historico.linhas.iter().all(|linha| {
            !linha.ocorrido_em.trim().is_empty()
                && !linha.descricao.trim().is_empty()
                && !linha.unidade.trim().is_empty()
                && !linha.usuario.trim().is_empty()
        })
}

/// Persiste exatamente as linhas observadas, em ordem, com substituição atômica.
///
/// # Errors
/// Preserva a causa SQLite ou recusa observação inconsistente.
pub fn guardar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    historico: &Historico,
) -> Result<(), Falha> {
    let tx = banco.unchecked_transaction()?;
    guardar_no_banco(&tx, instalacao, cenario, id, historico)?;
    tx.commit()?;
    Ok(())
}

fn guardar_no_banco(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    historico: &Historico,
) -> Result<(), Falha> {
    if cenario.is_empty() || !valido(id, historico) {
        return Err(Falha::DadoInvalido);
    }
    let origem = instalacao.url_base();
    let total = i64::try_from(historico.total_declarado).map_err(|_| Falha::DadoInvalido)?;
    banco.execute(
        "INSERT INTO andamento_historico (origem, cenario, id_procedimento, total_declarado)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (origem, cenario, id_procedimento) DO UPDATE SET
         total_declarado=excluded.total_declarado",
        params![origem, cenario, id, total],
    )?;
    banco.execute(
        "DELETE FROM andamento_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem, cenario, id],
    )?;
    for (posicao, linha) in historico.linhas.iter().enumerate() {
        banco.execute(
            "INSERT INTO andamento_linha
             (origem, cenario, id_procedimento, posicao, ocorrido_em, unidade, usuario, descricao)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                origem,
                cenario,
                id,
                i64::try_from(posicao).map_err(|_| Falha::DadoInvalido)?,
                linha.ocorrido_em,
                linha.unidade,
                linha.usuario,
                linha.descricao,
            ],
        )?;
    }
    Ok(())
}

/// Acrescenta uma linha ao histórico observado, preservando todas as anteriores.
/// O simulador é sequencial e a linha nova explicita metadados sintéticos.
///
/// # Errors
/// Propaga causas SQLite ou recusa observação incoerente.
pub fn acrescentar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    linha: Linha,
) -> Result<(), Falha> {
    // Serializa escritores antes da leitura, evitando atualizações perdidas.
    let tx = rusqlite::Transaction::new_unchecked(
        banco,
        rusqlite::TransactionBehavior::Immediate,
    )?;
    let Some(mut historico) = ler(&tx, instalacao, cenario, id)? else {
        return Err(Falha::DadoInvalido);
    };
    historico.total_declarado = historico
        .total_declarado
        .checked_add(1)
        .ok_or(Falha::DadoInvalido)?;
    historico.linhas.insert(0, linha);
    guardar_no_banco(&tx, instalacao, cenario, id, &historico)?;
    tx.commit()?;
    Ok(())
}

/// Reconstrói somente a observação presente e validada.
///
/// # Errors
/// Preserva o erro SQLite e recusa linhas corrompidas.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Option<Historico>, Falha> {
    let origem = instalacao.url_base();
    let total: Option<i64> = banco
        .query_row(
            "SELECT total_declarado FROM andamento_historico
             WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
            params![origem, cenario, id],
            |linha| linha.get(0),
        )
        .optional()?;
    let Some(total) = total else {
        return Ok(None);
    };
    let mut stmt = banco.prepare(
        "SELECT ocorrido_em, unidade, usuario, descricao FROM andamento_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3
         ORDER BY posicao",
    )?;
    let linhas = stmt
        .query_map(params![origem, cenario, id], |linha| {
            Ok(Linha {
                ocorrido_em: linha.get(0)?,
                unidade: linha.get(1)?,
                usuario: linha.get(2)?,
                descricao: linha.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let total_declarado = u64::try_from(total).map_err(|_| Falha::DadoInvalido)?;
    let historico = Historico {
        total_declarado,
        linhas,
    };
    if !valido(id, &historico) {
        return Err(Falha::DadoInvalido);
    }
    Ok(Some(historico))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banco_cassettes;

    #[test]
    fn nao_confunde_ausencia_com_historico_observado_e_regrava_idempotente() {
        let banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        assert!(ler(&banco, &origem, "cena", "10").unwrap().is_none());
        let historico = Historico {
            total_declarado: 1,
            linhas: vec![Linha {
                ocorrido_em: "09/11/2026 10:00".into(),
                unidade: "TESTE".into(),
                usuario: "aluno".into(),
                descricao: "Anterior".into(),
            }],
        };
        guardar(&banco, &origem, "cena", "10", &historico).unwrap();
        guardar(&banco, &origem, "cena", "10", &historico).unwrap();
        assert_eq!(ler(&banco, &origem, "cena", "10").unwrap(), Some(historico));
        assert!(ler(&banco, &origem, "outra", "10").unwrap().is_none());
        assert!(ler(&banco, &origem, "cena", "11").unwrap().is_none());
    }

    #[test]
    fn dados_parciais_ou_inconsistentes_nao_viram_historico_completo() {
        let banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let incompleto = Historico {
            total_declarado: 0,
            linhas: vec![],
        };
        assert!(matches!(
            guardar(&banco, &origem, "cena", "10", &incompleto),
            Err(Falha::DadoInvalido)
        ));
        assert!(ler(&banco, &origem, "cena", "10").unwrap().is_none());
    }
}
