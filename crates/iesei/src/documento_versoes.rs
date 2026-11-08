//! Versões de documentos simulados no SQLite efêmero, independentes do IES operacional.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::HashSet;
use std::num::NonZeroU32;

/// Conteúdo de uma versão, sem links assinados ou cookies persistidos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revisao {
    pub versao: NonZeroU32,
    pub usuario: String,
    pub unidade: String,
    pub modificacao: String,
    pub html_corpo: String,
}

/// Identidade compartilhada pelas versões no mesmo cenário.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentoSimulado {
    pub instalacao: Instalacao,
    pub id_documento: String,
    pub id_procedimento: String,
    pub numero: String,
    pub tipo: String,
    pub revisoes: Vec<Revisao>,
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

/// Reconstrói as versões do documento de forma atômica e idempotente.
///
/// # Errors
/// Recusa identidade, revisão repetida ou dados incompletos e preserva
/// a causa SQLite original.
pub fn guardar(
    banco: &mut Connection,
    cenario: &str,
    documento: &DocumentoSimulado,
) -> Result<(), Falha> {
    let valido = |texto: &str| !texto.trim().is_empty();
    if !valido(cenario)
        || !valido(&documento.id_procedimento)
        || !documento
            .id_procedimento
            .bytes()
            .all(|b| b.is_ascii_digit())
        || !valido(&documento.numero)
        || !documento.numero.bytes().all(|b| b.is_ascii_digit())
        || !valido(&documento.id_documento)
        || !documento.id_documento.bytes().all(|b| b.is_ascii_digit())
        || !valido(&documento.tipo)
        || documento.revisoes.is_empty()
    {
        return Err(Falha::DadoInvalido);
    }
    let mut vistas = HashSet::new();
    for revisao in &documento.revisoes {
        if !vistas.insert(revisao.versao)
            || !valido(&revisao.usuario)
            || !valido(&revisao.unidade)
            || !valido(&revisao.modificacao)
            || !valido(&revisao.html_corpo)
            || revisao.html_corpo.contains("infra_hash")
        {
            return Err(Falha::DadoInvalido);
        }
    }
    let origem = documento.instalacao.url_base();
    let tx = banco.transaction()?;
    tx.execute(
        "INSERT INTO documento_simulado (origem, cenario, id_documento, id_procedimento, numero, tipo)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(origem, cenario, id_documento) DO UPDATE SET
         id_procedimento=excluded.id_procedimento,
         numero=excluded.numero, tipo=excluded.tipo",
        params![origem, cenario, documento.id_documento, documento.id_procedimento, documento.numero, documento.tipo],
    )?;
    tx.execute(
        "DELETE FROM documento_revisao WHERE origem=?1 AND cenario=?2 AND id_documento=?3",
        params![origem, cenario, documento.id_documento],
    )?;
    for r in &documento.revisoes {
        tx.execute(
            "INSERT INTO documento_revisao
             (origem, cenario, id_documento, versao, usuario, unidade, modificacao, html_corpo)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                origem,
                cenario,
                documento.id_documento,
                i64::from(r.versao.get()),
                r.usuario,
                r.unidade,
                r.modificacao,
                r.html_corpo
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Recupera as revisões da identidade no cenário indicado, sem inventar
/// a versão atual nem confundir documentos de outras instalações.
///
/// # Errors
/// Preserva erro SQLite e recusa revisão persistida que não caiba no tipo.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id_documento: &str,
) -> Result<Option<DocumentoSimulado>, Falha> {
    let origem = instalacao.url_base();
    let cabecalho: Option<(String, String, String)> = banco
        .query_row(
            "SELECT id_procedimento, numero, tipo FROM documento_simulado
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3",
            params![origem, cenario, id_documento],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((id_procedimento, numero, tipo)) = cabecalho else {
        return Ok(None);
    };
    let mut stmt = banco.prepare(
        "SELECT versao, usuario, unidade, modificacao, html_corpo
         FROM documento_revisao WHERE origem=?1 AND cenario=?2 AND id_documento=?3
         ORDER BY versao",
    )?;
    let linhas = stmt.query_map(params![origem, cenario, id_documento], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;
    let mut revisoes = Vec::new();
    for linha in linhas {
        let (versao, usuario, unidade, modificacao, html_corpo) = linha?;
        let numero_versao = u32::try_from(versao).map_err(|_| Falha::DadoInvalido)?;
        let versao = NonZeroU32::new(numero_versao).ok_or(Falha::DadoInvalido)?;
        revisoes.push(Revisao {
            versao,
            usuario,
            unidade,
            modificacao,
            html_corpo,
        });
    }
    if revisoes.is_empty() {
        return Err(Falha::DadoInvalido);
    }
    Ok(Some(DocumentoSimulado {
        instalacao: instalacao.clone(),
        id_documento: id_documento.to_owned(),
        id_procedimento,
        numero,
        tipo,
        revisoes,
    }))
}
