//! Ciência processual sintética, independente de documento e sem credenciais.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Estado {
    Pendente,
    Registrada { id_atividade: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Resultado {
    Registrada { id_atividade: String },
    JaRegistrada,
    ProcessoNaoEncontrado,
}

/// Consulta o estado persistido de um processo conhecido.
///
/// # Errors
/// Propaga a falha SQLite original.
pub fn consultar(
    banco: &Connection, origem: &Instalacao, cenario: &str, processo: &str,
) -> rusqlite::Result<Option<Estado>> {
    let existe: bool = banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM sim_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3)",
        params![origem.url_base(), cenario, processo], |r| r.get(0),
    )?;
    if !existe {
        return Ok(None);
    }
    let atividade: Option<String> = banco.query_row(
        "SELECT id_atividade FROM ciencia_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem.url_base(), cenario, processo], |r| r.get(0),
    ).optional()?;
    Ok(Some(atividade.map_or(Estado::Pendente, |id_atividade| Estado::Registrada { id_atividade })))
}

/// Configuração sintética de visibilidade eventual, não evidência do SEI.
///
/// # Errors
/// Propaga erros SQL.
pub fn lista_invisivel(
    banco: &Connection, origem: &Instalacao, cenario: &str, processo: &str,
) -> rusqlite::Result<bool> {
    banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM ciencia_processo_lista_invisivel
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3)",
        params![origem.url_base(), cenario, processo], |r| r.get(0),
    )
}

/// A autoria só existe se o cenário conhece o usuário da sessão.
///
/// # Errors
/// Propaga erros SQL.
pub fn usuario_da_ciencia(
    banco: &Connection, origem: &Instalacao, cenario: &str, processo: &str,
) -> rusqlite::Result<Option<String>> {
    banco.query_row(
        "SELECT usuario FROM ciencia_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
        params![origem.url_base(), cenario, processo], |r| r.get::<_, Option<String>>(0),
    ).optional().map(Option::flatten)
}

/// Uma chamada mutadora registra no máximo uma atividade processual.
///
/// # Errors
/// Preserva erros SQLite; não há retry.
pub fn registrar(
    banco: &Connection, origem: &Instalacao, cenario: &str, processo: &str,
) -> rusqlite::Result<Resultado> {
    let inseridos = banco.execute(
        "INSERT INTO ciencia_processo
           (origem, cenario, id_procedimento, id_atividade, usuario)
         SELECT origem, cenario, id_procedimento,
            CAST(1 + COALESCE((
                SELECT MAX(id) FROM (
                    SELECT CAST(id_atividade AS INTEGER) AS id FROM ciencia_documental
                    WHERE origem=?1 AND cenario=?2
                    UNION ALL
                    SELECT CAST(id_atividade AS INTEGER) AS id FROM ciencia_processo
                    WHERE origem=?1 AND cenario=?2
                )
            ), 42456) AS TEXT),
            (SELECT usuario FROM sessao_usuario WHERE origem=?1 AND cenario=?2)
         FROM sim_processo
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3
         ON CONFLICT(origem, cenario, id_procedimento) DO NOTHING",
        params![origem.url_base(), cenario, processo],
    )?;
    if inseridos == 1 {
        let Some(Estado::Registrada { id_atividade }) = consultar(banco, origem, cenario, processo)? else {
            unreachable!("atividade recém-gravada deve ser consultável");
        };
        return Ok(Resultado::Registrada { id_atividade });
    }
    match consultar(banco, origem, cenario, processo)? {
        None => Ok(Resultado::ProcessoNaoEncontrado),
        Some(Estado::Registrada { .. }) => Ok(Resultado::JaRegistrada),
        Some(Estado::Pendente) => unreachable!("processo conhecido teria sido registrado"),
    }
}
