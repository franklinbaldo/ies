//! Estados e transições de processos no banco descartável do anti-IES.
//! A hipótese é sintética, não uma afirmação sobre o protocolo SEI real.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Destino {
    pub id: String,
    pub sigla: String,
    pub descricao: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessoSimulado {
    pub id: String,
    pub numero: String,
    pub unidade_origem: String,
    pub aberto_em: Vec<String>,
    pub anotacao_origem: Option<String>,
    pub destinos: Vec<Destino>,
}

#[derive(Clone, Copy, Debug)]
pub struct PedidoEnvio<'a> {
    pub id: &'a str,
    pub id_destino: &'a str,
    pub manter_aberto: bool,
    pub remover_anotacoes: bool,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    DadoInvalido,
    ProcessoNaoEncontrado,
    OrigemNaoAberta,
    DestinoNaoOferecido,
    DestinoJaAberto,
    AnotacaoNaoObservada,
}

impl From<rusqlite::Error> for Falha {
    fn from(falha: rusqlite::Error) -> Self {
        Self::Sqlite(falha)
    }
}

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())
}

fn validar(processo: &ProcessoSimulado) -> Result<(), Falha> {
    if !id_valido(&processo.id)
        || processo.numero.is_empty()
        || processo.unidade_origem.is_empty()
        || processo.aberto_em.iter().any(String::is_empty)
        || processo
            .destinos
            .iter()
            .any(|d| !id_valido(&d.id) || d.sigla.is_empty() || d.descricao.is_empty())
    {
        return Err(Falha::DadoInvalido);
    }
    for (i, unidade) in processo.aberto_em.iter().enumerate() {
        if processo.aberto_em[..i].contains(unidade) {
            return Err(Falha::DadoInvalido);
        }
    }
    for (i, destino) in processo.destinos.iter().enumerate() {
        if processo.destinos[..i]
            .iter()
            .any(|anterior| anterior.id == destino.id || anterior.sigla == destino.sigla)
        {
            return Err(Falha::DadoInvalido);
        }
    }
    Ok(())
}

