//! Observação negativa por período, separada da pesquisa documental.
use crate::instalacao::Instalacao;
use crate::observacao::UnidadeExibida;
use rusqlite::{Connection, OptionalExtension, params};

/// Limites como publicados no formulário; vazio representa limite omitido.
#[derive(Clone, Copy)]
pub struct Periodo<'a> {
    pub de: &'a str,
    pub ate: &'a str,
}

/// Registra somente a observação negativa explícita do cenário.
///
/// # Errors
/// Propaga a falha SQLite (inclusive período inteiramente vazio).
pub fn registrar_sem_resultado(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    periodo: Periodo<'_>,
) -> rusqlite::Result<()> {
    banco.execute(
        "INSERT INTO pesquisa_processo_periodo_vazio
         (origem, cenario, unidade, data_inicio, data_fim)
         VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT DO NOTHING",
        params![
            instalacao.url_base(),
            cenario,
            unidade.as_str(),
            periodo.de,
            periodo.ate
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
    periodo: Periodo<'_>,
) -> rusqlite::Result<bool> {
    let registro: Option<i64> = banco
        .query_row(
            "SELECT 1 FROM pesquisa_processo_periodo_vazio
             WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND data_inicio=?4 AND data_fim=?5",
            params![
                instalacao.url_base(),
                cenario,
                unidade.as_str(),
                periodo.de,
                periodo.ate
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
        let periodo = Periodo {
            de: "01/01/2026",
            ate: "31/01/2026",
        };
        assert!(!tem_resultado_negativo(&banco, &origem, "c", &unidade, periodo).unwrap());
        registrar_sem_resultado(&banco, &origem, "c", &unidade, periodo).unwrap();
        assert!(tem_resultado_negativo(&banco, &origem, "c", &unidade, periodo).unwrap());
        for outro in [
            Periodo { de: "", ..periodo },
            Periodo { ate: "30/01/2026", ..periodo },
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
