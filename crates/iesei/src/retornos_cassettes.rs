//! Fatos mensais observados no acervo; nunca uma sessão ou URLs do SEI.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Situacao {
    Aguardando,
    Devolver,
}

impl Situacao {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Aguardando => "aguardando",
            Self::Devolver => "devolver",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Retorno {
    pub situacao: Situacao,
    pub numero: String,
    pub unidade: String,
    pub data_programada: String,
    pub retornado_em: Option<String>,
    pub prazo_restante: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mes {
    pub instalacao: Instalacao,
    pub data_inicial: String,
    pub data_final: String,
    pub itens: Vec<Retorno>,
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

/// Persiste uma listagem mensal já conferida pelo leitor de produção do IES.
/// A ausência de uma listagem nunca é convertida em lista vazia.
///
/// # Errors
/// Preserva erro SQLite e rejeita campos obrigatórios ausentes.
pub fn guardar(banco: &mut Connection, cenario: &str, mes: &Mes) -> Result<(), Falha> {
    if cenario.is_empty() || mes.data_inicial.is_empty() || mes.data_final.is_empty()
        || mes.itens.iter().any(|r| {
            r.numero.is_empty() || r.unidade.is_empty() || r.data_programada.is_empty()
        })
    {
        return Err(Falha::DadoInvalido);
    }
    let origem = mes.instalacao.url_base();
    let tx = banco.transaction()?;
    tx.execute(
        "INSERT INTO retorno_mes (origem, cenario, data_inicial, data_final)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (origem, cenario, data_inicial, data_final) DO NOTHING",
        params![origem, cenario, mes.data_inicial, mes.data_final],
    )?;
    tx.execute(
        "DELETE FROM retorno_linha
         WHERE origem=?1 AND cenario=?2 AND data_inicial=?3 AND data_final=?4",
        params![origem, cenario, mes.data_inicial, mes.data_final],
    )?;
    for (posicao, item) in mes.itens.iter().enumerate() {
        let posicao = i64::try_from(posicao).map_err(|_| Falha::DadoInvalido)?;
        tx.execute(
            "INSERT INTO retorno_linha
             (origem, cenario, data_inicial, data_final, posicao,
              situacao, numero, unidade, data_programada, retornado_em, prazo_restante)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                origem, cenario, mes.data_inicial, mes.data_final, posicao,
                item.situacao.as_str(), item.numero, item.unidade,
                item.data_programada, item.retornado_em, item.prazo_restante
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Lê apenas um mês explicitamente observado; nenhum registro significa
/// None se o mês não foi capturado, Some(vec![]) se o vazio foi comprovado.
///
/// # Errors
/// Preserva a causa SQLite, ou uma situação inválida encontrada na projeção.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    data_inicial: &str,
    data_final: &str,
) -> Result<Option<Mes>, Falha> {
    let origem = instalacao.url_base();
    let conhecido: Option<i64> = banco
        .query_row(
            "SELECT 1 FROM retorno_mes
             WHERE origem=?1 AND cenario=?2 AND data_inicial=?3 AND data_final=?4",
            params![origem, cenario, data_inicial, data_final],
            |r| r.get(0),
        )
        .optional()?;
    if conhecido.is_none() {
        return Ok(None);
    }
    let mut consulta = banco.prepare(
        "SELECT situacao, numero, unidade, data_programada, retornado_em, prazo_restante
         FROM retorno_linha WHERE origem=?1 AND cenario=?2
           AND data_inicial=?3 AND data_final=?4 ORDER BY posicao",
    )?;
    let resultados = consulta.query_map(
        params![origem, cenario, data_inicial, data_final],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
            ))
        },
    )?;
    let mut itens = Vec::new();
    for item in resultados {
        let (situacao, numero, unidade, data_programada, retornado_em, prazo_restante) = item?;
        let situacao = match situacao.as_str() {
            "aguardando" => Situacao::Aguardando,
            "devolver" => Situacao::Devolver,
            _ => return Err(Falha::DadoInvalido),
        };
        itens.push(Retorno {
            situacao,
            numero,
            unidade,
            data_programada,
            retornado_em,
            prazo_restante,
        });
    }
    Ok(Some(Mes {
        instalacao: instalacao.clone(),
        data_inicial: data_inicial.to_owned(),
        data_final: data_final.to_owned(),
        itens,
    }))
}
