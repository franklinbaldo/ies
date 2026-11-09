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
    pub numero_processo: String,
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
        || !valido(&documento.numero_processo)
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
        "INSERT INTO documento_simulado
         (origem, cenario, id_documento, id_procedimento, numero_processo, numero, tipo)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(origem, cenario, id_documento) DO UPDATE SET
         id_procedimento=excluded.id_procedimento,
         numero_processo=excluded.numero_processo,
         numero=excluded.numero, tipo=excluded.tipo",
        params![
            origem,
            cenario,
            documento.id_documento,
            documento.id_procedimento,
            documento.numero_processo,
            documento.numero,
            documento.tipo
        ],
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
    let cabecalho: Option<(String, String, String, String)> = banco
        .query_row(
            "SELECT id_procedimento, numero_processo, numero, tipo FROM documento_simulado
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3",
            params![origem, cenario, id_documento],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let Some((id_procedimento, numero_processo, numero, tipo)) = cabecalho else {
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
        numero_processo,
        numero,
        tipo,
        revisoes,
    }))
}

/// Pesquisa pelo numero SEI no mesmo estado das revisoes, sem inferir
/// versoes ou escolher silenciosamente entre documentos ambiguos.
///
/// # Errors
/// Preserva a falha de SQLite.
pub fn ids_por_numero(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    numero: &str,
) -> Result<Vec<String>, Falha> {
    let mut consulta = banco.prepare(
        "SELECT id_documento FROM documento_simulado
         WHERE origem=?1 AND cenario=?2 AND numero=?3
         ORDER BY id_documento",
    )?;
    let linhas = consulta.query_map(params![instalacao.url_base(), cenario, numero], |linha| {
        linha.get::<_, String>(0)
    })?;
    let mut encontrados = Vec::new();
    for linha in linhas {
        encontrados.push(linha?);
    }
    Ok(encontrados)
}

/// Lista documentos com revisão existente no processo e cenário.
/// A identidade de um documento recém-criado não inventa versão nem corpo.
///
/// # Errors
/// Propaga a falha original do SQLite.
pub fn ids_do_processo(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut stmt = banco.prepare(
        "SELECT d.id_documento FROM documento_simulado d
         WHERE d.origem=?1 AND d.cenario=?2 AND d.id_procedimento=?3
         AND EXISTS(
             SELECT 1 FROM documento_revisao r
             WHERE r.origem=d.origem AND r.cenario=d.cenario
               AND r.id_documento=d.id_documento
         )
         ORDER BY d.id_documento",
    )?;
    let linhas = stmt.query_map(
        params![instalacao.url_base(), cenario, id_procedimento],
        |linha| linha.get::<_, String>(0),
    )?;
    linhas.collect()
}

#[cfg(test)]
mod testes_criacao {
    use super::*;
    use crate::banco_cassettes::iniciar;
    use crate::documentos_simulados::{self, Processo, Tipo};

    #[test]
    fn criado_nao_inventa_revisao_mas_passa_a_ser_versionado_apos_edicao() {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        let cena = "criar-depois-editar";
        documentos_simulados::registrar_processo(
            &banco,
            &origem,
            cena,
            &Processo {
                id: "10".into(),
                nup: "00001.000001/2026-01".into(),
                unidade: "TESTE".into(),
            },
        )
        .unwrap();
        documentos_simulados::registrar_tipo(
            &banco,
            &origem,
            cena,
            "10",
            &Tipo {
                id: "291".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
        let criado = documentos_simulados::criar(&banco, &origem, cena, "10", "291", "").unwrap();
        assert_eq!(ids_do_processo(&banco, &origem, cena, "10").unwrap(), []);
        guardar(
            &mut banco,
            cena,
            &DocumentoSimulado {
                instalacao: origem.clone(),
                id_documento: criado.id.to_string(),
                id_procedimento: "10".into(),
                numero_processo: "00001.000001/2026-01".into(),
                numero: criado.numero_sei,
                tipo: criado.tipo,
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "TESTE".into(),
                    unidade: "TESTE".into(),
                    modificacao: "09/11/2026".into(),
                    html_corpo: "<p>Conteúdo sintético</p>".into(),
                }],
            },
        )
        .unwrap();
        assert_eq!(
            ids_do_processo(&banco, &origem, cena, "10").unwrap(),
            vec![criado.id.to_string()]
        );
    }
}
