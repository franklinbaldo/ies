//! Comentários observados no SQLite efêmero, sem confundir ausência e vazio.
//! Suporta fotografias incompletas: quem consome decide se pode afirmar completude.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visao {
    Processo,
    Todos,
}

impl Visao {
    fn codigo(self) -> &'static str {
        match self {
            Self::Processo => "P",
            Self::Todos => "T",
        }
    }
    fn decodificar(valor: &str) -> Option<Self> {
        match valor {
            "P" => Some(Self::Processo),
            "T" => Some(Self::Todos),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comentario {
    pub id: String,
    pub data_hora: String,
    pub unidade: String,
    pub usuario: String,
    pub texto: String,
    pub protocolo: Option<String>,
    pub tipo: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observacao {
    pub visao: Visao,
    pub total_declarado: usize,
    pub itens: Vec<Comentario>,
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

fn valida(foto: &Observacao) -> bool {
    if foto.itens.len() > foto.total_declarado {
        return false;
    }
    for (i, item) in foto.itens.iter().enumerate() {
        if item.id.is_empty()
            || !item.id.bytes().all(|b| b.is_ascii_digit())
            || item.data_hora.is_empty()
            || item.unidade.is_empty()
            || item.usuario.is_empty()
            || item.protocolo.is_some() != item.tipo.is_some()
            || foto.itens[..i].iter().any(|outro| outro.id == item.id)
        {
            return false;
        }
    }
    true
}

/// Grava foto atômica, inclusive sua cobertura declarada, sem afirmar completude.
///
/// # Errors
/// Rejeita processo não observado, ids ambíguos e falhas SQLite.
pub fn guardar(
    banco: &mut Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    foto: &Observacao,
) -> Result<(), Falha> {
    if cenario.is_empty()
        || id.is_empty()
        || !id.bytes().all(|b| b.is_ascii_digit())
        || !valida(foto)
    {
        return Err(Falha::DadoInvalido);
    }
    let total = i64::try_from(foto.total_declarado).map_err(|_| Falha::DadoInvalido)?;
    let tx = banco.transaction()?;
    let conhecido: i64 = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sim_processo
          WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3)",
        params![origem.url_base(), cenario, id],
        |r| r.get(0),
    )?;
    if conhecido == 0 {
        return Err(Falha::ProcessoNaoObservado);
    }
    tx.execute(
        "INSERT INTO processo_comentarios_observados
         (origem, cenario, id_procedimento, visao, total_declarado)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (origem, cenario, id_procedimento)
         DO UPDATE SET visao=excluded.visao, total_declarado=excluded.total_declarado",
        params![origem.url_base(), cenario, id, foto.visao.codigo(), total],
    )?;
    tx.execute(
        "DELETE FROM processo_comentario_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem.url_base(), cenario, id],
    )?;
    for (posicao, item) in foto.itens.iter().enumerate() {
        let posicao = i64::try_from(posicao).map_err(|_| Falha::DadoInvalido)?;
        tx.execute(
            "INSERT INTO processo_comentario_linha
             (origem, cenario, id_procedimento, posicao, id_comentario,
              data_hora, unidade, usuario, texto, protocolo, tipo)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                origem.url_base(),
                cenario,
                id,
                posicao,
                item.id,
                item.data_hora,
                item.unidade,
                item.usuario,
                item.texto,
                item.protocolo,
                item.tipo
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Consulta a foto como observada: desconhecido não vira zero.
///
/// # Errors
/// Propaga erros SQLite e recusa esquema/conteúdo inconsistente.
pub fn ler(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Option<Observacao>, Falha> {
    let cabecalho: Option<(String, i64)> = banco
        .query_row(
            "SELECT visao, total_declarado FROM processo_comentarios_observados
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
            params![origem.url_base(), cenario, id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((codigo, total)) = cabecalho else {
        return Ok(None);
    };
    let visao = Visao::decodificar(&codigo).ok_or(Falha::DadoInvalido)?;
    let total_declarado = usize::try_from(total).map_err(|_| Falha::DadoInvalido)?;
    let mut stmt = banco.prepare(
        "SELECT id_comentario, data_hora, unidade, usuario, texto, protocolo, tipo
         FROM processo_comentario_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 ORDER BY posicao",
    )?;
    let itens = stmt
        .query_map(params![origem.url_base(), cenario, id], |r| {
            Ok(Comentario {
                id: r.get(0)?,
                data_hora: r.get(1)?,
                unidade: r.get(2)?,
                usuario: r.get(3)?,
                texto: r.get(4)?,
                protocolo: r.get(5)?,
                tipo: r.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let foto = Observacao {
        visao,
        total_declarado,
        itens,
    };
    if !valida(&foto) {
        return Err(Falha::DadoInvalido);
    }
    Ok(Some(foto))
}

/// Acrescenta um comentário **somente** à observação completa, em transação
/// IMMEDIATE para não perder escritas concorrentes. Identidade é sintética.
/// Não altera o índice operacional `ies.sqlite3`.
///
/// # Errors
/// Ausência, fotografia parcial, novo item inválido ou falha SQLite.
pub fn acrescentar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    mut novo: Comentario,
) -> Result<String, Falha> {
    if cenario.is_empty()
        || !novo.id.is_empty()
        || novo.texto.trim().is_empty()
        || novo.data_hora.is_empty()
        || novo.unidade.is_empty()
        || novo.usuario.is_empty()
        || novo.protocolo.is_some()
        || novo.tipo.is_some()
    {
        return Err(Falha::DadoInvalido);
    }
    let tx = rusqlite::Transaction::new_unchecked(banco, rusqlite::TransactionBehavior::Immediate)?;
    let Some(foto) = ler(&tx, origem, cenario, id)? else {
        return Err(Falha::DadoInvalido);
    };
    if foto.total_declarado != foto.itens.len() {
        return Err(Falha::DadoInvalido);
    }
    let proximo = foto
        .itens
        .iter()
        .map(|item| item.id.parse::<u64>().map_err(|_| Falha::DadoInvalido))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or(Falha::DadoInvalido)?;
    novo.id = proximo.to_string();
    let total = foto.total_declarado.checked_add(1).and_then(|valor| i64::try_from(valor).ok()).ok_or(Falha::DadoInvalido)?;
    let posicao = i64::try_from(foto.itens.len()).map_err(|_| Falha::DadoInvalido)?;
    let atualizadas = tx.execute(
        "UPDATE processo_comentarios_observados SET total_declarado=?4
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem.url_base(), cenario, id, total],
    )?;
    if atualizadas != 1 {
        return Err(Falha::DadoInvalido);
    }
    tx.execute(
        "INSERT INTO processo_comentario_linha
         (origem, cenario, id_procedimento, posicao, id_comentario,
          data_hora, unidade, usuario, texto, protocolo, tipo)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            origem.url_base(),
            cenario,
            id,
            posicao,
            novo.id,
            novo.data_hora,
            novo.unidade,
            novo.usuario,
            novo.texto,
            novo.protocolo,
            novo.tipo,
        ],
    )?;
    tx.commit()?;
    Ok(proximo.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banco_cassettes;
    use crate::processos::{self, ProcessoSimulado};

    #[test]
    fn distingue_desconhecido_vazio_e_pagina_incompleta() {
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let cenario = "comentarios";
        processos::guardar(
            &mut banco,
            &origem,
            cenario,
            &ProcessoSimulado {
                id: "10".into(),
                numero: "00001.000001/2026-01".into(),
                unidade_origem: "TESTE".into(),
                aberto_em: vec!["TESTE".into()],
                anotacao_origem: None,
                destinos: vec![],
            },
        )
        .unwrap();
        assert_eq!(ler(&banco, &origem, cenario, "10").unwrap(), None);
        let mut foto = Observacao {
            visao: Visao::Processo,
            total_declarado: 0,
            itens: vec![],
        };
        guardar(&mut banco, &origem, cenario, "10", &foto).unwrap();
        assert_eq!(
            ler(&banco, &origem, cenario, "10").unwrap(),
            Some(foto.clone())
        );
        foto.total_declarado = 2;
        foto.itens.push(Comentario {
            id: "42".into(),
            data_hora: "10/11/2026".into(),
            unidade: "TESTE".into(),
            usuario: "u".into(),
            texto: "texto".into(),
            protocolo: None,
            tipo: None,
        });
        guardar(&mut banco, &origem, cenario, "10", &foto).unwrap();
        assert_eq!(ler(&banco, &origem, cenario, "10").unwrap(), Some(foto));
    }
}
