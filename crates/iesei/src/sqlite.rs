//! Projeção relacional observada do Controle: dados, nunca sessão ou HTML.
//! A abertura segura e a transação pertencem ao consumidor.
use crate::instalacao::Instalacao;
use crate::observacao::{
    Cobertura, FaixaObservada, FormaDoControle, ObservacaoDoControle, ProcessoNaLista,
    Proveniencia, TabelaObservada, UnidadeExibida,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

/// Migração incremental do índice privado do IES; sem dados de sessão.
/// Chaves de origem e contexto impedem mistura de perfis.
pub const MIGRACAO_CONTROLE: &str = "
CREATE TABLE controle_observado (
  origem TEXT NOT NULL,
  contexto TEXT NOT NULL,
  unidade TEXT NOT NULL,
  visao TEXT NOT NULL CHECK(visao IN ('R','D')),
  fonte_tipo TEXT NOT NULL CHECK(fonte_tipo IN ('cassette','web')),
  fonte TEXT NOT NULL,
  PRIMARY KEY(origem, contexto)
) STRICT;
CREATE TABLE controle_tabela (
  origem TEXT NOT NULL,
  contexto TEXT NOT NULL,
  categoria TEXT NOT NULL CHECK(categoria IN ('R','G','D')),
  total INTEGER CHECK(total >= 0),
  inicio INTEGER CHECK(inicio > 0),
  fim INTEGER CHECK(fim >= inicio),
  PRIMARY KEY(origem, contexto, categoria),
  FOREIGN KEY(origem, contexto) REFERENCES controle_observado(origem, contexto)
) STRICT;
CREATE TABLE controle_linha (
  origem TEXT NOT NULL,
  contexto TEXT NOT NULL,
  categoria TEXT NOT NULL CHECK(categoria IN ('R','G','D')),
  posicao INTEGER NOT NULL CHECK(posicao >= 0),
  id TEXT NOT NULL,
  numero TEXT NOT NULL,
  PRIMARY KEY(origem, contexto, categoria, posicao),
  FOREIGN KEY(origem, contexto, categoria)
    REFERENCES controle_tabela(origem, contexto, categoria)
) STRICT;";

fn numero(valor: usize) -> rusqlite::Result<i64> {
    i64::try_from(valor).map_err(|_| rusqlite::Error::InvalidQuery)
}

fn indice(valor: i64) -> rusqlite::Result<usize> {
    usize::try_from(valor).map_err(|_| rusqlite::Error::InvalidQuery)
}

fn gravar_tabela(
    tx: &Transaction<'_>,
    origem: &str,
    contexto: &str,
    categoria: &str,
    cobertura: &Cobertura<TabelaObservada<ProcessoNaLista>>,
) -> rusqlite::Result<()> {
    let Cobertura::Observada(tabela) = cobertura else {
        return Ok(());
    };
    tx.execute(
        "INSERT INTO controle_tabela(origem,contexto,categoria,total,inicio,fim)
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            origem,
            contexto,
            categoria,
            tabela.total_declarado.map(numero).transpose()?,
            tabela.faixa_observada.map(|f| numero(f.inicio())).transpose()?,
            tabela.faixa_observada.map(|f| numero(f.fim())).transpose()?,
        ],
    )?;
    let mut stmt = tx.prepare(
        "INSERT INTO controle_linha(origem,contexto,categoria,posicao,id,numero)
         VALUES (?1,?2,?3,?4,?5,?6)",
    )?;
    for (i, linha) in tabela.linhas.iter().enumerate() {
        stmt.execute(params![
            origem, contexto, categoria, numero(i)?, linha.id, linha.numero
        ])?;
    }
    Ok(())
}

