//! Estado tipado e compartilhado do simulador local de documento interno.
//! Hipótese sintética revisável; não contém cookies, URLs assinadas ou dados reais.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Processo {
    pub id: String,
    pub nup: String,
    pub unidade: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tipo {
    pub id: String,
    pub nome: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Documento {
    pub id: i64,
    pub numero_sei: String,
    pub tipo: String,
    pub descricao: String,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    EntradaInvalida,
    TipoIndisponivel,
    ProcessoIndisponivel,
    ProcessoNaoAbertoNaUnidade,
}

impl From<rusqlite::Error> for Falha {
    fn from(causa: rusqlite::Error) -> Self {
        Self::Sqlite(causa)
    }
}

fn escopo_valido(cenario: &str, id: &str) -> bool {
    !cenario.is_empty() && !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())
}

/// Registra processo sintético em um SQLite reconstruível.
///
/// # Errors
/// Rejeita identificadores inválidos ou propaga o erro SQLite.
pub fn registrar_processo(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    processo: &Processo,
) -> Result<(), Falha> {
    if !escopo_valido(cenario, &processo.id)
        || processo.nup.is_empty()
        || processo.unidade.is_empty()
    {
        return Err(Falha::EntradaInvalida);
    }
    banco.execute(
        "INSERT INTO sim_processo (origem, cenario, id_procedimento, nup, unidade)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(origem, cenario, id_procedimento)
         DO UPDATE SET nup=excluded.nup, unidade=excluded.unidade",
        params![
            origem.url_base(),
            cenario,
            processo.id,
            processo.nup,
            processo.unidade
        ],
    )?;
    Ok(())
}

/// Registra um tipo selecionável para o processo simulado.
///
/// # Errors
/// Rejeita identificadores inválidos ou propaga a causa SQLite.
pub fn registrar_tipo(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
    tipo: &Tipo,
) -> Result<(), Falha> {
    if !escopo_valido(cenario, id_procedimento)
        || !escopo_valido(cenario, &tipo.id)
        || tipo.nome.trim().is_empty()
    {
        return Err(Falha::EntradaInvalida);
    }
    banco.execute(
        "INSERT INTO sim_tipo (origem, cenario, id_procedimento, id_serie, nome)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(origem, cenario, id_procedimento, id_serie)
         DO UPDATE SET nome=excluded.nome",
        params![
            origem.url_base(),
            cenario,
            id_procedimento,
            tipo.id,
            tipo.nome
        ],
    )?;
    Ok(())
}

