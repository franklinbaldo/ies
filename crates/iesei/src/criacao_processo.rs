//! Tipos de processo oferecidos pelo cadastro simulado, com identidade no SQLite.
//! Observação real e hipóteses sintéticas devem permanecer identificadas pelo cenário.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TipoDeProcesso {
    pub id: String,
    pub nome: String,
    pub id_assunto: String,
    pub assunto: String,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    DadoInvalido,
}
impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}

fn validar(tipo: &TipoDeProcesso) -> bool {
    [&tipo.id, &tipo.id_assunto]
        .iter()
        .all(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        && [&tipo.nome, &tipo.assunto]
            .iter()
            .all(|s| !s.trim().is_empty() && !s.chars().any(char::is_control))
}

/// Registra uma escolha de tipo que a instalação ofereceu, sem criar processo.
///
/// # Errors
/// Recusa dados inválidos e preserva falhas SQLite.
pub fn registrar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    tipo: &TipoDeProcesso,
) -> Result<(), Falha> {
    if cenario.is_empty() || !validar(tipo) {
        return Err(Falha::DadoInvalido);
    }
    banco.execute(
        "INSERT INTO tipo_processo_simulado
         (origem, cenario, id_tipo, nome, id_assunto, assunto)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(origem, cenario, id_tipo) DO UPDATE SET
         nome=excluded.nome, id_assunto=excluded.id_assunto,
         assunto=excluded.assunto",
        params![
            instalacao.url_base(),
            cenario,
            tipo.id,
            tipo.nome,
            tipo.id_assunto,
            tipo.assunto
        ],
    )?;
    Ok(())
}

