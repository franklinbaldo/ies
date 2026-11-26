//! Tipos oferecidos e páginas observadas por critérios da pesquisa processual.
use crate::instalacao::Instalacao;
use crate::observacao::UnidadeExibida;
use rusqlite::{Connection, OptionalExtension, params};

/// ID e limites como publicados no formulário; vazio representa filtro omitido.
#[derive(Clone, Copy)]
pub struct Criterios<'a> {
    pub tipo_id: &'a str,
    pub de: &'a str,
    pub ate: &'a str,
}

/// Registra uma opção do seletor de pesquisa, independente do resultado.
///
/// # Errors
/// Propaga falha SQLite.
pub fn registrar_tipo(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    id: &str,
    nome: &str,
) -> rusqlite::Result<()> {
    banco.execute(
        "INSERT INTO pesquisa_processo_tipo (origem, cenario, unidade, id_tipo, nome)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(origem, cenario, unidade, id_tipo) DO UPDATE SET nome=excluded.nome",
        params![instalacao.url_base(), cenario, unidade.as_str(), id, nome],
    )?;
    Ok(())
}

/// Opções conhecidas da pesquisa na unidade, sem inferir oferta pelo cadastro.
///
/// # Errors
/// Propaga falha SQLite.
pub fn tipos(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
) -> rusqlite::Result<Vec<(String, String)>> {
    let mut consulta = banco.prepare(
        "SELECT id_tipo, nome FROM pesquisa_processo_tipo
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 ORDER BY id_tipo",
    )?;
    consulta
        .query_map(
            params![instalacao.url_base(), cenario, unidade.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?
        .collect()
}

/// Registra somente a observação negativa explícita do cenário.
///
/// # Errors
/// Propaga a falha SQLite (inclusive critérios inteiramente vazios).
pub fn registrar_sem_resultado(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    criterios: Criterios<'_>,
) -> rusqlite::Result<()> {
    banco.execute(
        "INSERT INTO pesquisa_processo_pagina
         (origem, cenario, unidade, id_tipo, data_inicio, data_fim, pagina, inicio, fim, total)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 1, 0, 0)
         ON CONFLICT(origem, cenario, unidade, id_tipo, data_inicio, data_fim, pagina)
         DO UPDATE SET total=excluded.total",
        params![
            instalacao.url_base(),
            cenario,
            unidade.as_str(),
            criterios.tipo_id,
            criterios.de,
            criterios.ate
        ],
    )?;
    Ok(())
}

/// Estado observado da chave completa, independentemente da página solicitada.
#[derive(Debug, PartialEq, Eq)]
pub enum Coorte {
    Desconhecida,
    Vazia,
    Positiva,
    Contraditoria,
}

/// Classifica a coorte sem apagar observações.
/// # Errors
/// Propaga a falha SQLite.
pub fn coorte(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    criterios: Criterios<'_>,
) -> rusqlite::Result<Coorte> {
    let (zero, positiva): (bool, bool) = banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM pesquisa_processo_pagina
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND id_tipo=?4
         AND data_inicio=?5 AND data_fim=?6 AND pagina=1 AND total=0),
         EXISTS(SELECT 1 FROM pesquisa_processo_pagina
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND id_tipo=?4
         AND data_inicio=?5 AND data_fim=?6 AND total>0)",
        params![
            instalacao.url_base(),
            cenario,
            unidade.as_str(),
            criterios.tipo_id,
            criterios.de,
            criterios.ate
        ],
        |linha| Ok((linha.get(0)?, linha.get(1)?)),
    )?;
    Ok(match (zero, positiva) {
        (false, false) => Coorte::Desconhecida,
        (true, false) => Coorte::Vazia,
        (false, true) => Coorte::Positiva,
        (true, true) => Coorte::Contraditoria,
    })
}

/// Faixa publicada e identidades das linhas efetivamente observadas.
#[derive(Debug, PartialEq, Eq)]
pub struct Pagina {
    pub numero: u32,
    pub inicio: u64,
    pub fim: u64,
    pub total: u64,
    pub ids: Vec<String>,
}

