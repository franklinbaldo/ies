//! Itens observados de um bloco publicado; estado derivado independente do HTML.
//! SQLite efemero compartilhado pelo anti-IES; nenhum dado de sessao.
use crate::blocos::Tipo;
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    pub processo: String,
    pub documento: Option<String>,
    pub tipo: Option<String>,
    pub anotacoes: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Detalhe {
    pub numero: String,
    pub tipo: Tipo,
    pub total: usize,
    pub faixa: Option<(usize, usize)>,
    pub itens: Vec<Item>,
}

fn valido(d: &Detalhe) -> bool {
    let faixa = match d.faixa {
        Some((inicio, fim)) => {
            inicio > 0 && fim >= inicio && fim <= d.total && fim - inicio + 1 == d.itens.len()
        }
        None => d.total == 0 && d.itens.is_empty(),
    };
    faixa
        && !d.numero.is_empty()
        && d.numero.bytes().all(|b| b.is_ascii_digit())
        && d.itens.iter().all(|i| {
            !i.processo.is_empty()
                && (d.tipo != Tipo::Assinatura
                    || i.documento.as_ref().is_some_and(|doc| !doc.is_empty()))
        })
}

/// Guarda um detalhe somente se o numero constar de uma pagina registrada.
/// A ausencia de observacao nunca e materializada como lista vazia.
///
/// # Errors
/// Rejeita forma inconsistente, bloco nao publicado ou erro SQLite.
pub fn guardar(
    banco: &mut Connection,
    instalacao: &Instalacao,
    cenario: &str,
    detalhe: &Detalhe,
) -> rusqlite::Result<()> {
    if cenario.is_empty() || !valido(detalhe) {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let publicado: Option<i64> = banco
        .query_row(
            "SELECT 1 FROM sim_bloco_linha
             WHERE origem=?1 AND cenario=?2 AND tipo=?3 AND numero=?4 LIMIT 1",
            params![
            instalacao.url_base(),
            cenario,
            detalhe.tipo.nome(),
            detalhe.numero
        ],
            |r| r.get(0),
        )
        .optional()?;
    if publicado.is_none() {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let total = i64::try_from(detalhe.total).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let (inicio, fim) = match detalhe.faixa {
        Some((a, b)) => (
            Some(i64::try_from(a).map_err(|_| rusqlite::Error::InvalidQuery)?),
            Some(i64::try_from(b).map_err(|_| rusqlite::Error::InvalidQuery)?),
        ),
        None => (None, None),
    };
    let tx = banco.transaction()?;
    tx.execute(
        "DELETE FROM sim_bloco_item
         WHERE origem=?1 AND cenario=?2 AND tipo=?3 AND numero=?4",
        params![
            instalacao.url_base(),
            cenario,
            detalhe.tipo.nome(),
            detalhe.numero
        ],
    )?;
    tx.execute(
        "INSERT INTO sim_bloco_detalhe(origem,cenario,tipo,numero,total,inicio,fim)
         VALUES(?1,?2,?3,?4,?5,?6,?7)
         ON CONFLICT(origem,cenario,tipo,numero) DO UPDATE SET
         total=excluded.total,inicio=excluded.inicio,fim=excluded.fim",
        params![
            instalacao.url_base(),
            cenario,
            detalhe.tipo.nome(),
            detalhe.numero,
            total,
            inicio,
            fim
        ],
    )?;
    for (posicao, item) in detalhe.itens.iter().enumerate() {
        tx.execute(
            "INSERT INTO sim_bloco_item(
               origem,cenario,tipo,numero,posicao,processo,documento,tipo_documento,anotacoes
             ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                instalacao.url_base(),
                cenario,
                detalhe.tipo.nome(),
                detalhe.numero,
                i64::try_from(posicao).map_err(|_| rusqlite::Error::InvalidQuery)?,
                item.processo,
                item.documento,
                item.tipo,
                item.anotacoes
            ],
        )?;
    }
    tx.commit()
}

/// Le o detalhe observado ou None (nao observado), sem inferir vazio.
///
/// # Errors
/// Retorna erro SQLite se o estado armazenado estiver inconsistente.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    tipo: Tipo,
    numero: &str,
) -> rusqlite::Result<Option<Detalhe>> {
    let meta: Option<(i64, Option<i64>, Option<i64>)> = banco
        .query_row(
            "SELECT total,inicio,fim FROM sim_bloco_detalhe
             WHERE origem=?1 AND cenario=?2 AND tipo=?3 AND numero=?4",
            params![instalacao.url_base(), cenario, tipo.nome(), numero],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((total, inicio, fim)) = meta else {
        return Ok(None);
    };
    let faixa = match (inicio, fim) {
        (Some(i), Some(f)) => Some((
            usize::try_from(i).map_err(|_| rusqlite::Error::InvalidQuery)?,
            usize::try_from(f).map_err(|_| rusqlite::Error::InvalidQuery)?,
        )),
        (None, None) => None,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    let mut stmt = banco.prepare(
        "SELECT processo,documento,tipo_documento,anotacoes FROM sim_bloco_item
         WHERE origem=?1 AND cenario=?2 AND tipo=?3 AND numero=?4 ORDER BY posicao",
    )?;
    let itens = stmt
        .query_map(
            params![instalacao.url_base(), cenario, tipo.nome(), numero],
            |r| {
                Ok(Item {
                    processo: r.get(0)?,
                    documento: r.get(1)?,
                    tipo: r.get(2)?,
                    anotacoes: r.get(3)?,
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let detalhe = Detalhe {
        numero: numero.to_owned(),
        tipo,
        total: usize::try_from(total).map_err(|_| rusqlite::Error::InvalidQuery)?,
        faixa,
        itens,
    };
    if !valido(&detalhe) {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(Some(detalhe))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocos::{self, Linha, Pagina};

    #[test]
    fn detalhe_nao_observado_nao_e_vazio_e_gravacao_idempotente() {
        let mut db = Connection::open_in_memory().unwrap();
        crate::banco_cassettes::iniciar(&db).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let pagina = Pagina {
            tipo: Tipo::Assinatura,
            total: 3,
            faixa: Some((1, 1)),
            linhas: vec![Linha {
                numero: "2195".into(),
                estado: "Gerado".into(),
                descricao: "Em assinatura".into(),
            }],
        };
        blocos::guardar(&mut db, &origem, "a", &pagina).unwrap();
        let detalhe = Detalhe {
            numero: "2195".into(),
            tipo: Tipo::Assinatura,
            total: 1,
            faixa: Some((1, 1)),
            itens: vec![Item {
                processo: "NUP 1".into(),
                documento: Some("DOC 1".into()),
                tipo: None,
                anotacoes: String::new(),
            }],
        };
        assert_eq!(
            ler(&db, &origem, "a", Tipo::Assinatura, "2195").unwrap(),
            None
        );
        assert!(
            guardar(
                &mut db,
                &origem,
                "a",
                &Detalhe {
                    numero: "9999".into(),
                    ..detalhe.clone()
                }
            )
            .is_err()
        );
        guardar(&mut db, &origem, "a", &detalhe).unwrap();
        guardar(&mut db, &origem, "a", &detalhe).unwrap();
        assert_eq!(
            ler(&db, &origem, "a", Tipo::Assinatura, "2195").unwrap(),
            Some(detalhe)
        );
        assert_eq!(
            ler(&db, &origem, "b", Tipo::Assinatura, "2195").unwrap(),
            None
        );
    }
}
