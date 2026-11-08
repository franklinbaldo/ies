//! Cancelamento de documento no SQLite descartável do anti-IES (#273).
//! O mesmo documento, processo e cenário servem prévia, POST e releitura.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Situacao {
    Disponivel,
    Cancelado { motivo: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aplicacao {
    Aplicada,
    JaCancelado,
    DocumentoNaoEncontrado,
}

/// Consulta a situação sem inferir que documento ausente está ativo.
///
/// # Errors
/// Propaga o erro SQLite original.
pub fn consultar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id_documento: &str,
    id_procedimento: &str,
) -> rusqlite::Result<Option<Situacao>> {
    let existe: bool = banco.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM documento_simulado
            WHERE origem=?1 AND cenario=?2 AND id_documento=?3 AND id_procedimento=?4
        )",
        params![origem.url_base(), cenario, id_documento, id_procedimento],
        |linha| linha.get(0),
    )?;
    if !existe {
        return Ok(None);
    }
    let motivo: Option<String> = banco
        .query_row(
            "SELECT motivo FROM documento_cancelamento
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3 AND id_procedimento=?4",
            params![origem.url_base(), cenario, id_documento, id_procedimento],
            |linha| linha.get(0),
        )
        .optional()?;
    Ok(Some(motivo.map_or(Situacao::Disponivel, |motivo| {
        Situacao::Cancelado { motivo }
    })))
}

/// Aplica a escrita uma única vez no cenário; concorrência jamais troca o motivo.
///
/// # Errors
/// Preserva a falha SQLite da tentativa.
pub fn aplicar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id_documento: &str,
    id_procedimento: &str,
    motivo: &str,
) -> rusqlite::Result<Aplicacao> {
    let inseridos = banco.execute(
        "INSERT INTO documento_cancelamento
            (origem, cenario, id_documento, id_procedimento, motivo)
         SELECT origem, cenario, id_documento, id_procedimento, ?5
         FROM documento_simulado
         WHERE origem=?1 AND cenario=?2 AND id_documento=?3 AND id_procedimento=?4
         ON CONFLICT(origem, cenario, id_documento) DO NOTHING",
        params![origem.url_base(), cenario, id_documento, id_procedimento, motivo],
    )?;
    if inseridos == 1 {
        return Ok(Aplicacao::Aplicada);
    }
    match consultar(banco, origem, cenario, id_documento, id_procedimento)? {
        Some(Situacao::Cancelado { .. }) => Ok(Aplicacao::JaCancelado),
        Some(Situacao::Disponivel) => unreachable!("registro disponível teria sido inserido"),
        None => Ok(Aplicacao::DocumentoNaoEncontrado),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banco_cassettes::iniciar;
    use crate::documento_versoes::{DocumentoSimulado, Revisao, guardar};
    use std::num::NonZeroU32;

    #[test]
    fn cancelamento_usa_mesma_identidade_e_preserva_motivo_original() {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        guardar(&mut banco, "cenario-a", &DocumentoSimulado {
            instalacao: origem.clone(),
            id_documento: "42".into(), id_procedimento: "7".into(),
            numero_processo: "00001.000001/2026-01".into(), numero: "0011519".into(),
            tipo: "Despacho".into(), revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).unwrap(), usuario: "Ana".into(),
                unidade: "TESTE".into(), modificacao: "01/11/2026".into(),
                html_corpo: "<p>texto</p>".into(),
            }],
        }).unwrap();
        assert_eq!(consultar(&banco, &origem, "cenario-a", "42", "7").unwrap(), Some(Situacao::Disponivel));
        assert_eq!(aplicar(&banco, &origem, "cenario-a", "42", "7", "Motivo A").unwrap(), Aplicacao::Aplicada);
        assert_eq!(aplicar(&banco, &origem, "cenario-a", "42", "7", "Motivo B").unwrap(), Aplicacao::JaCancelado);
        assert_eq!(consultar(&banco, &origem, "cenario-a", "42", "7").unwrap(),
            Some(Situacao::Cancelado { motivo: "Motivo A".into() }));
        assert_eq!(consultar(&banco, &origem, "cenario-b", "42", "7").unwrap(), None);
        assert_eq!(consultar(&banco, &origem, "cenario-a", "42", "8").unwrap(), None);
    }
}
