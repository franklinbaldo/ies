//! Observação explícita da atribuição e do menu da unidade no SQLite efêmero.
//! Hipótese sintética de BDD-264-01, não prova de paridade SEI.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Usuario {
    pub id: String,
    pub login: String,
    pub nome: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observacao {
    /// Vazio significa atribuição observada sem login; None na leitura é não observado.
    pub login_atual: String,
    pub usuarios: Vec<Usuario>,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    DadoInvalido,
    ProcessoNaoObservado,
}

impl From<rusqlite::Error> for Falha {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())
}

/// Associa à identidade já guardada a situação observada e as opções explícitas.
/// Regrava o menu de maneira atômica, sem criar ou atribuir processos.
///
/// # Errors
/// Recusa identidades inválidas ou não presentes e propaga SQLite.
pub fn registrar(
    banco: &mut Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
    observacao: &Observacao,
) -> Result<(), Falha> {
    if !id_valido(id)
        || cenario.is_empty()
        || unidade.is_empty()
        || observacao.usuarios.iter().any(|u| {
            !id_valido(&u.id)
                || u.login.trim() != u.login
                || u.login.is_empty()
                || u.nome.trim().is_empty()
        })
    {
        return Err(Falha::DadoInvalido);
    }
    for (i, usuario) in observacao.usuarios.iter().enumerate() {
        if observacao.usuarios[..i].iter().any(|x| x.id == usuario.id) {
            return Err(Falha::DadoInvalido);
        }
    }
    let tx = banco.transaction()?;
    let existe: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM processo_abertura WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4)",
        params![origem.url_base(), cenario, id, unidade],
        |r| r.get(0),
    )?;
    if !existe {
        return Err(Falha::ProcessoNaoObservado);
    }
    tx.execute(
        "INSERT INTO processo_atribuicao_observada (origem,cenario,id_procedimento,unidade,login_atual)
         VALUES (?1,?2,?3,?4,?5)
         ON CONFLICT(origem,cenario,id_procedimento,unidade)
         DO UPDATE SET login_atual=excluded.login_atual",
        params![origem.url_base(), cenario, id, unidade, observacao.login_atual],
    )?;
    tx.execute(
        "DELETE FROM processo_atribuicao_opcao WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4",
        params![origem.url_base(), cenario, id, unidade],
    )?;
    for usuario in &observacao.usuarios {
        tx.execute(
            "INSERT INTO processo_atribuicao_opcao (origem,cenario,id_procedimento,unidade,id_usuario,login,nome)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![origem.url_base(), cenario, id, unidade, usuario.id, usuario.login, usuario.nome],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Lê o que foi explicitamente registrado para a unidade ativa.
/// None não é uma lista vazia nem autorização para atribuir.
///
/// # Errors
/// Preserva falha SQLite, sem inventar resultado.
pub fn ler(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
) -> Result<Option<Observacao>, Falha> {
    let login_atual: Option<String> = banco
        .query_row(
            "SELECT login_atual FROM processo_atribuicao_observada
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4",
            params![origem.url_base(), cenario, id, unidade],
            |r| r.get(0),
        )
        .optional()?;
    let Some(login_atual) = login_atual else {
        return Ok(None);
    };
    let mut stmt = banco.prepare(
        "SELECT id_usuario, login, nome FROM processo_atribuicao_opcao
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4
         ORDER BY id_usuario",
    )?;
    let usuarios = stmt
        .query_map(params![origem.url_base(), cenario, id, unidade], |r| {
            Ok(Usuario {
                id: r.get(0)?,
                login: r.get(1)?,
                nome: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some(Observacao {
        login_atual,
        usuarios,
    }))
}
