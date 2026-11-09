//! Ciência documental do anti-IES, vinculada ao documento e ao processo no SQLite.
//! O identificador de atividade é local e sintético; não representa o SEI.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Estado {
    Pendente,
    Registrada { id_atividade: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resultado {
    Registrada { id_atividade: String },
    JaRegistrada,
    DocumentoNaoEncontrado,
}

/// Diferencia documento desconhecido de ciência ainda não registrada.
///
/// # Errors
/// Propaga falhas de SQLite sem classificações textuais.
pub fn consultar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    documento: &str,
    processo: &str,
) -> rusqlite::Result<Option<Estado>> {
    let existe: bool = banco.query_row(
        "SELECT EXISTS (SELECT 1 FROM documento_simulado
         WHERE origem=?1 AND cenario=?2 AND id_documento=?3 AND id_procedimento=?4)",
        params![origem.url_base(), cenario, documento, processo],
        |row| row.get(0),
    )?;
    if !existe {
        return Ok(None);
    }
    let atividade: Option<String> = banco
        .query_row(
            "SELECT id_atividade FROM ciencia_documental
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3 AND id_procedimento=?4",
            params![origem.url_base(), cenario, documento, processo],
            |row| row.get(0),
        )
        .optional()?;
    Ok(Some(atividade.map_or(Estado::Pendente, |id_atividade| {
        Estado::Registrada { id_atividade }
    })))
}

/// Consulta a identidade proprietária de um documento antes de um GET de ciência.
/// O SEI 4.0 pode publicar a URL da ciência sem `id_procedimento`.
///
/// # Errors
/// Propaga falhas de SQLite.
pub fn processo_do_documento(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    documento: &str,
) -> rusqlite::Result<Option<String>> {
    banco
        .query_row(
            "SELECT id_procedimento FROM documento_simulado
         WHERE origem=?1 AND cenario=?2 AND id_documento=?3",
            params![origem.url_base(), cenario, documento],
            |row| row.get(0),
        )
        .optional()
}

/// A autoria só é conhecida quando o cenário recebeu uma identidade de sessão
/// explícita. Nunca é deduzida da revisão do documento.
/// Ausência de atividade e atividade sem usuário conhecido têm o mesmo valor
/// de apresentação (None), sem inventar o responsável.
///
/// # Errors
/// Preserva falhas do SQLite.
pub fn usuario_da_ciencia(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    documento: &str,
    processo: &str,
) -> rusqlite::Result<Option<String>> {
    banco
        .query_row(
            "SELECT usuario FROM ciencia_documental
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3 AND id_procedimento=?4",
            params![origem.url_base(), cenario, documento, processo],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()
        .map(Option::flatten)
}

/// Um GET potencialmente mutador registra no máximo uma atividade por documento.
/// A atividade deriva do banco efêmero e jamais de uma URL assinada.
///
/// # Errors
/// Mantém o erro SQLite original.
pub fn registrar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    documento: &str,
    processo: &str,
) -> rusqlite::Result<Resultado> {
    let inseridos = banco.execute(
        "INSERT INTO ciencia_documental
         (origem, cenario, id_documento, id_procedimento, id_atividade, usuario)
         SELECT origem, cenario, id_documento, id_procedimento,
            CAST(1 + COALESCE((
                SELECT MAX(CAST(id_atividade AS INTEGER)) FROM ciencia_documental
                WHERE origem=?1 AND cenario=?2
            ), 42456) AS TEXT),
            (SELECT usuario FROM sessao_usuario WHERE origem=?1 AND cenario=?2)
         FROM documento_simulado
         WHERE origem=?1 AND cenario=?2 AND id_documento=?3 AND id_procedimento=?4
         ON CONFLICT(origem, cenario, id_documento) DO NOTHING",
        params![origem.url_base(), cenario, documento, processo],
    )?;
    if inseridos == 1 {
        let Some(Estado::Registrada { id_atividade }) =
            consultar(banco, origem, cenario, documento, processo)?
        else {
            unreachable!("inserção comprovada deveria ser consultável");
        };
        return Ok(Resultado::Registrada { id_atividade });
    }
    match consultar(banco, origem, cenario, documento, processo)? {
        None => Ok(Resultado::DocumentoNaoEncontrado),
        Some(Estado::Registrada { .. }) => Ok(Resultado::JaRegistrada),
        Some(Estado::Pendente) => unreachable!("documento existente teria sido registrado"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banco_cassettes::iniciar;
    use crate::documento_versoes::{DocumentoSimulado, Revisao, guardar};
    use std::num::NonZeroU32;

    #[test]
    fn ciencia_documental_preserva_identidade_e_recusa_segunda_gravacao() {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        guardar(
            &mut banco,
            "ciencia",
            &DocumentoSimulado {
                instalacao: origem.clone(),
                id_documento: "42".into(),
                id_procedimento: "7".into(),
                numero_processo: "00001.000001/2026-01".into(),
                numero: "0011519".into(),
                tipo: "Despacho".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "TESTE".into(),
                    unidade: "TESTE".into(),
                    modificacao: "09/11/2026".into(),
                    html_corpo: "<p>Sintético</p>".into(),
                }],
            },
        )
        .unwrap();
        assert_eq!(
            consultar(&banco, &origem, "ciencia", "42", "7").unwrap(),
            Some(Estado::Pendente)
        );
        assert_eq!(
            registrar(&banco, &origem, "ciencia", "42", "8").unwrap(),
            Resultado::DocumentoNaoEncontrado
        );
        let realizado = registrar(&banco, &origem, "ciencia", "42", "7").unwrap();
        assert_eq!(
            realizado,
            Resultado::Registrada {
                id_atividade: "42457".into()
            }
        );
        assert_eq!(
            registrar(&banco, &origem, "ciencia", "42", "7").unwrap(),
            Resultado::JaRegistrada
        );
        assert_eq!(
            usuario_da_ciencia(&banco, &origem, "ciencia", "42", "7").unwrap(),
            None,
            "sem identidade explícita de sessão não há autoria inventada"
        );
        assert_eq!(
            consultar(&banco, &origem, "outro", "42", "7").unwrap(),
            None
        );
    }
}