/// Persiste uma página positiva sem inventar processos ausentes da faixa.
///
/// # Errors
/// Propaga falhas SQLite e recusa faixa ou identidade inconsistente.
pub fn registrar_pagina(
    banco: &mut Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    criterios: Criterios<'_>,
    pagina: &Pagina,
) -> rusqlite::Result<()> {
    let quantidade = u64::try_from(pagina.ids.len()).map_err(|_| rusqlite::Error::InvalidQuery)?;
    if pagina.total == 0
        || pagina.inicio == 0
        || pagina.fim < pagina.inicio
        || quantidade > pagina.fim - pagina.inicio + 1
    {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let inicio = i64::try_from(pagina.inicio).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let fim = i64::try_from(pagina.fim).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let total = i64::try_from(pagina.total).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let tx = banco.transaction()?;
    tx.execute(
        "INSERT INTO pesquisa_processo_pagina
         (origem, cenario, unidade, id_tipo, data_inicio, data_fim, pagina, inicio, fim, total)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            instalacao.url_base(),
            cenario,
            unidade.as_str(),
            criterios.tipo_id,
            criterios.de,
            criterios.ate,
            pagina.numero,
            inicio,
            fim,
            total
        ],
    )?;
    for (posicao, id) in pagina.ids.iter().enumerate() {
        if crate::documentos_simulados::ler_processo(&tx, instalacao, cenario, id)?.is_none() {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        tx.execute(
            "INSERT INTO pesquisa_processo_pagina_linha
             (origem, cenario, unidade, id_tipo, data_inicio, data_fim, pagina, posicao, id_procedimento)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                instalacao.url_base(),
                cenario,
                unidade.as_str(),
                criterios.tipo_id,
                criterios.de,
                criterios.ate,
                pagina.numero,
                i64::try_from(posicao).map_err(|_| rusqlite::Error::InvalidQuery)?,
                id
            ],
        )?;
    }
    tx.commit()
}