/// Insere um snapshot atômico na transação **do IES**, sem conexão HTTP.
/// Em observação idêntica, não regrava dados nem incrementa changes.
///
/// # Errors
///
/// Retorna o erro original SQLite para falha de banco, esquema ou valores.
pub fn registrar(
    tx: &Transaction<'_>,
    contexto: &str,
    observacao: &ObservacaoDoControle<ProcessoNaLista>,
) -> rusqlite::Result<()> {
    if contexto.is_empty() {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let origem = observacao.instalacao.url_base();
    if consultar(tx, &origem, contexto)?.as_ref() == Some(observacao) {
        return Ok(());
    }
    let (visao, fontes) = match &observacao.forma {
        FormaDoControle::Resumida { recebidos, gerados } => (
            "R",
            vec![("R", recebidos), ("G", gerados)],
        ),
        FormaDoControle::Detalhada { processos } => ("D", vec![("D", processos)]),
    };
    let (tipo, fonte) = match &observacao.proveniencia {
        Proveniencia::Cassette { cartao } => ("cassette", cartao),
        Proveniencia::LeituraWeb { operacao } => ("web", operacao),
    };
    tx.execute(
        "INSERT INTO controle_observado(origem,contexto,unidade,visao,fonte_tipo,fonte)
         VALUES (?1,?2,?3,?4,?5,?6)
         ON CONFLICT(origem,contexto) DO UPDATE SET
         unidade=excluded.unidade,visao=excluded.visao,
         fonte_tipo=excluded.fonte_tipo,fonte=excluded.fonte",
        params![origem, contexto, observacao.unidade.as_str(), visao, tipo, fonte],
    )?;
    tx.execute(
        "DELETE FROM controle_linha WHERE origem=?1 AND contexto=?2",
        params![origem, contexto],
    )?;
    tx.execute(
        "DELETE FROM controle_tabela WHERE origem=?1 AND contexto=?2",
        params![origem, contexto],
    )?;
    for (categoria, tabela) in fontes {
        gravar_tabela(tx, &origem, contexto, categoria, tabela)?;
    }
    Ok(())
}

fn ler_tabela(
    con: &Connection,
    origem: &str,
    contexto: &str,
    categoria: &str,
) -> rusqlite::Result<Cobertura<TabelaObservada<ProcessoNaLista>>> {
    let cabecalho: Option<(Option<i64>, Option<i64>, Option<i64>)> = con
        .query_row(
            "SELECT total,inicio,fim FROM controle_tabela
             WHERE origem=?1 AND contexto=?2 AND categoria=?3",
            params![origem, contexto, categoria],
            |l| Ok((l.get(0)?, l.get(1)?, l.get(2)?)),
        )
        .optional()?;
    let Some((total, inicio, fim)) = cabecalho else {
        return Ok(Cobertura::NaoObservada);
    };
    let faixa = match (inicio, fim) {
        (Some(i), Some(f)) => Some(
            FaixaObservada::nova(indice(i)?, indice(f)?)
                .ok_or(rusqlite::Error::InvalidQuery)?,
        ),
        (None, None) => None,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    let mut stmt = con.prepare(
        "SELECT id,numero FROM controle_linha
         WHERE origem=?1 AND contexto=?2 AND categoria=?3 ORDER BY posicao",
    )?;
    let linhas: Vec<ProcessoNaLista> = stmt
        .query_map(params![origem, contexto, categoria], |l| {
            let id: String = l.get(0)?;
            let nup: String = l.get(1)?;
            ProcessoNaLista::novo(&id, &nup).ok_or(rusqlite::Error::InvalidQuery)
        })?
        .collect::<rusqlite::Result<_>>()?;
    let tabela = TabelaObservada::nova(total.map(indice).transpose()?, faixa, linhas)
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
    Ok(Cobertura::Observada(tabela))
}

/// Lê somente os fatos de domínio, sem tocar em cookies e links assinados.
/// Três a quatro SELECTs por snapshot; nenhuma consulta por linha.
///
/// # Errors
///
/// Propaga erro SQLite ou indica registro incoerente com InvalidQuery.
pub fn consultar(
    con: &Connection,
    origem: &str,
    contexto: &str,
) -> rusqlite::Result<Option<ObservacaoDoControle<ProcessoNaLista>>> {
    let cabecalho: Option<(String, String, String, String)> = con
        .query_row(
            "SELECT unidade,visao,fonte_tipo,fonte FROM controle_observado
             WHERE origem=?1 AND contexto=?2",
            params![origem, contexto],
            |l| Ok((l.get(0)?, l.get(1)?, l.get(2)?, l.get(3)?)),
        )
        .optional()?;
    let Some((unidade, visao, tipo, fonte)) = cabecalho else {
        return Ok(None);
    };
    let instalacao = Instalacao::interpretar(origem).ok_or(rusqlite::Error::InvalidQuery)?;
    let unidade = UnidadeExibida::nova(&unidade).ok_or(rusqlite::Error::InvalidQuery)?;
    let proveniencia = match tipo.as_str() {
        "cassette" => Proveniencia::Cassette { cartao: fonte },
        "web" => Proveniencia::LeituraWeb { operacao: fonte },
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    let forma = match visao.as_str() {
        "R" => FormaDoControle::Resumida {
            recebidos: ler_tabela(con, origem, contexto, "R")?,
            gerados: ler_tabela(con, origem, contexto, "G")?,
        },
        "D" => FormaDoControle::Detalhada {
            processos: ler_tabela(con, origem, contexto, "D")?,
        },
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(Some(ObservacaoDoControle {
        instalacao,
        unidade,
        forma,
        proveniencia,
    }))
}
