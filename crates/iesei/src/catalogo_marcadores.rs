//! Catálogo da unidade observado independentemente dos marcadores de processos.
//! Ausência de registro significa desconhecido, não catálogo vazio.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opcao {
    pub id: String,
    pub nome: String,
    pub cor: String,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroCatalogoMarcadores {
    Sqlite(#[from] rusqlite::Error),
    DadoInvalido,
}

fn valido(itens: &[Opcao]) -> bool {
    for (i, item) in itens.iter().enumerate() {
        if item.id.is_empty()
            || !item.id.bytes().all(|b| b.is_ascii_digit())
            || item.nome.trim().is_empty()
            || item.nome.chars().any(char::is_control)
            || item.cor.is_empty()
            || !item
                .cor
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_')
            || itens[..i].iter().any(|outro| outro.id == item.id)
        {
            return false;
        }
    }
    true
}

/// Registra observação completa de catálogo da unidade, nunca de um processo.
/// Uma lista vazia é explícita e distinta de falta de observação.
///
/// # Errors
/// Recusa entradas inválidas e propaga a falha original do SQLite.
pub fn registrar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &str,
    itens: &[Opcao],
) -> Result<(), ErroCatalogoMarcadores> {
    if cenario.is_empty() || unidade.is_empty() || !valido(itens) {
        return Err(ErroCatalogoMarcadores::DadoInvalido);
    }
    let total = i64::try_from(itens.len()).map_err(|_| ErroCatalogoMarcadores::DadoInvalido)?;
    let tx = banco.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO unidade_catalogo_marcadores
         (origem, cenario, unidade, total_declarado) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (origem, cenario, unidade)
         DO UPDATE SET total_declarado=excluded.total_declarado",
        params![instalacao.url_base(), cenario, unidade, total],
    )?;
    tx.execute(
        "DELETE FROM unidade_catalogo_marcador_opcao
         WHERE origem=?1 AND cenario=?2 AND unidade=?3",
        params![instalacao.url_base(), cenario, unidade],
    )?;
    for (posicao, item) in itens.iter().enumerate() {
        let posicao = i64::try_from(posicao).map_err(|_| ErroCatalogoMarcadores::DadoInvalido)?;
        tx.execute(
            "INSERT INTO unidade_catalogo_marcador_opcao
             (origem, cenario, unidade, posicao, id_marcador, nome, cor)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                instalacao.url_base(),
                cenario,
                unidade,
                posicao,
                item.id,
                item.nome,
                item.cor
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Consulta catálogo observado da unidade (não inventa opções a partir de processos).
///
/// # Errors
/// Preserva erros SQL e rejeita observação incompleta.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &str,
) -> Result<Option<Vec<Opcao>>, ErroCatalogoMarcadores> {
    let total: Option<i64> = banco
        .query_row(
            "SELECT total_declarado FROM unidade_catalogo_marcadores
             WHERE origem=?1 AND cenario=?2 AND unidade=?3",
            params![instalacao.url_base(), cenario, unidade],
            |linha| linha.get(0),
        )
        .optional()?;
    let Some(total) = total else { return Ok(None) };
    let total = usize::try_from(total).map_err(|_| ErroCatalogoMarcadores::DadoInvalido)?;
    let mut consulta = banco.prepare(
        "SELECT id_marcador, nome, cor FROM unidade_catalogo_marcador_opcao
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 ORDER BY posicao",
    )?;
    let itens = consulta
        .query_map(params![instalacao.url_base(), cenario, unidade], |r| {
            Ok(Opcao {
                id: r.get(0)?,
                nome: r.get(1)?,
                cor: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if itens.len() != total || !valido(&itens) {
        return Err(ErroCatalogoMarcadores::DadoInvalido);
    }
    Ok(Some(itens))
}
