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

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroMarcadores {
    Sqlite(#[from] rusqlite::Error),
    DadoInvalido,
    ProcessoNaoObservado,
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
            || !marcador
                .cor
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_')
            || marcador.data_hora.is_empty()
            || observacao.itens[..i]
                .iter()
                .any(|outro| outro.id == marcador.id)
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
) -> Result<(), ErroMarcadores> {
    if cenario.is_empty()
        || id.is_empty()
        || !id.bytes().all(|b| b.is_ascii_digit())
        || unidade.is_empty()
        || !valido(observacao)
    {
        return Err(ErroMarcadores::DadoInvalido);
    }
    let tx = banco.transaction()?;
    let aberto: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM processo_abertura
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4)",
        params![origem.url_base(), cenario, id, unidade],
        |linha| linha.get(0),
    )?;
    if !aberto {
        return Err(ErroMarcadores::ProcessoNaoObservado);
    }
    let total =
        i64::try_from(observacao.total_declarado).map_err(|_| ErroMarcadores::DadoInvalido)?;
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
        let posicao = i64::try_from(posicao).map_err(|_| ErroMarcadores::DadoInvalido)?;
        tx.execute(
            "INSERT INTO processo_marcador_linha
             (origem, cenario, id_procedimento, unidade, posicao,
              id_marcador, nome, cor, texto, usuario, data_hora)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                origem.url_base(),
                cenario,
                id,
                unidade,
                posicao,
                marcador.id,
                marcador.nome,
                marcador.cor,
                marcador.texto,
                marcador.usuario,
                marcador.data_hora
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
) -> Result<Option<Observacao>, ErroMarcadores> {
    let total: Option<i64> = banco
        .query_row(
            "SELECT total_declarado FROM processo_marcadores_observados
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4",
            params![origem.url_base(), cenario, id, unidade],
            |linha| linha.get(0),
        )
        .optional()?;
    let Some(total) = total else { return Ok(None) };
    let total_declarado = usize::try_from(total).map_err(|_| ErroMarcadores::DadoInvalido)?;
    let mut consulta = banco.prepare(
        "SELECT id_marcador, nome, cor, texto, usuario, data_hora
         FROM processo_marcador_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4
         ORDER BY posicao",
    )?;
    let itens = consulta
        .query_map(params![origem.url_base(), cenario, id, unidade], |r| {
            Ok(Marcador {
                id: r.get(0)?,
                nome: r.get(1)?,
                cor: r.get(2)?,
                texto: r.get(3)?,
                usuario: r.get(4)?,
                data_hora: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let observacao = Observacao {
        total_declarado,
        itens,
    };
    if !valido(&observacao) {
        return Err(ErroMarcadores::DadoInvalido);
    }
    Ok(Some(observacao))
}

/// Retira uma associação observada, atomicamente, sem alterar o catálogo da unidade.
/// Exclusivo do simulador: nenhum acesso à instalação SEI.
///
/// # Errors
/// Rejeita identidade ausente, processo não observado ou falha SQLite.
pub fn retirar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
    id_marcador: &str,
) -> Result<(), ErroMarcadores> {
    if id_marcador.is_empty() || !id_marcador.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroMarcadores::DadoInvalido);
    }
    let tx = banco.unchecked_transaction()?;
    let lidos =
        ler(&tx, origem, cenario, id, unidade)?.ok_or(ErroMarcadores::ProcessoNaoObservado)?;
    if !lidos.itens.iter().any(|m| m.id == id_marcador) {
        return Err(ErroMarcadores::DadoInvalido);
    }
    let alteradas = tx.execute(
        "DELETE FROM processo_marcador_linha
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4
         AND id_marcador=?5",
        params![origem.url_base(), cenario, id, unidade, id_marcador],
    )?;
    if alteradas != 1 {
        return Err(ErroMarcadores::DadoInvalido);
    }
    let total_esperado =
        i64::try_from(lidos.total_declarado).map_err(|_| ErroMarcadores::DadoInvalido)?;
    let alteradas = tx.execute(
        "UPDATE processo_marcadores_observados SET total_declarado=total_declarado-1
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4
         AND total_declarado=?5",
        params![origem.url_base(), cenario, id, unidade, total_esperado],
    )?;
    if alteradas != 1 {
        return Err(ErroMarcadores::DadoInvalido);
    }
    tx.commit()?;
    Ok(())
}

/// Aplica marcador oferecido no catálogo observado da unidade ao processo
/// com observação completa, exclusivamente no banco efêmero do anti-IES.
/// O carimbo de hora é sintético e não constitui dado observado do SEI.
///
/// # Errors
/// Recusa catálogo ausente, id não oferecido, associação duplicada ou falha SQL.
pub fn atribuir(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
    id_marcador: &str,
    texto: &str,
) -> Result<(), ErroMarcadores> {
    if id_marcador.is_empty()
        || !id_marcador.bytes().all(|b| b.is_ascii_digit())
        || texto.chars().count() > 250
    {
        return Err(ErroMarcadores::DadoInvalido);
    }
    let tx = banco.unchecked_transaction()?;
    let observacao =
        ler(&tx, origem, cenario, id, unidade)?.ok_or(ErroMarcadores::ProcessoNaoObservado)?;
    if observacao.itens.iter().any(|m| m.id == id_marcador) {
        return Err(ErroMarcadores::DadoInvalido);
    }
    // É o catálogo independente, NÃO as associações presentes, que oferece o id.
    let opcao: Option<(String, String)> = tx
        .query_row(
            "SELECT opcao.nome, opcao.cor
             FROM unidade_catalogo_marcador_opcao opcao
             JOIN unidade_catalogo_marcadores catalogo
               ON catalogo.origem=opcao.origem AND catalogo.cenario=opcao.cenario
               AND catalogo.unidade=opcao.unidade
             WHERE opcao.origem=?1 AND opcao.cenario=?2 AND opcao.unidade=?3
               AND opcao.id_marcador=?4",
            params![origem.url_base(), cenario, unidade, id_marcador],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (nome, cor) = opcao.ok_or(ErroMarcadores::DadoInvalido)?;
    let posicao =
        i64::try_from(observacao.itens.len()).map_err(|_| ErroMarcadores::DadoInvalido)?;
    let atual =
        i64::try_from(observacao.total_declarado).map_err(|_| ErroMarcadores::DadoInvalido)?;
    tx.execute(
        "INSERT INTO processo_marcador_linha
         (origem, cenario, id_procedimento, unidade, posicao,
          id_marcador, nome, cor, texto, usuario, data_hora)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10)",
        params![
            origem.url_base(),
            cenario,
            id,
            unidade,
            posicao,
            id_marcador,
            nome,
            cor,
            texto,
            "01/01/2026 00:00"
        ],
    )?;
    let alteradas = tx.execute(
        "UPDATE processo_marcadores_observados
         SET total_declarado=total_declarado+1
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4
           AND total_declarado=?5",
        params![origem.url_base(), cenario, id, unidade, atual],
    )?;
    if alteradas != 1 {
        return Err(ErroMarcadores::DadoInvalido);
    }
    tx.commit()?;
    Ok(())
}