/// Reconstrói um cenário com suas relações numa transação indivisível.
///
/// # Errors
/// Rejeita identidade duplicada/inválida ou preserva o erro SQLite.
pub fn guardar(
    banco: &mut Connection,
    instalacao: &Instalacao,
    cenario: &str,
    processo: &ProcessoSimulado,
) -> Result<(), Falha> {
    validar(processo)?;
    if cenario.is_empty() {
        return Err(Falha::DadoInvalido);
    }
    let origem = instalacao.url_base();
    let tx = banco.transaction()?;
    tx.execute(
        "INSERT INTO processo_simulado
         (origem, cenario, id_procedimento, numero, unidade_origem, anotacao_origem)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(origem, cenario, id_procedimento) DO UPDATE SET
         numero=excluded.numero, unidade_origem=excluded.unidade_origem,
         anotacao_origem=excluded.anotacao_origem",
        params![
            origem,
            cenario,
            processo.id,
            processo.numero,
            processo.unidade_origem,
            processo.anotacao_origem
        ],
    )?;
    tx.execute(
        "DELETE FROM processo_abertura
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem, cenario, processo.id],
    )?;
    tx.execute(
        "DELETE FROM processo_destino
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem, cenario, processo.id],
    )?;
    for (posicao, unidade) in processo.aberto_em.iter().enumerate() {
        let posicao = i64::try_from(posicao).map_err(|_| Falha::DadoInvalido)?;
        tx.execute(
            "INSERT INTO processo_abertura
             (origem, cenario, id_procedimento, posicao, unidade)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![origem, cenario, processo.id, posicao, unidade],
        )?;
    }
    for destino in &processo.destinos {
        tx.execute(
            "INSERT INTO processo_destino
             (origem, cenario, id_procedimento, id_unidade, sigla, descricao)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                origem,
                cenario,
                processo.id,
                destino.id,
                destino.sigla,
                destino.descricao
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Consulta a mesma fotografia que o servidor emprega em GET e POST.
///
/// # Errors
/// Propaga erros SQLite e estado estruturalmente inválido.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Option<ProcessoSimulado>, Falha> {
    let origem = instalacao.url_base();
    let cabecalho: Option<(String, String, Option<String>)> = banco
        .query_row(
            "SELECT numero, unidade_origem, anotacao_origem FROM processo_simulado
             WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
            params![origem, cenario, id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((numero, unidade_origem, anotacao_origem)) = cabecalho else {
        return Ok(None);
    };
    let mut aberturas = banco.prepare(
        "SELECT unidade FROM processo_abertura
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3
         ORDER BY posicao",
    )?;
    let aberto_em = aberturas
        .query_map(params![origem, cenario, id], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    let mut consulta = banco.prepare(
        "SELECT id_unidade, sigla, descricao FROM processo_destino
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 ORDER BY id_unidade",
    )?;
    let destinos = consulta
        .query_map(params![origem, cenario, id], |r| {
            Ok(Destino {
                id: r.get(0)?,
                sigla: r.get(1)?,
                descricao: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<Destino>>>()?;
    let processo = ProcessoSimulado {
        id: id.to_owned(),
        numero,
        unidade_origem,
        aberto_em,
        anotacao_origem,
        destinos,
    };
    validar(&processo)?;
    Ok(Some(processo))
}

/// Tramita uma vez; dados inválidos não produzem escrita parcial.
///
/// # Errors
/// Distingue processo ausente, origem já fechada, destino não oferecido,
/// destino já aberto, anotação não observada e erro SQLite real.
pub fn enviar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    pedido: PedidoEnvio<'_>,
) -> Result<ProcessoSimulado, Falha> {
    let tx = banco.unchecked_transaction()?;
    let mut processo =
        ler(&tx, instalacao, cenario, pedido.id)?.ok_or(Falha::ProcessoNaoEncontrado)?;
    if !processo.aberto_em.contains(&processo.unidade_origem) {
        return Err(Falha::OrigemNaoAberta);
    }
    let destino = processo
        .destinos
        .iter()
        .find(|d| d.id == pedido.id_destino)
        .ok_or(Falha::DestinoNaoOferecido)?;
    if processo.aberto_em.contains(&destino.sigla) {
        return Err(Falha::DestinoJaAberto);
    }
    if pedido.remover_anotacoes {
        let anotacao = processo
            .anotacao_origem
            .as_mut()
            .ok_or(Falha::AnotacaoNaoObservada)?;
        anotacao.clear();
    }
    let sigla = destino.sigla.clone();
    if !pedido.manter_aberto {
        processo
            .aberto_em
            .retain(|unidade| unidade != &processo.unidade_origem);
    }
    processo.aberto_em.push(sigla);
    let origem = instalacao.url_base();
    tx.execute(
        "UPDATE processo_simulado SET anotacao_origem=?4
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem, cenario, pedido.id, processo.anotacao_origem],
    )?;
    tx.execute(
        "DELETE FROM processo_abertura
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem, cenario, pedido.id],
    )?;
    for (i, unidade) in processo.aberto_em.iter().enumerate() {
        let i = i64::try_from(i).map_err(|_| Falha::DadoInvalido)?;
        tx.execute(
            "INSERT INTO processo_abertura
             (origem, cenario, id_procedimento, posicao, unidade)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![origem, cenario, pedido.id, i, unidade],
        )?;
    }
    tx.commit()?;
    Ok(processo)
}

/// Simula a alternativa de cadastro da anotação antes do envio.
///
/// # Errors
/// Preserva ausência da observação e erro SQLite.
pub fn limpar_anotacao(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<(), Falha> {
    let processo = ler(banco, instalacao, cenario, id)?.ok_or(Falha::ProcessoNaoEncontrado)?;
    if processo.anotacao_origem.is_none() {
        return Err(Falha::AnotacaoNaoObservada);
    }
    banco.execute(
        "UPDATE processo_simulado SET anotacao_origem='' 
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![instalacao.url_base(), cenario, id],
    )?;
    Ok(())
}

 
/// Lista os destinos oferecidos no cenário, compartilhados entre processos.
///
/// Um cenário inexistente não equivale a lista vazia. Uma lista vazia,
/// em contraste, pode representar um processo conhecido sem destinos.
///
/// # Errors
/// Preserva erros da leitura SQLite.
pub fn destinos_do_cenario(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<Option<Vec<Destino>>, Falha> {
    let origem = instalacao.url_base();
    let observado: i64 = banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM processo_simulado WHERE origem=?1 AND cenario=?2)",
        params![origem, cenario],
        |linha| linha.get(0),
    )?;
    if observado == 0 {
        return Ok(None);
    }
    let mut consulta = banco.prepare(
        "SELECT DISTINCT id_unidade, sigla, descricao FROM processo_destino
         WHERE origem=?1 AND cenario=?2 ORDER BY sigla, id_unidade",
    )?;
    let destinos = consulta
        .query_map(params![origem, cenario], |linha| {
            Ok(Destino {
                id: linha.get(0)?,
                sigla: linha.get(1)?,
                descricao: linha.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some(destinos))
}
