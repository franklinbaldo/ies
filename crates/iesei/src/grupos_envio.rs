//! Catálogo integral sintético de grupos e oferta independente de unidades.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

#[derive(Clone, Copy)]
pub struct Escopo<'a> {
    pub instalacao: &'a Instalacao,
    pub cenario: &'a str,
    pub unidade: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Membro {
    pub id: String,
    pub nome: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Grupo {
    pub id: String,
    pub nome: String,
    pub descricao: String,
}

/// Semeia explicitamente um catálogo vazio e a oferta do autocomplete.
/// # Errors
/// Propaga falha SQL, inclusive tentativa de substituir catálogo existente.
pub fn semear(
    banco: &Connection,
    escopo: Escopo<'_>,
    oferecidos: &[Membro],
) -> rusqlite::Result<()> {
    let origem = escopo.instalacao.url_base();
    let tx = Transaction::new_unchecked(banco, TransactionBehavior::Immediate)?;
    tx.execute(
        "INSERT INTO sim_grupo_envio_catalogo VALUES (?1,?2,?3,1)",
        params![origem, escopo.cenario, escopo.unidade],
    )?;
    for membro in oferecidos {
        tx.execute(
            "INSERT INTO sim_grupo_envio_unidade VALUES (?1,?2,?3,?4,?5)",
            params![
                origem,
                escopo.cenario,
                escopo.unidade,
                membro.id,
                membro.nome
            ],
        )?;
    }
    tx.commit()
}

fn catalogo_existe(banco: &Connection, escopo: Escopo<'_>) -> rusqlite::Result<bool> {
    banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM sim_grupo_envio_catalogo
         WHERE origem=?1 AND cenario=?2 AND unidade=?3)",
        params![escopo.instalacao.url_base(), escopo.cenario, escopo.unidade],
        |r| r.get(0),
    )
}

