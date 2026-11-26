//! Tipos oferecidos e observação negativa por critérios da pesquisa processual.
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
        "INSERT INTO pesquisa_processo_periodo_vazio
         (origem, cenario, unidade, id_tipo, data_inicio, data_fim)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT DO NOTHING",
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

/// A ausência da chave indica desconhecimento, não uma busca vazia.
///
/// # Errors
/// Propaga a falha SQLite.
pub fn tem_resultado_negativo(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    criterios: Criterios<'_>,
) -> rusqlite::Result<bool> {
    let registro: Option<i64> = banco
        .query_row(
            "SELECT 1 FROM pesquisa_processo_periodo_vazio
             WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND id_tipo=?4 AND data_inicio=?5 AND data_fim=?6",
            params![
                instalacao.url_base(),
                cenario,
                unidade.as_str(),
                criterios.tipo_id,
                criterios.de,
                criterios.ate
            ],
            |linha| linha.get(0),
        )
        .optional()?;
    Ok(registro.is_some())
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
        assert!(!tem_resultado_negativo(&banco, &origem, "c", &unidade, periodo).unwrap());
        registrar_sem_resultado(&banco, &origem, "c", &unidade, periodo).unwrap();
        assert!(tem_resultado_negativo(&banco, &origem, "c", &unidade, periodo).unwrap());
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
            assert!(!tem_resultado_negativo(&banco, &origem, "c", &unidade, outro).unwrap());
        }
        assert!(
            !tem_resultado_negativo(
                &banco,
                &origem,
                "c",
                &UnidadeExibida::nova("OUTRA").unwrap(),
                periodo
            )
            .unwrap()
        );
    }
}
