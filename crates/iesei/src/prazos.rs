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

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroPrazos {
    Sqlite(#[from] rusqlite::Error),
    EstadoInvalido,
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
) -> Result<(), ErroPrazos> {
    if cenario.is_empty() || id.is_empty() {
        return Err(ErroPrazos::EstadoInvalido);
    }
    let (vence_em, concluido, dias): (Option<&str>, bool, Option<i64>) = match estado {
        EstadoDoPrazo::SemPrazo => (None, false, None),
        EstadoDoPrazo::Data(data) if !data.is_empty() => (Some(data), false, None),
        EstadoDoPrazo::Dias(dias) if (1..=999).contains(dias) => {
            (None, false, Some(i64::from(*dias)))
        }
        EstadoDoPrazo::Data(_) | EstadoDoPrazo::Dias(_) => return Err(ErroPrazos::EstadoInvalido),
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
) -> Result<Option<EstadoDoPrazo>, ErroPrazos> {
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
                .map_err(|_| ErroPrazos::EstadoInvalido),
            (None, true, None) => Ok(EstadoDoPrazo::Concluido),
            _ => Err(ErroPrazos::EstadoInvalido),
        })
        .transpose()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinhaDePrazo {
    pub id_procedimento: String,
    pub nup: String,
    pub estado: EstadoDoPrazo,
}

/// Lista apenas os controles de prazo em vigor na unidade observada.
///
/// # Errors
/// Propaga falhas SQLite, sem transformar ausencia de observacao em lista vazia.
pub fn listar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &str,
) -> Result<Vec<LinhaDePrazo>, ErroPrazos> {
    let mut consulta = banco.prepare(
        "SELECT prazo.id_procedimento, base.nup, prazo.vence_em, prazo.dias
         FROM prazo_processo prazo
         JOIN sim_processo base
           ON base.origem=prazo.origem AND base.cenario=prazo.cenario
          AND base.id_procedimento=prazo.id_procedimento
         JOIN processo_abertura aberta
           ON aberta.origem=prazo.origem AND aberta.cenario=prazo.cenario
          AND aberta.id_procedimento=prazo.id_procedimento
         WHERE prazo.origem=?1 AND prazo.cenario=?2 AND aberta.unidade=?3
           AND (prazo.vence_em IS NOT NULL OR prazo.dias IS NOT NULL)
           AND prazo.concluido=0
         ORDER BY prazo.id_procedimento",
    )?;
    let linhas = consulta
        .query_map(params![instalacao.url_base(), cenario, unidade], |linha| {
            Ok((
                linha.get::<_, String>(0)?,
                linha.get::<_, String>(1)?,
                linha.get::<_, Option<String>>(2)?,
                linha.get::<_, Option<i64>>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    linhas
        .into_iter()
        .map(|(id_procedimento, nup, data, dias)| {
            let estado = match (data, dias) {
                (Some(data), None) if !data.is_empty() => EstadoDoPrazo::Data(data),
                (None, Some(dias @ 1..=999)) => EstadoDoPrazo::Dias(
                    u16::try_from(dias).map_err(|_| ErroPrazos::EstadoInvalido)?,
                ),
                _ => return Err(ErroPrazos::EstadoInvalido),
            };
            Ok(LinhaDePrazo {
                id_procedimento,
                nup,
                estado,
            })
        })
        .collect()
}

/// Retira o prazo ativo, preservando a observacao explicita `SemPrazo`.
/// Identidade/escopo devem ter sido conferidos na lista da unidade.
///
/// # Errors
/// Propaga falha de escrita SQLite; retorna false se nao havia prazo ativo.
pub fn excluir(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<bool, ErroPrazos> {
    let linhas = banco.execute(
        "UPDATE prazo_processo
         SET vence_em=NULL, dias=NULL, concluido=0
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3
           AND concluido=0 AND (vence_em IS NOT NULL OR dias IS NOT NULL)",
        params![instalacao.url_base(), cenario, id],
    )?;
    Ok(linhas == 1)
}