/// Recupera apenas a faixa positiva registrada para o deslocamento solicitado.
///
/// # Errors
/// Propaga a falha SQLite.
pub fn consultar_pagina(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    criterios: Criterios<'_>,
    deslocamento: u64,
) -> rusqlite::Result<Option<Pagina>> {
    let inicio = deslocamento
        .checked_add(1)
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let inicio_sql = i64::try_from(inicio).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let dados: Option<(u32, i64, i64)> = banco
        .query_row(
            "SELECT pagina, fim, total FROM pesquisa_processo_pagina
             WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND id_tipo=?4
             AND data_inicio=?5 AND data_fim=?6 AND inicio=?7 AND total>0",
            params![
                instalacao.url_base(),
                cenario,
                unidade.as_str(),
                criterios.tipo_id,
                criterios.de,
                criterios.ate,
                inicio_sql
            ],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((numero, fim, total)) = dados else {
        return Ok(None);
    };
    let mut consulta = banco.prepare(
        "SELECT id_procedimento FROM pesquisa_processo_pagina_linha
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND id_tipo=?4
         AND data_inicio=?5 AND data_fim=?6 AND pagina=?7 ORDER BY posicao",
    )?;
    let ids = consulta
        .query_map(
            params![
                instalacao.url_base(),
                cenario,
                unidade.as_str(),
                criterios.tipo_id,
                criterios.de,
                criterios.ate,
                numero
            ],
            |r| r.get(0),
        )?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(Some(Pagina {
        numero,
        inicio,
        fim: u64::try_from(fim).map_err(|_| rusqlite::Error::InvalidQuery)?,
        total: u64::try_from(total).map_err(|_| rusqlite::Error::InvalidQuery)?,
        ids,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negativo_nao_vaza_para_outro_periodo_ou_unidade() {
        let banco = Connection::open_in_memory().unwrap();
        crate::banco_cassettes::iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let unidade = UnidadeExibida::nova("TESTE").unwrap();
        let periodo = Criterios {
            tipo_id: "",
            de: "01/01/2026",
            ate: "31/01/2026",
        };
        assert_eq!(
            coorte(&banco, &origem, "c", &unidade, periodo).unwrap(),
            Coorte::Desconhecida
        );
        registrar_sem_resultado(&banco, &origem, "c", &unidade, periodo).unwrap();
        assert_eq!(
            coorte(&banco, &origem, "c", &unidade, periodo).unwrap(),
            Coorte::Vazia
        );
        for outro in [
            Criterios {
                tipo_id: "73",
                ..periodo
            },
            Criterios { de: "", ..periodo },
            Criterios {
                ate: "30/01/2026",
                ..periodo
            },
        ] {
            assert_eq!(
            coorte(&banco, &origem, "c", &unidade, outro).unwrap(),
            Coorte::Desconhecida
        );
        }
        assert_eq!(
            coorte(
                &banco,
                &origem,
                "c",
                &UnidadeExibida::nova("OUTRA").unwrap(),
                periodo
            )
            .unwrap(),
            Coorte::Desconhecida
        );
    }

    #[test]
    fn pagina_parcial_preserva_faixa_e_identidade() {
        let mut banco = Connection::open_in_memory().unwrap();
        crate::banco_cassettes::iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let unidade = UnidadeExibida::nova("TESTE").unwrap();
        let criterios = Criterios {
            tipo_id: "73",
            de: "01/01/2026",
            ate: "31/01/2026",
        };
        let fora_sqlite = Pagina {
            numero: 1,
            inicio: 1,
            fim: 1,
            total: u64::MAX,
            ids: vec![],
        };
        assert!(
            registrar_pagina(&mut banco, &origem, "c", &unidade, criterios, &fora_sqlite).is_err()
        );
        assert!(consultar_pagina(&banco, &origem, "c", &unidade, criterios, u64::MAX).is_err());
        assert!(
            consultar_pagina(
                &banco,
                &origem,
                "c",
                &unidade,
                criterios,
                i64::MAX.unsigned_abs()
            )
            .is_err()
        );
        let pagina = Pagina {
            numero: 1,
            inicio: 1,
            fim: 2,
            total: 4,
            ids: vec!["7".into()],
        };
        assert!(registrar_pagina(&mut banco, &origem, "c", &unidade, criterios, &pagina).is_err());
        assert!(
            consultar_pagina(&banco, &origem, "c", &unidade, criterios, 0)
                .unwrap()
                .is_none()
        );
        crate::documentos_simulados::registrar_processo(
            &banco,
            &origem,
            "c",
            &crate::documentos_simulados::Processo {
                id: "7".into(),
                nup: "00001.000007/2026-00".into(),
                unidade: "TESTE".into(),
            },
        )
        .unwrap();
        registrar_pagina(&mut banco, &origem, "c", &unidade, criterios, &pagina).unwrap();
        assert_eq!(
            consultar_pagina(&banco, &origem, "c", &unidade, criterios, 0).unwrap(),
            Some(pagina)
        );
        assert!(
            consultar_pagina(&banco, &origem, "c", &unidade, criterios, 2)
                .unwrap()
                .is_none()
        );
        assert!(registrar_sem_resultado(&banco, &origem, "c", &unidade, criterios).is_err());
        assert_eq!(
            coorte(&banco, &origem, "c", &unidade, criterios).unwrap(),
            Coorte::Positiva
        );
    }

    #[test]
    fn coorte_contraditoria_preserva_observacoes_e_isola_chave() {
        for zero_primeiro in [true, false] {
            let mut banco = Connection::open_in_memory().unwrap();
            crate::banco_cassettes::iniciar(&banco).unwrap();
            let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
            let unidade = UnidadeExibida::nova("TESTE").unwrap();
            let criterios = Criterios {
                tipo_id: "73",
                de: "01/01/2026",
                ate: "31/01/2026",
            };
            let pagina = Pagina {
                numero: 2,
                inicio: 3,
                fim: 4,
                total: 4,
                ids: vec![],
            };
            if zero_primeiro {
                registrar_sem_resultado(&banco, &origem, "c", &unidade, criterios).unwrap();
            }
            registrar_pagina(&mut banco, &origem, "c", &unidade, criterios, &pagina).unwrap();
            if !zero_primeiro {
                registrar_sem_resultado(&banco, &origem, "c", &unidade, criterios).unwrap();
            }
            assert_eq!(
                coorte(&banco, &origem, "c", &unidade, criterios).unwrap(),
                Coorte::Contraditoria
            );
            assert_eq!(
                consultar_pagina(&banco, &origem, "c", &unidade, criterios, 2).unwrap(),
                Some(pagina)
            );
            for (o, c, u, crit) in [
                (Instalacao::interpretar("https://outro.test/sei").unwrap(), "c", unidade.clone(), criterios),
                (origem.clone(), "outro", unidade.clone(), criterios),
                (origem.clone(), "c", UnidadeExibida::nova("OUTRA").unwrap(), criterios),
                (origem.clone(), "c", unidade.clone(), Criterios { tipo_id: "74", ..criterios }),
                (origem.clone(), "c", unidade.clone(), Criterios { de: "", ..criterios }),
                (origem.clone(), "c", unidade.clone(), Criterios { ate: "", ..criterios }),
            ] {
                assert_eq!(coorte(&banco, &o, c, &u, crit).unwrap(), Coorte::Desconhecida);
            }
        }
    }
}
