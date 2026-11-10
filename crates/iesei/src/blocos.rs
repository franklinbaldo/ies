//! Estado de uma unica pagina de blocos no banco derivado do ensaio.
//! Nao armazenar href assinado, cookie, hash ou dados da sessao operacional.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tipo {
    Assinatura,
    Interno,
    Reuniao,
}
impl Tipo {
    #[must_use]
    pub const fn nome(self) -> &'static str {
        match self {
            Self::Assinatura => "assinatura",
            Self::Interno => "interno",
            Self::Reuniao => "reuniao",
        }
    }
    #[must_use]
    pub const fn acao(self) -> &'static str {
        match self {
            Self::Assinatura => "bloco_assinatura_listar",
            Self::Interno => "bloco_interno_listar",
            Self::Reuniao => "bloco_reuniao_listar",
        }
    }
    #[must_use]
    pub fn da_acao(acao: &str) -> Option<Self> {
        match acao {
            "bloco_assinatura_listar" => Some(Self::Assinatura),
            "bloco_interno_listar" => Some(Self::Interno),
            "bloco_reuniao_listar" => Some(Self::Reuniao),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Linha {
    pub numero: String,
    pub estado: String,
    pub descricao: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pagina {
    pub tipo: Tipo,
    pub total: usize,
    /// Ausente em lista comprovadamente vazia.
    pub faixa: Option<(usize, usize)>,
    pub linhas: Vec<Linha>,
}
fn valido(p: &Pagina) -> bool {
    let intervalo = match p.faixa {
        Some((inicio, fim)) => {
            inicio > 0 && fim >= inicio && fim <= p.total && fim - inicio + 1 == p.linhas.len()
        }
        None => p.total == 0 && p.linhas.is_empty(),
    };
    let mut vistos = HashSet::new();
    intervalo
        && p.linhas.iter().all(|l| {
            !l.numero.is_empty()
                && l.numero.bytes().all(|c| c.is_ascii_digit())
                && vistos.insert(&l.numero)
        })
}
/// Guarda somente estado de uma pagina; a fixture nao declara completude.
/// # Errors
/// Falha se a forma observada nao fecha ou se o SQLite recusar a gravacao.
pub fn guardar(
    banco: &mut Connection,
    instalacao: &Instalacao,
    cenario: &str,
    pagina: &Pagina,
) -> rusqlite::Result<()> {
    guardar_na_pagina(banco, instalacao, cenario, 1, pagina)
}

/// Persiste uma pagina observada sem apagar as demais.
/// # Errors
/// Recusa pagina zero, dados incoerentes e erros SQLite.
pub fn guardar_na_pagina(
    banco: &mut Connection,
    instalacao: &Instalacao,
    cenario: &str,
    numero_pagina: usize,
    pagina: &Pagina,
) -> rusqlite::Result<()> {
    if cenario.is_empty() || numero_pagina == 0 || !valido(pagina) {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let numero_pagina = i64::try_from(numero_pagina).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let total = i64::try_from(pagina.total).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let (inicio, fim) = pagina.faixa.map_or((None, None), |(i, f)| {
        (i64::try_from(i).ok(), i64::try_from(f).ok())
    });
    if pagina.faixa.is_some() && (inicio.is_none() || fim.is_none()) {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let transacao = banco.transaction()?;
    transacao.execute(
        "DELETE FROM sim_bloco_linha WHERE origem=?1 AND cenario=?2 AND tipo=?3 AND pagina=?4",
        params![
            instalacao.url_base(),
            cenario,
            pagina.tipo.nome(),
            numero_pagina
        ],
    )?;
    transacao.execute(
        "INSERT INTO sim_bloco_pagina(origem,cenario,tipo,pagina,total,inicio,fim)
         VALUES(?1,?2,?3,?4,?5,?6,?7)
         ON CONFLICT(origem,cenario,tipo,pagina) DO UPDATE SET
         total=excluded.total,inicio=excluded.inicio,fim=excluded.fim",
        params![
            instalacao.url_base(),
            cenario,
            pagina.tipo.nome(),
            numero_pagina,
            total,
            inicio,
            fim
        ],
    )?;
    for (posicao, linha) in pagina.linhas.iter().enumerate() {
        transacao.execute(
            "INSERT INTO sim_bloco_linha(origem,cenario,tipo,pagina,posicao,numero,estado,descricao)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                instalacao.url_base(),
                cenario,
                pagina.tipo.nome(),
                numero_pagina,
                i64::try_from(posicao).map_err(|_| rusqlite::Error::InvalidQuery)?,
                linha.numero,
                linha.estado,
                linha.descricao
            ],
        )?;
    }
    transacao.commit()
}
/// Le uma pagina existente sem inferir paginas ou blocos ausentes.
/// # Errors
/// Propaga erro SQLite ou representacao numerica inconsistente.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    tipo: Tipo,
) -> rusqlite::Result<Option<Pagina>> {
    ler_pagina(banco, instalacao, cenario, tipo, 1)
}

/// Lê somente a pagina solicitada e observada; ausente nao significa vazia.
/// # Errors
/// Propaga erros SQLite e recusa indices nao representaveis.
pub fn ler_pagina(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    tipo: Tipo,
    numero_pagina: usize,
) -> rusqlite::Result<Option<Pagina>> {
    if numero_pagina == 0 {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let numero_pagina = i64::try_from(numero_pagina).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let linha: Option<(i64, Option<i64>, Option<i64>)> = banco
        .query_row(
            "SELECT total,inicio,fim FROM sim_bloco_pagina
         WHERE origem=?1 AND cenario=?2 AND tipo=?3 AND pagina=?4",
            params![instalacao.url_base(), cenario, tipo.nome(), numero_pagina],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((total, inicio, fim)) = linha else {
        return Ok(None);
    };
    let total = usize::try_from(total).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let faixa = match (inicio, fim) {
        (Some(i), Some(f)) => Some((
            usize::try_from(i).map_err(|_| rusqlite::Error::InvalidQuery)?,
            usize::try_from(f).map_err(|_| rusqlite::Error::InvalidQuery)?,
        )),
        (None, None) => None,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    let mut consulta = banco.prepare(
        "SELECT numero,estado,descricao FROM sim_bloco_linha
         WHERE origem=?1 AND cenario=?2 AND tipo=?3 AND pagina=?4 ORDER BY posicao",
    )?;
    let linhas = consulta
        .query_map(
            params![instalacao.url_base(), cenario, tipo.nome(), numero_pagina],
            |r| {
                Ok(Linha {
                    numero: r.get(0)?,
                    estado: r.get(1)?,
                    descricao: r.get(2)?,
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let pagina = Pagina {
        tipo,
        total,
        faixa,
        linhas,
    };
    if !valido(&pagina) {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(Some(pagina))
}

/// Indices de paginas realmente registradas para este tipo/cenario.
/// # Errors
/// Propaga falhas SQLite.
pub fn paginas_observadas(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    tipo: Tipo,
) -> rusqlite::Result<Vec<usize>> {
    let mut consulta = banco.prepare(
        "SELECT pagina FROM sim_bloco_pagina WHERE origem=?1 AND cenario=?2 AND tipo=?3 ORDER BY pagina",
    )?;
    let registros =
        consulta.query_map(params![instalacao.url_base(), cenario, tipo.nome()], |r| {
            let n: i64 = r.get(0)?;
            usize::try_from(n).map_err(|_| rusqlite::Error::InvalidQuery)
        })?;
    registros.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagina_persiste_sem_inferir_totalidade_e_sem_contaminar_cenarios() {
        let mut banco = Connection::open_in_memory().unwrap();
        crate::banco_cassettes::iniciar(&banco).unwrap();
        let instalacao = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let pagina = Pagina {
            tipo: Tipo::Assinatura,
            total: 11,
            faixa: Some((5, 6)),
            linhas: vec![
                Linha {
                    numero: "2195".into(),
                    estado: "Gerado".into(),
                    descricao: "Teste A".into(),
                },
                Linha {
                    numero: "2196".into(),
                    estado: "Disponibilizado".into(),
                    descricao: "Teste B".into(),
                },
            ],
        };
        guardar(&mut banco, &instalacao, "um", &pagina).unwrap();
        guardar(&mut banco, &instalacao, "um", &pagina).unwrap();
        assert_eq!(
            ler(&banco, &instalacao, "um", Tipo::Assinatura).unwrap(),
            Some(pagina)
        );
        assert_eq!(
            ler(&banco, &instalacao, "outro", Tipo::Assinatura).unwrap(),
            None
        );
        assert_eq!(ler(&banco, &instalacao, "um", Tipo::Interno).unwrap(), None);
    }

    #[test]
    fn duas_paginas_distintas_e_pagina_ausente_nao_e_vazia() {
        let mut banco = Connection::open_in_memory().unwrap();
        crate::banco_cassettes::iniciar(&banco).unwrap();
        let instalacao = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let pagina = |inicio: usize, numero: usize| Pagina {
            tipo: Tipo::Interno,
            total: 4,
            faixa: Some((inicio, inicio + 1)),
            linhas: [numero, numero + 1]
                .into_iter()
                .map(|n| Linha {
                    numero: n.to_string(),
                    estado: "Gerado".into(),
                    descricao: "Teste".into(),
                })
                .collect(),
        };
        let primeira = pagina(1, 1001);
        let segunda = pagina(3, 1003);
        guardar_na_pagina(&mut banco, &instalacao, "um", 1, &primeira).unwrap();
        guardar_na_pagina(&mut banco, &instalacao, "um", 2, &segunda).unwrap();
        guardar_na_pagina(&mut banco, &instalacao, "um", 2, &segunda).unwrap();
        assert_eq!(
            paginas_observadas(&banco, &instalacao, "um", Tipo::Interno).unwrap(),
            vec![1, 2]
        );
        assert_eq!(
            ler(&banco, &instalacao, "um", Tipo::Interno).unwrap(),
            Some(primeira)
        );
        assert_eq!(
            ler_pagina(&banco, &instalacao, "um", Tipo::Interno, 2).unwrap(),
            Some(segunda)
        );
        assert_eq!(
            ler_pagina(&banco, &instalacao, "um", Tipo::Interno, 3).unwrap(),
            None
        );
        assert!(ler_pagina(&banco, &instalacao, "um", Tipo::Interno, 0).is_err());
        assert!(guardar_na_pagina(&mut banco, &instalacao, "um", 0, &pagina(1, 1001)).is_err());
        assert_eq!(
            paginas_observadas(&banco, &instalacao, "outro", Tipo::Interno).unwrap(),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn vazio_observado_e_ausencia_de_observacao_sao_estados_distintos() {
        let mut banco = Connection::open_in_memory().unwrap();
        crate::banco_cassettes::iniciar(&banco).unwrap();
        let instalacao = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let vazio = Pagina {
            tipo: Tipo::Assinatura,
            total: 0,
            faixa: None,
            linhas: vec![],
        };
        assert_eq!(
            ler(&banco, &instalacao, "um", Tipo::Assinatura).unwrap(),
            None
        );
        guardar(&mut banco, &instalacao, "um", &vazio).unwrap();
        assert_eq!(
            ler(&banco, &instalacao, "um", Tipo::Assinatura).unwrap(),
            Some(vazio)
        );
        let nao_conferido = Pagina {
            tipo: Tipo::Assinatura,
            total: 10,
            faixa: Some((2, 4)),
            linhas: vec![],
        };
        assert!(guardar(&mut banco, &instalacao, "um", &nao_conferido).is_err());
    }
}