/// Ausência de processo não cria uma árvore fictícia.
///
/// # Errors
/// Propaga a causa original SQLite.
pub fn ler_processo(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> rusqlite::Result<Option<Processo>> {
    banco
        .query_row(
            "SELECT id_procedimento, nup, unidade FROM sim_processo
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
            params![origem.url_base(), cenario, id],
            |r| {
                Ok(Processo {
                    id: r.get(0)?,
                    nup: r.get(1)?,
                    unidade: r.get(2)?,
                })
            },
        )
        .optional()
}

/// Lista oferecida pelo modelo, inclusive nomes duplicados para testar ambiguidade.
///
/// # Errors
/// Propaga a causa SQLite.
pub fn tipos(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
) -> rusqlite::Result<Vec<Tipo>> {
    let mut stmt = banco.prepare(
        "SELECT id_serie, nome FROM sim_tipo WHERE origem=?1 AND cenario=?2
         AND id_procedimento=?3 ORDER BY id_serie",
    )?;
    stmt.query_map(params![origem.url_base(), cenario, id_procedimento], |r| {
        Ok(Tipo {
            id: r.get(0)?,
            nome: r.get(1)?,
        })
    })?
    .collect()
}

/// Pesquisa NUP exato no estado compartilhado do simulador.
/// Não cria processo nem resolve arbitrariamente números duplicados.
///
/// # Errors
/// Propaga a falha original do SQLite.
pub fn ids_por_nup(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    nup: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut consulta = banco.prepare(
        "SELECT id_procedimento FROM sim_processo
         WHERE origem=?1 AND cenario=?2 AND nup=?3 ORDER BY id_procedimento",
    )?;
    consulta
        .query_map(params![origem.url_base(), cenario, nup], |linha| {
            linha.get(0)
        })?
        .collect()
}

/// Confere a abertura na unidade ativa quando o processo possui estado de
/// tramitação. Cenas antigas sem a projeção de envio continuam independentes.
///
/// # Errors
/// Retorna a causa tipada de processo ausente/fechado ou o erro SQLite.
pub fn conferir_abertura(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
) -> Result<(), Falha> {
    let processo = ler_processo(banco, origem, cenario, id_procedimento)?
        .ok_or(Falha::ProcessoIndisponivel)?;
    let (tem_tramitacao, aberto): (bool, bool) = banco.query_row(
        "SELECT
           EXISTS(SELECT 1 FROM processo_simulado
                  WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3),
           EXISTS(SELECT 1 FROM processo_abertura
                  WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4)",
        params![
            origem.url_base(),
            cenario,
            id_procedimento,
            processo.unidade
        ],
        |linha| Ok((linha.get(0)?, linha.get(1)?)),
    )?;
    if tem_tramitacao && !aberto {
        return Err(Falha::ProcessoNaoAbertoNaUnidade);
    }
    Ok(())
}

/// Fotografia independente dos documentos que existem no estado simulado.
///
/// # Errors
/// Propaga a causa SQLite.
pub fn documentos(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
) -> rusqlite::Result<Vec<Documento>> {
    let mut stmt = banco.prepare(
        "SELECT d.id_documento, t.nome, d.descricao
         FROM sim_documento d JOIN sim_tipo t ON
          (t.origem=d.origem AND t.cenario=d.cenario
           AND t.id_procedimento=d.id_procedimento AND t.id_serie=d.id_serie)
         WHERE d.origem=?1 AND d.cenario=?2 AND d.id_procedimento=?3
         ORDER BY d.id_documento",
    )?;
    stmt.query_map(params![origem.url_base(), cenario, id_procedimento], |r| {
        let id: i64 = r.get(0)?;
        Ok(Documento {
            id,
            numero_sei: format!("{:07}", 1_000_000 + id),
            tipo: r.get(1)?,
            descricao: r.get(2)?,
        })
    })?
    .collect()
}

/// Uma chamada equivale a uma única escrita. Não há retry ou sucesso presumido.
///
/// # Errors
/// Recusa série não oferecida e propaga a causa SQLite.
pub fn criar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
    id_serie: &str,
    descricao: &str,
) -> Result<Documento, Falha> {
    if !escopo_valido(cenario, id_procedimento) || !escopo_valido(cenario, id_serie) {
        return Err(Falha::EntradaInvalida);
    }
    conferir_abertura(banco, origem, cenario, id_procedimento)?;
    // Criação e cancelamento precisam compartilhar a identidade documental.
    // Apenas a identidade é conhecida aqui: não fabricar revisão, autor ou HTML.
    // A transação impede que um cadastro apareça numa árvore e falte na outra.
    let transacao = banco.unchecked_transaction()?;
    let quantidade = transacao.execute(
        "INSERT INTO sim_documento (origem, cenario, id_procedimento, id_serie, descricao, nivel)
         SELECT origem, cenario, id_procedimento, id_serie, ?5, 0
         FROM sim_tipo WHERE origem=?1 AND cenario=?2
           AND id_procedimento=?3 AND id_serie=?4",
        params![
            origem.url_base(),
            cenario,
            id_procedimento,
            id_serie,
            descricao
        ],
    )?;
    if quantidade != 1 {
        return Err(Falha::TipoIndisponivel);
    }
    let id = transacao.last_insert_rowid();
    let numero_sei = format!("{:07}", 1_000_000 + id);
    let tipo: String = transacao.query_row(
        "SELECT nome FROM sim_tipo WHERE origem=?1 AND cenario=?2
         AND id_procedimento=?3 AND id_serie=?4",
        params![origem.url_base(), cenario, id_procedimento, id_serie],
        |r| r.get(0),
    )?;
    transacao.execute(
        "INSERT INTO documento_simulado
            (origem, cenario, id_documento, id_procedimento, numero, tipo)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            origem.url_base(),
            cenario,
            id.to_string(),
            id_procedimento,
            numero_sei,
            tipo,
        ],
    )?;
    transacao.commit()?;
    Ok(Documento {
        id,
        numero_sei,
        tipo,
        descricao: descricao.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banco_cassettes::iniciar;

    #[test]
    fn escrita_de_documento_altera_uma_fotografia_sem_tocar_outros_cenarios() {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        let processo = Processo {
            id: "10".into(),
            nup: "00001.000001/2026-01".into(),
            unidade: "U1".into(),
        };
        registrar_processo(&banco, &origem, "bdd-246", &processo).unwrap();
        registrar_tipo(
            &banco,
            &origem,
            "bdd-246",
            "10",
            &Tipo {
                id: "291".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
        assert_eq!(
            documentos(&banco, &origem, "bdd-246", "10").unwrap(),
            Vec::<Documento>::new()
        );
        let criado = criar(&banco, &origem, "bdd-246", "10", "291", "Teste").unwrap();
        assert_eq!(
            documentos(&banco, &origem, "bdd-246", "10").unwrap(),
            vec![criado]
        );
        assert_eq!(
            documentos(&banco, &origem, "outro", "10").unwrap(),
            Vec::<Documento>::new()
        );
        assert!(
            ler_processo(&banco, &origem, "outro", "10")
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            criar(&banco, &origem, "bdd-246", "10", "999", "X"),
            Err(Falha::TipoIndisponivel)
        ));
        assert_eq!(
            documentos(&banco, &origem, "bdd-246", "10").unwrap().len(),
            1
        );
    }

    #[test]
    fn esquema_descartavel_e_idempotente_por_identidade_da_serie() {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        assert_eq!(
            banco
                .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            32
        );
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        let tipo = Tipo {
            id: "291".into(),
            nome: "Despacho".into(),
        };
        registrar_tipo(&banco, &origem, "c", "10", &tipo).unwrap();
        registrar_tipo(&banco, &origem, "c", "10", &tipo).unwrap();
        assert_eq!(tipos(&banco, &origem, "c", "10").unwrap(), vec![tipo]);
        // Uma série órfã não permite criar documento sem processo existente.
        assert!(matches!(
            criar(&banco, &origem, "c", "10", "291", "Sem processo"),
            Err(Falha::ProcessoIndisponivel)
        ));
        assert_eq!(documentos(&banco, &origem, "c", "10").unwrap(), []);
    }
}
