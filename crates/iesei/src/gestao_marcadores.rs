//! Testemunhas imutáveis do catálogo de gestão, distintas das opções de atribuição.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Estado {
    Ativo,
    Inativo,
}

impl Estado {
    fn nome(self) -> &'static str {
        match self {
            Self::Ativo => "ativo",
            Self::Inativo => "inativo",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marcador {
    pub id: String,
    pub nome: String,
    pub cor: String,
    pub estado: Estado,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pagina {
    pub total: usize,
    pub faixa: Option<(usize, usize)>,
    pub itens: Vec<Marcador>,
}

fn faixa_valida(pagina: &Pagina) -> bool {
    match pagina.faixa {
        Some((inicio, fim)) => {
            inicio > 0
                && fim >= inicio
                && fim <= pagina.total
                && fim - inicio + 1 == pagina.itens.len()
        }
        None => pagina.total == 0 && pagina.itens.is_empty(),
    }
}

fn inteiro(n: usize) -> rusqlite::Result<i64> {
    i64::try_from(n).map_err(|_| rusqlite::Error::InvalidQuery)
}

fn natural(n: i64) -> rusqlite::Result<usize> {
    usize::try_from(n).map_err(|_| rusqlite::Error::InvalidQuery)
}

/// Acrescenta uma testemunha por ordinal da captura; não substitui observações.
/// # Errors
/// Recusa faixa incoerente, chave repetida ou restrição SQLite violada.
pub fn registrar(
    banco: &mut Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &str,
    ordinal: usize,
    pagina: &Pagina,
) -> rusqlite::Result<()> {
    if cenario.is_empty() || unidade.is_empty() || !faixa_valida(pagina) {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let origem = instalacao.url_base();
    let ordinal = inteiro(ordinal)?;
    let (inicio, fim) = match pagina.faixa {
        Some((i, f)) => (Some(inteiro(i)?), Some(inteiro(f)?)),
        None => (None, None),
    };
    let tx = banco.transaction()?;
    tx.execute(
        "INSERT INTO unidade_gestao_marcadores
         (origem,cenario,unidade,ordinal,total_declarado,inicio,fim)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            origem,
            cenario,
            unidade,
            ordinal,
            inteiro(pagina.total)?,
            inicio,
            fim
        ],
    )?;
    for (posicao, item) in pagina.itens.iter().enumerate() {
        tx.execute(
            "INSERT INTO unidade_gestao_marcador_linha
             (origem,cenario,unidade,ordinal,posicao,id_marcador,nome,cor,estado)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                origem,
                cenario,
                unidade,
                ordinal,
                inteiro(posicao)?,
                item.id,
                item.nome,
                item.cor,
                item.estado.nome()
            ],
        )?;
    }
    tx.commit()
}

/// Recupera uma testemunha exata; ausência não significa catálogo vazio.
/// # Errors
/// Propaga erro SQLite e recusa cardinalidade ou posições incoerentes.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &str,
    ordinal: usize,
) -> rusqlite::Result<Option<Pagina>> {
    let origem = instalacao.url_base();
    let ordinal = inteiro(ordinal)?;
    let meta: Option<(i64, Option<i64>, Option<i64>)> = banco
        .query_row(
            "SELECT total_declarado,inicio,fim FROM unidade_gestao_marcadores
             WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND ordinal=?4",
            params![origem, cenario, unidade, ordinal],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((total, inicio, fim)) = meta else {
        return Ok(None);
    };
    let faixa = match (inicio, fim) {
        (Some(i), Some(f)) => Some((natural(i)?, natural(f)?)),
        (None, None) => None,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    let mut stmt = banco.prepare(
        "SELECT posicao,id_marcador,nome,cor,estado FROM unidade_gestao_marcador_linha
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND ordinal=?4
         ORDER BY posicao",
    )?;
    let linhas = stmt.query_map(params![origem, cenario, unidade, ordinal], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;
    let mut itens = Vec::new();
    for linha in linhas {
        let (posicao, id, nome, cor, estado) = linha?;
        if natural(posicao)? != itens.len() {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let estado = match estado.as_str() {
            "ativo" => Estado::Ativo,
            "inativo" => Estado::Inativo,
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        itens.push(Marcador {
            id,
            nome,
            cor,
            estado,
        });
    }
    let pagina = Pagina {
        total: natural(total)?,
        faixa,
        itens,
    };
    if !faixa_valida(&pagina) {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(Some(pagina))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn banco() -> Connection {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        db.execute_batch(include_str!("../migrations/0043_gestao_marcadores.sql"))
            .unwrap();
        db
    }

    fn pagina() -> Pagina {
        Pagina {
            total: 5,
            faixa: Some((2, 3)),
            itens: vec![
                Marcador {
                    id: "01".into(),
                    nome: "Mesmo nome".into(),
                    cor: "Branco".into(),
                    estado: Estado::Ativo,
                },
                Marcador {
                    id: "02".into(),
                    nome: "Mesmo nome".into(),
                    cor: "Amarelo".into(),
                    estado: Estado::Inativo,
                },
            ],
        }
    }

    #[test]
    fn historico_exato_sem_inferir_vazio() {
        let mut db = banco();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let outra = Instalacao::interpretar("https://sei.test/outro").unwrap();
        let original = pagina();
        let vazia = Pagina {
            total: 0,
            faixa: None,
            itens: vec![],
        };
        registrar(&mut db, &origem, "a", "U", 1, &original).unwrap();
        registrar(&mut db, &origem, "a", "U", 2, &vazia).unwrap();
        assert_eq!(ler(&db, &origem, "a", "U", 1).unwrap(), Some(original));
        assert_eq!(ler(&db, &origem, "a", "U", 2).unwrap(), Some(vazia));
        assert_eq!(ler(&db, &origem, "a", "U", 3).unwrap(), None);
        assert_eq!(ler(&db, &origem, "a", "V", 1).unwrap(), None);
        assert_eq!(ler(&db, &origem, "b", "U", 1).unwrap(), None);
        assert_eq!(ler(&db, &outra, "a", "U", 1).unwrap(), None);
    }

    #[test]
    fn rollback_de_linha_invalida_e_ordinal_repetido() {
        let mut db = banco();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let original = pagina();
        let mut invalida = original.clone();
        invalida.itens[1].id = invalida.itens[0].id.clone();
        assert!(registrar(&mut db, &origem, "a", "U", 1, &invalida).is_err());
        assert_eq!(ler(&db, &origem, "a", "U", 1).unwrap(), None);
        let contagem: i64 = db
            .query_row(
                "SELECT count(*) FROM unidade_gestao_marcador_linha",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(contagem, 0);
        registrar(&mut db, &origem, "a", "U", 1, &original).unwrap();
        let mut alterada = original.clone();
        alterada.itens[0].estado = Estado::Inativo;
        assert!(registrar(&mut db, &origem, "a", "U", 1, &alterada).is_err());
        assert_eq!(ler(&db, &origem, "a", "U", 1).unwrap(), Some(original));
    }

    #[test]
    fn faixa_incoerente_e_lacuna_de_posicao_sao_erros() {
        let mut db = banco();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let mut incompleta = pagina();
        incompleta.itens.pop();
        assert!(registrar(&mut db, &origem, "a", "U", 1, &incompleta).is_err());
        registrar(&mut db, &origem, "a", "U", 1, &pagina()).unwrap();
        db.execute(
            "UPDATE unidade_gestao_marcador_linha SET posicao=3 WHERE posicao=1",
            [],
        )
        .unwrap();
        assert!(ler(&db, &origem, "a", "U", 1).is_err());
    }
}
