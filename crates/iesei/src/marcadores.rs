//! Marcadores observados na unidade, como dados tipados do SQLite efêmero.
 //! Hipótese para BDD-255-03. Não representa uma submissão Web nem seu aceite.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marcador {
    pub id: String,
    pub nome: String,
    pub cor: String,
    pub texto: Option<String>,
    pub usuario: Option<String>,
    pub data_hora: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observacao {
    pub total_declarado: usize,
    pub itens: Vec<Marcador>,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    DadoInvalido,
    ProcessoNaoObservado,
}

impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}

fn valido(observacao: &Observacao) -> bool {
    if observacao.total_declarado != observacao.itens.len() {
        return false;
    }
    for (i, marcador) in observacao.itens.iter().enumerate() {
        if marcador.id.is_empty()
            || !marcador.id.bytes().all(|b| b.is_ascii_digit())
            || marcador.nome.is_empty()
            || marcador.cor.is_empty()
            || !marcador.cor.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
            || marcador.data_hora.is_empty()
            || observacao.itens[..i].iter().any(|outro| outro.id == marcador.id)
        {
            return false;
        }
    }
    true
}

/// Registra apenas uma observação já conhecida da unidade, de modo atômico.
/// A observação vazia é explícita, não inferida da falta de dados.
///
/// # Errors
/// Rejeita entrada inválida ou processo não aberto e preserva SQLite.
pub fn registrar(
    banco: &mut Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
    observacao: &Observacao,
) -> Result<(), Falha> {
    if cenario.is_empty() || id.is_empty()
        || !id.bytes().all(|b| b.is_ascii_digit())
        || unidade.is_empty() || !valido(observacao)
    {
        return Err(Falha::DadoInvalido);
    }
    let tx = banco.transaction()?;
    let aberto: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM processo_abertura
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4)",
        params![origem.url_base(), cenario, id, unidade],
        |linha| linha.get(0),
    )?;
    if !aberto {
        return Err(Falha::ProcessoNaoObservado);
    }
    let total = i64::try_from(observacao.total_declarado).map_err(|_| Falha::DadoInvalido)?;
    tx.execute(
        "INSERT INTO processo_marcadores_observados
         (origem, cenario, id_procedimento, unidade, total_declarado)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (origem, cenario, id_procedimento, unidade)
         DO UPDATE SET total_declarado=excluded.total_declarado",
        params![origem.url_base(), cenario, id, unidade, total],
    )?;
    tx.execute(
        "DELETE FROM processo_marcador_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4",
        params![origem.url_base(), cenario, id, unidade],
    )?;
    for (posicao, marcador) in observacao.itens.iter().enumerate() {
        let posicao = i64::try_from(posicao).map_err(|_| Falha::DadoInvalido)?;
        tx.execute(
            "INSERT INTO processo_marcador_linha
             (origem, cenario, id_procedimento, unidade, posicao,
              id_marcador, nome, cor, texto, usuario, data_hora)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                origem.url_base(), cenario, id, unidade, posicao,
                marcador.id, marcador.nome, marcador.cor,
                marcador.texto, marcador.usuario, marcador.data_hora
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Busca a observação sem confundir sua ausência com uma lista vazia.
/// A ordem das linhas da tela é preservada por posicao.
///
/// # Errors
/// Preserva SQLite e recusa quantidade inválida.
pub fn ler(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
) -> Result<Option<Observacao>, Falha> {
    let total: Option<i64> = banco.query_row(
        "SELECT total_declarado FROM processo_marcadores_observados
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4",
        params![origem.url_base(), cenario, id, unidade],
        |linha| linha.get(0),
    ).optional()?;
    let Some(total) = total else { return Ok(None) };
    let total_declarado = usize::try_from(total).map_err(|_| Falha::DadoInvalido)?;
    let mut consulta = banco.prepare(
        "SELECT id_marcador, nome, cor, texto, usuario, data_hora
         FROM processo_marcador_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4
         ORDER BY posicao",
    )?;
    let itens = consulta.query_map(
        params![origem.url_base(), cenario, id, unidade],
        |r| Ok(Marcador {
            id:r.get(0)?, nome:r.get(1)?, cor:r.get(2)?,
            texto:r.get(3)?, usuario:r.get(4)?, data_hora:r.get(5)?,
        }),
    )?.collect::<rusqlite::Result<Vec<_>>>()?;
    let observacao = Observacao { total_declarado, itens };
    if !valido(&observacao) {
        return Err(Falha::DadoInvalido);
    }
    Ok(Some(observacao))
}
