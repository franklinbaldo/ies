//! Relação de anexação observável no banco efêmero do anti-IES.
//! A direção pai -> anexado não é um relacionamento bilateral reversível.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anexado {
    pub id: String,
    pub numero: String,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroAnexacoes {
    Sqlite(#[from] rusqlite::Error),
    DadoInvalido,
    ProcessoNaoObservado,
    JaAnexado,
}

fn valido(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())
}

fn existe(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> rusqlite::Result<bool> {
    banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM sim_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3)",
        params![origem.url_base(), cenario, id], |r| r.get(0)
    )
}

/// `None`: o principal não existe no cenário; `Some([])`: nenhum anexado.
///
/// # Errors
/// Propaga problemas SQL sem converter falta de evidência em lista vazia.
pub fn listar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    principal: &str,
) -> Result<Option<Vec<Anexado>>, ErroAnexacoes> {
    if !valido(principal) || cenario.is_empty() {
        return Err(ErroAnexacoes::DadoInvalido);
    }
    if !existe(banco, origem, cenario, principal)? {
        return Ok(None);
    }
    let mut stmt = banco.prepare(
        "SELECT a.id_anexado, p.nup FROM sim_anexacao a
         JOIN sim_processo p ON p.origem=a.origem AND p.cenario=a.cenario
              AND p.id_procedimento=a.id_anexado
         WHERE a.origem=?1 AND a.cenario=?2 AND a.id_principal=?3
         ORDER BY a.id_anexado",
    )?;
    let valores = stmt
        .query_map(params![origem.url_base(), cenario, principal], |r| {
            Ok(Anexado {
                id: r.get(0)?,
                numero: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some(valores))
}

/// Grava uma única anexação, recusando ciclos e pai já anexado.
///
/// # Errors
/// Mantém integridade relacional mesmo em tentativas repetidas ou concorrentes.
pub fn anexar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    principal: &str,
    anexado: &str,
) -> Result<(), ErroAnexacoes> {
    if cenario.is_empty() || !valido(principal) || !valido(anexado) || principal == anexado {
        return Err(ErroAnexacoes::DadoInvalido);
    }
    if !existe(banco, origem, cenario, principal)? || !existe(banco, origem, cenario, anexado)? {
        return Err(ErroAnexacoes::ProcessoNaoObservado);
    }
    let envolvidos: i64 = banco.query_row(
        "SELECT COUNT(*) FROM sim_anexacao WHERE origem=?1 AND cenario=?2
           AND (id_anexado=?3 OR id_anexado=?4 OR id_principal=?4)",
        params![origem.url_base(), cenario, principal, anexado],
        |r| r.get(0),
    )?;
    if envolvidos != 0 {
        return Err(ErroAnexacoes::JaAnexado);
    }
    match banco.execute(
        "INSERT INTO sim_anexacao (origem, cenario, id_principal, id_anexado)
         VALUES (?1, ?2, ?3, ?4)",
        params![origem.url_base(), cenario, principal, anexado],
    ) {
        Ok(1) => Ok(()),
        Ok(_) => Err(ErroAnexacoes::DadoInvalido),
        // A origem do erro SQL deve sobreviver: UNIQUE, FOREIGN KEY e I/O
        // não são sinônimos de "já anexado". A duplicidade observada acima
        // conserva sua própria variante tipada.
        Err(e) => Err(ErroAnexacoes::Sqlite(e)),
    }
}