/// Nenhuma linha significa cenário de escolha não observado, não lista vazia.
///
/// # Errors
/// Propaga falhas de leitura do SQLite.
pub fn listar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> rusqlite::Result<Option<Vec<TipoDeProcesso>>> {
    let mut consulta = banco.prepare(
        "SELECT id_tipo, nome, id_assunto, assunto
         FROM tipo_processo_simulado WHERE origem=?1 AND cenario=?2
         ORDER BY id_tipo",
    )?;
    let tipos = consulta
        .query_map(params![instalacao.url_base(), cenario], |r| {
            Ok(TipoDeProcesso {
                id: r.get(0)?,
                nome: r.get(1)?,
                id_assunto: r.get(2)?,
                assunto: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok((!tipos.is_empty()).then_some(tipos))
}

/// Identificador de tipo informado na URL não autoriza inventar uma escolha.
///
/// # Errors
/// Preserva erro SQLite.
pub fn consultar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> rusqlite::Result<Option<TipoDeProcesso>> {
    banco
        .query_row(
            "SELECT id_tipo, nome, id_assunto, assunto FROM tipo_processo_simulado
         WHERE origem=?1 AND cenario=?2 AND id_tipo=?3",
            params![instalacao.url_base(), cenario, id],
            |r| {
                Ok(TipoDeProcesso {
                    id: r.get(0)?,
                    nome: r.get(1)?,
                    id_assunto: r.get(2)?,
                    assunto: r.get(3)?,
                })
            },
        )
        .optional()
}

/// Registro causal de uma criacao sintética. O ID/NUP nascem de UMA transacao.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Criado {
    pub id_procedimento: String,
    pub nup: String,
    pub tipo: TipoDeProcesso,
    pub especificacao: String,
}

/// Cria uma unica identidade na unidade ativa, com propriedades e abertura
/// compartilhadas com a arvore e o Controle. Esta hipotese aceita apenas
/// acesso publico, sem interessados: outras variantes seguem fechadas.
///
/// # Errors
/// Rejeita tipo ou entrada nao observada e propaga erros SQLite.
pub fn criar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id_tipo: &str,
    especificacao: &str,
    unidade: &str,
) -> Result<Criado, Falha> {
    if cenario.is_empty()
        || unidade.trim().is_empty()
        || especificacao.trim().is_empty()
        || especificacao.chars().any(char::is_control)
    {
        return Err(Falha::DadoInvalido);
    }
    let tx = banco.unchecked_transaction()?;
    let tipo = consultar(&tx, instalacao, cenario, id_tipo)?
        .ok_or(Falha::DadoInvalido)?;
    let origem = instalacao.url_base();
    tx.execute(
        "INSERT INTO processo_criado
         (origem, cenario, id_tipo, especificacao, nivel)
         VALUES (?1, ?2, ?3, ?4, 0)",
        params![origem, cenario, id_tipo, especificacao],
    )?;
    let sequencia = tx.last_insert_rowid();
    let id_procedimento = format!("9{sequencia}");
    let nup = format!("00000.{sequencia:06}/2026-01");
    tx.execute(
        "UPDATE processo_criado SET id_procedimento=?1 WHERE id=?2",
        params![id_procedimento, sequencia],
    )?;
    tx.execute(
        "INSERT INTO sim_processo
         (origem, cenario, id_procedimento, nup, unidade)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![origem, cenario, id_procedimento, nup, unidade],
    )?;
    tx.execute(
        "INSERT INTO processo_simulado
         (origem, cenario, id_procedimento, anotacao_origem)
         VALUES (?1, ?2, ?3, NULL)",
        params![origem, cenario, id_procedimento],
    )?;
    tx.execute(
        "INSERT INTO processo_abertura
         (origem, cenario, id_procedimento, posicao, unidade)
         VALUES (?1, ?2, ?3, 0, ?4)",
        params![origem, cenario, id_procedimento, unidade],
    )?;
    tx.commit()?;
    Ok(Criado {
        id_procedimento,
        nup,
        tipo,
        especificacao: especificacao.to_owned(),
    })
}

/// Relê a identidade e propriedades do processo criado a partir do SQLite.
/// Nenhuma correspondencia retorna None: ausencia nao cria documento ficticio.
///
/// # Errors
/// Preserva falhas de leitura SQLite.
pub fn ler_criado(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> rusqlite::Result<Option<Criado>> {
    banco
        .query_row(
            "SELECT c.id_procedimento, p.nup, t.id_tipo, t.nome,
                    t.id_assunto, t.assunto, c.especificacao
             FROM processo_criado c
             JOIN sim_processo p ON p.origem=c.origem AND p.cenario=c.cenario
               AND p.id_procedimento=c.id_procedimento
             JOIN tipo_processo_simulado t ON t.origem=c.origem AND t.cenario=c.cenario
               AND t.id_tipo=c.id_tipo
             WHERE c.origem=?1 AND c.cenario=?2 AND c.id_procedimento=?3",
            params![instalacao.url_base(), cenario, id],
            |linha| {
                Ok(Criado {
                    id_procedimento: linha.get(0)?,
                    nup: linha.get(1)?,
                    tipo: TipoDeProcesso {
                        id: linha.get(2)?,
                        nome: linha.get(3)?,
                        id_assunto: linha.get(4)?,
                        assunto: linha.get(5)?,
                    },
                    especificacao: linha.get(6)?,
                })
            },
        )
        .optional()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banco_cassettes::iniciar;

    #[test]
    fn registrar_processo_e_propriedades_num_unico_sqlite() {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        let tipo = TipoDeProcesso {
            id: "12".to_owned(),
            nome: "Ensaio".to_owned(),
            id_assunto: "21".to_owned(),
            assunto: "Assunto de Ensaio".to_owned(),
        };
        registrar(&banco, &origem, "cenario", &tipo).unwrap();
        assert!(ler_criado(&banco, &origem, "cenario", "91").unwrap().is_none());
        let novo = criar(&banco, &origem, "cenario", "12", "Nova origem", "U1").unwrap();
        assert_eq!(novo.tipo, tipo);
        assert_eq!(ler_criado(&banco, &origem, "cenario", &novo.id_procedimento).unwrap(), Some(novo.clone()));
        let processo = crate::processos::ler(&banco, &origem, "cenario", &novo.id_procedimento)
            .unwrap().unwrap();
        assert_eq!(processo.numero, novo.nup);
        assert_eq!(processo.aberto_em, ["U1"]);
        assert!(matches!(
            criar(&banco, &origem, "cenario", "999", "Invalido", "U1"),
            Err(Falha::DadoInvalido)
        ));
        assert_eq!(
            banco.query_row("SELECT COUNT(*) FROM sim_processo", [], |r| r.get::<_, i64>(0)).unwrap(),
            1
        );
    }
}
