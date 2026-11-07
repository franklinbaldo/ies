//! Banco efêmero derivado das cassettes, separado de ies.sqlite3.
use crate::instalacao::Instalacao;
use crate::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use rusqlite::{Connection, OptionalExtension, params};

pub const ESQUEMA: &str = include_str!("../migrations/0001_controle_resumido.sql");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Linha {
    pub id: String,
    pub numero: String,
    pub visualizado: bool,
    pub sigiloso: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControleResumido {
    pub instalacao: Instalacao,
    pub unidade: UnidadeExibida,
    pub recebidos: TabelaObservada<Linha>,
    pub gerados: TabelaObservada<Linha>,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    ObservacaoInsuficiente,
    DadoInvalido,
}

impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}

/// Cria o esquema da sessão de testes; nunca migra um banco operacional.
///
/// # Errors
/// Propaga a falha de SQLite.
pub fn iniciar(banco: &Connection) -> rusqlite::Result<()> {
    banco.execute_batch(ESQUEMA)
}

fn numero_sql(numero: usize) -> Result<i64, Falha> {
    i64::try_from(numero).map_err(|_| Falha::DadoInvalido)
}

fn metadados(
    tabela: &TabelaObservada<Linha>,
) -> Result<(Option<i64>, Option<i64>, Option<i64>), Falha> {
    if !tabela.pagina_verificavel() {
        return Err(Falha::ObservacaoInsuficiente);
    }
    let total = tabela.total_declarado.map(numero_sql).transpose()?;
    let inicio = tabela.faixa_observada.map(|f| numero_sql(f.inicio())).transpose()?;
    let fim = tabela.faixa_observada.map(|f| numero_sql(f.fim())).transpose()?;
    Ok((total, inicio, fim))
}

/// Guarda somente o Controle R cujo conteúdo o IES realmente leu.
///
/// # Errors
/// Recusa páginas insuficientes, IDs inválidos e erros SQL.
pub fn guardar(
    banco: &mut Connection,
    cenario: &str,
    controle: &ControleResumido,
) -> Result<(), Falha> {
    if cenario.is_empty() {
        return Err(Falha::DadoInvalido);
    }
    let rec = metadados(&controle.recebidos)?;
    let ger = metadados(&controle.gerados)?;
    for linha in controle
        .recebidos
        .linhas
        .iter()
        .chain(&controle.gerados.linhas)
    {
        if linha.id.is_empty()
            || !linha.id.bytes().all(|b| b.is_ascii_digit())
            || linha.numero.is_empty()
        {
            return Err(Falha::DadoInvalido);
        }
    }
    let origem = controle.instalacao.url_base();
    let transacao = banco.transaction()?;
    transacao.execute(
        "INSERT INTO controle_resumido
         (origem, cenario, unidade, recebidos_total, recebidos_inicio,
          recebidos_fim, gerados_total, gerados_inicio, gerados_fim)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(origem, cenario) DO UPDATE SET
         unidade=excluded.unidade,
         recebidos_total=excluded.recebidos_total,
         recebidos_inicio=excluded.recebidos_inicio,
         recebidos_fim=excluded.recebidos_fim,
         gerados_total=excluded.gerados_total,
         gerados_inicio=excluded.gerados_inicio,
         gerados_fim=excluded.gerados_fim",
        params![
            origem,
            cenario,
            controle.unidade.as_str(),
            rec.0,
            rec.1,
            rec.2,
            ger.0,
            ger.1,
            ger.2
        ],
    )?;
    transacao.execute(
        "DELETE FROM controle_linha WHERE origem=?1 AND cenario=?2",
        params![origem, cenario],
    )?;
    for (categoria, tabela) in [
        ("recebidos", &controle.recebidos),
        ("gerados", &controle.gerados),
    ] {
        for (posicao, linha) in tabela.linhas.iter().enumerate() {
            transacao.execute(
                "INSERT INTO controle_linha
                 (origem, cenario, categoria, posicao, id, numero,
                  visualizado, sigiloso) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    origem,
                    cenario,
                    categoria,
                    numero_sql(posicao)?,
                    linha.id,
                    linha.numero,
                    linha.visualizado,
                    linha.sigiloso
                ],
            )?;
        }
    }
    transacao.commit()?;
    Ok(())
}

fn positivo(numero: Option<i64>) -> Result<Option<usize>, Falha> {
    numero
        .map(|n| usize::try_from(n).map_err(|_| Falha::DadoInvalido))
        .transpose()
}

fn tabela(
    total: Option<i64>,
    inicio: Option<i64>,
    fim: Option<i64>,
    linhas: Vec<Linha>,
) -> Result<TabelaObservada<Linha>, Falha> {
    let faixa = match (positivo(inicio)?, positivo(fim)?) {
        (None, None) => None,
        (Some(i), Some(f)) => Some(FaixaObservada::nova(i, f).ok_or(Falha::DadoInvalido)?),
        _ => return Err(Falha::DadoInvalido),
    };
    let tabela = TabelaObservada::nova(positivo(total)?, faixa, linhas)
        .map_err(|_| Falha::DadoInvalido)?;
    if !tabela.pagina_verificavel() {
        return Err(Falha::ObservacaoInsuficiente);
    }
    Ok(tabela)
}

/// Um SELECT de metadados e um SELECT de linhas, sem N+1.
/// Ausência de snapshot não significa unidade ou lista vazia.
///
/// # Errors
/// Propaga erros SQLite e rejeita estado malformado sem inferências.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<Option<ControleResumido>, Falha> {
    let origem = instalacao.url_base();
    type Metadados = (
        String,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
    );
    let dados: Option<Metadados> = banco
        .query_row(
            "SELECT unidade, recebidos_total, recebidos_inicio, recebidos_fim,
                    gerados_total, gerados_inicio, gerados_fim
             FROM controle_resumido WHERE origem=?1 AND cenario=?2",
            params![origem, cenario],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            },
        )
        .optional()?;
    let Some((unidade, rt, ri, rf, gt, gi, gf)) = dados else {
        return Ok(None);
    };
    let unidade = UnidadeExibida::nova(&unidade).ok_or(Falha::DadoInvalido)?;
    let mut recebidos = Vec::new();
    let mut gerados = Vec::new();
    let mut consulta = banco.prepare(
        "SELECT categoria, id, numero, visualizado, sigiloso
         FROM controle_linha WHERE origem=?1 AND cenario=?2
         ORDER BY categoria, posicao",
    )?;
    let rows = consulta.query_map(params![origem, cenario], |r| {
        Ok((
            r.get::<_, String>(0)?,
            Linha {
                id: r.get(1)?,
                numero: r.get(2)?,
                visualizado: r.get(3)?,
                sigiloso: r.get(4)?,
            },
        ))
    })?;
    for item in rows {
        let (categoria, linha) = item?;
        match categoria.as_str() {
            "recebidos" => recebidos.push(linha),
            "gerados" => gerados.push(linha),
            _ => return Err(Falha::DadoInvalido),
        }
    }
    Ok(Some(ControleResumido {
        instalacao: instalacao.clone(),
        unidade,
        recebidos: tabela(rt, ri, rf, recebidos)?,
        gerados: tabela(gt, gi, gf, gerados)?,
    }))
}
