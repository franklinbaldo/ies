//! Acompanhamentos especiais observados por processo e unidade no SQLite de testes.
//! Hipótese estrutural para BDD-348-01/03; não representa leitura de SEI ao vivo.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Acompanhamento {
    pub id: String,
    pub grupo: String,
    pub observacao: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observacao {
    pub itens: Vec<Acompanhamento>,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroAcompanhamentos {
    Sqlite(#[from] rusqlite::Error),
    DadoInvalido,
    ProcessoNaoObservado,
}

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit())
}

fn valida(observacao: &Observacao) -> bool {
    observacao.itens.iter().enumerate().all(|(i, atual)| {
        id_valido(&atual.id)
            && !observacao
                .itens
                .iter()
                .take(i)
                .any(|anterior| anterior.id == atual.id)
    })
}

/// Registra a página observada, inclusive a lista explicitamente vazia.
/// A ausência dessa fotografia no banco não afirma nenhum acompanhamento.
///
/// # Errors
/// Recusa estado sem processo aberto ou preserva o erro SQLite original.
pub fn registrar(
    banco: &mut Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
    observacao: &Observacao,
) -> Result<(), ErroAcompanhamentos> {
    if cenario.is_empty() || !id_valido(id) || unidade.is_empty() || !valida(observacao) {
        return Err(ErroAcompanhamentos::DadoInvalido);
    }
    let tx = banco.transaction()?;
    let aberto: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM processo_abertura
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4)",
        params![origem.url_base(), cenario, id, unidade],
        |linha| linha.get(0),
    )?;
    if !aberto {
        return Err(ErroAcompanhamentos::ProcessoNaoObservado);
    }
    tx.execute(
        "INSERT INTO processo_acompanhamentos_observados
         (origem, cenario, id_procedimento, unidade) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (origem, cenario, id_procedimento, unidade) DO NOTHING",
        params![origem.url_base(), cenario, id, unidade],
    )?;
    tx.execute(
        "DELETE FROM processo_acompanhamento_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4",
        params![origem.url_base(), cenario, id, unidade],
    )?;
    for (posicao, item) in observacao.itens.iter().enumerate() {
        let posicao = i64::try_from(posicao).map_err(|_| ErroAcompanhamentos::DadoInvalido)?;
        tx.execute(
            "INSERT INTO processo_acompanhamento_linha
             (origem, cenario, id_procedimento, unidade, posicao,
              id_acompanhamento, grupo, observacao)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                origem.url_base(),
                cenario,
                id,
                unidade,
                posicao,
                item.id,
                item.grupo,
                item.observacao
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Distingue observação ausente de lista observada vazia, sem inventar linhas.
///
/// # Errors
/// Propaga erros SQLite e recusa linhas armazenadas com identidade inválida.
pub fn ler(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
) -> Result<Option<Observacao>, ErroAcompanhamentos> {
    let existe: bool = banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM processo_acompanhamentos_observados
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4)",
        params![origem.url_base(), cenario, id, unidade],
        |linha| linha.get(0),
    )?;
    if !existe {
        return Ok(None);
    }
    let mut consulta = banco.prepare(
        "SELECT id_acompanhamento, grupo, observacao
         FROM processo_acompanhamento_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4
         ORDER BY posicao",
    )?;
    let itens = consulta
        .query_map(params![origem.url_base(), cenario, id, unidade], |linha| {
            Ok(Acompanhamento {
                id: linha.get(0)?,
                grupo: linha.get(1)?,
                observacao: linha.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let observacao = Observacao { itens };
    if !valida(&observacao) {
        return Err(ErroAcompanhamentos::DadoInvalido);
    }
    Ok(Some(observacao))
}

/// Remove uma única identidade já observada sem apagar os demais grupos.
/// Retorna false quando a linha não existe, inclusive em segundo envio.
///
/// # Errors
/// Propaga falha SQLite; recusa identidades inválidas ou fotografia ausente.
pub fn retirar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
    id_alvo: &str,
) -> Result<bool, ErroAcompanhamentos> {
    if cenario.is_empty() || unidade.is_empty() || !id_valido(id) || !id_valido(id_alvo) {
        return Err(ErroAcompanhamentos::DadoInvalido);
    }
    if ler(banco, origem, cenario, id, unidade)?.is_none() {
        return Err(ErroAcompanhamentos::ProcessoNaoObservado);
    }
    let removidos = banco.execute(
        "DELETE FROM processo_acompanhamento_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3
           AND unidade=?4 AND id_acompanhamento=?5",
        params![origem.url_base(), cenario, id, unidade, id_alvo],
    )?;
    Ok(removidos == 1)
}