/// Oferta independente da existência de grupos. None significa catálogo ausente.
/// # Errors
/// Propaga erro SQLite.
pub fn unidades(banco: &Connection, escopo: Escopo<'_>) -> rusqlite::Result<Option<Vec<Membro>>> {
    if !catalogo_existe(banco, escopo)? {
        return Ok(None);
    }
    let mut consulta = banco.prepare(
        "SELECT id_unidade,nome FROM sim_grupo_envio_unidade
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 ORDER BY id_unidade",
    )?;
    let membros = consulta
        .query_map(
            params![escopo.instalacao.url_base(), escopo.cenario, escopo.unidade],
            |r| {
                Ok(Membro {
                    id: r.get(0)?,
                    nome: r.get(1)?,
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some(membros))
}

/// Aloca identidade e persiste o grupo e seus membros em uma transação.
/// # Errors
/// Recusa catálogo ausente, par não oferecido ou colisão; reverte toda a escrita.
pub fn criar(
    banco: &Connection,
    escopo: Escopo<'_>,
    nome: &str,
    descricao: &str,
    membros: &[Membro],
) -> rusqlite::Result<String> {
    let origem = escopo.instalacao.url_base();
    let tx = Transaction::new_unchecked(banco, TransactionBehavior::Immediate)?;
    let proximo: i64 = tx.query_row(
        "UPDATE sim_grupo_envio_catalogo SET proximo_id=proximo_id+1
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 RETURNING proximo_id-1",
        params![origem, escopo.cenario, escopo.unidade],
        |r| r.get(0),
    )?;
    let id = proximo.to_string();
    tx.execute(
        "INSERT INTO sim_grupo_envio VALUES (?1,?2,?3,?4,?5,?6)",
        params![origem, escopo.cenario, escopo.unidade, id, nome, descricao],
    )?;
    for (posicao, membro) in membros.iter().enumerate() {
        let posicao = i64::try_from(posicao).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let inseridos = tx.execute(
            "INSERT INTO sim_grupo_envio_membro
             SELECT origem,cenario,unidade,?4,id_unidade,nome,?7
             FROM sim_grupo_envio_unidade
             WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND id_unidade=?5 AND nome=?6",
            params![
                origem,
                escopo.cenario,
                escopo.unidade,
                id,
                membro.id,
                membro.nome,
                posicao
            ],
        )?;
        if inseridos != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
    }
    tx.commit()?;
    Ok(id)
}

/// Lista somente o catálogo explicitamente semeado.
/// # Errors
/// Propaga erro SQLite.
pub fn listar(banco: &Connection, escopo: Escopo<'_>) -> rusqlite::Result<Option<Vec<Grupo>>> {
    if !catalogo_existe(banco, escopo)? {
        return Ok(None);
    }
    let mut consulta = banco.prepare(
        "SELECT id_grupo,nome,descricao FROM sim_grupo_envio
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 ORDER BY id_grupo",
    )?;
    let grupos = consulta
        .query_map(
            params![escopo.instalacao.url_base(), escopo.cenario, escopo.unidade],
            |r| {
                Ok(Grupo {
                    id: r.get(0)?,
                    nome: r.get(1)?,
                    descricao: r.get(2)?,
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some(grupos))
}

/// Lê um grupo específico e seus membros na ordem registrada.
/// # Errors
/// Propaga erro SQLite; ausência não equivale a grupo vazio.
pub fn obter(
    banco: &Connection,
    escopo: Escopo<'_>,
    id: &str,
) -> rusqlite::Result<Option<(Grupo, Vec<Membro>)>> {
    let origem = escopo.instalacao.url_base();
    let grupo = banco
        .query_row(
            "SELECT id_grupo,nome,descricao FROM sim_grupo_envio
             WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND id_grupo=?4",
            params![origem, escopo.cenario, escopo.unidade, id],
            |r| {
                Ok(Grupo {
                    id: r.get(0)?,
                    nome: r.get(1)?,
                    descricao: r.get(2)?,
                })
            },
        )
        .optional()?;
    let Some(grupo) = grupo else {
        return Ok(None);
    };
    let mut consulta = banco.prepare(
        "SELECT id_unidade,nome FROM sim_grupo_envio_membro
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND id_grupo=?4 ORDER BY posicao",
    )?;
    let membros = consulta
        .query_map(params![origem, escopo.cenario, escopo.unidade, id], |r| {
            Ok(Membro {
                id: r.get(0)?,
                nome: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some((grupo, membros)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn membro(id: &str, nome: &str) -> Membro {
        Membro {
            id: id.to_owned(),
            nome: nome.to_owned(),
        }
    }

    #[test]
    fn oferta_independente_e_rollback_do_id() {
        // DDL isolado enquanto 0042–0044 não forem integradas.
        // Remover aplicação manual quando 0045 entrar no registrador.
        let banco = Connection::open_in_memory().unwrap();
        banco
            .execute_batch(include_str!("../migrations/0045_grupos_envio.sql"))
            .unwrap();
        let instalacao = Instalacao::interpretar("https://sei.example.test").unwrap();
        let escopo = Escopo {
            instalacao: &instalacao,
            cenario: "grupo",
            unidade: "A",
        };
        let outro = Escopo {
            unidade: "B",
            ..escopo
        };
        let oferta = [membro("001", "X - Unidade"), membro("1", "Y - Unidade")];
        assert_eq!(listar(&banco, escopo).unwrap(), None);
        assert_eq!(unidades(&banco, escopo).unwrap(), None);
        semear(&banco, escopo, &oferta).unwrap();
        assert_eq!(listar(&banco, escopo).unwrap(), Some(Vec::new()));
        assert_eq!(unidades(&banco, escopo).unwrap(), Some(oferta.to_vec()));
        assert_eq!(unidades(&banco, outro).unwrap(), None);
        let falsos = [membro("001", "X - Unidade"), membro("1", "Nome inventado")];
        assert!(criar(&banco, escopo, "Grupo", "", &falsos).is_err());
        assert_eq!(listar(&banco, escopo).unwrap(), Some(Vec::new()));
        let id = criar(&banco, escopo, "Grupo", "Descrição", &oferta).unwrap();
        assert_eq!(id, "1");
        let (grupo, membros) = obter(&banco, escopo, &id).unwrap().unwrap();
        assert_eq!(grupo.nome, "Grupo");
        assert_eq!(grupo.descricao, "Descrição");
        assert_eq!(membros, oferta);
        assert_eq!(obter(&banco, outro, &id).unwrap(), None);
        assert_eq!(unidades(&banco, escopo).unwrap(), Some(oferta.to_vec()));
    }
}
