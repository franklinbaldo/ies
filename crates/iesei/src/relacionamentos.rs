//! Relacionamentos observados do processo; ausência de observação não é lista vazia.
//! Hipótese funcional do anti-IES, não validação contra SEI real.
use crate::instalacao::Instalacao;
use rusqlite::{Connection, params};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relacionamento {
    pub id: String,
    pub numero: String,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    DadoInvalido,
    ProcessoNaoObservado,
    DestinoAmbiguo,
}
impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}
fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|c| c.is_ascii_digit())
}
fn existe(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> rusqlite::Result<bool> {
    banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM sim_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3)",
        params![origem.url_base(), cenario, id], |linha| linha.get(0),
    )
}
/// Marca a lista como efetivamente examinada; não presume relações vazias.
/// A observação é idempotente e não substitui vínculos já registrados.
///
/// # Errors
/// Recusa processo não presente no cenário e preserva erro SQLite.
pub fn observar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<(), Falha> {
    if cenario.is_empty() || !id_valido(id) {
        return Err(Falha::DadoInvalido);
    }
    if !existe(banco, origem, cenario, id)? {
        return Err(Falha::ProcessoNaoObservado);
    }
    banco.execute(
        "INSERT OR IGNORE INTO processo_relacionamentos_observados (origem, cenario, id_procedimento) VALUES (?1, ?2, ?3)",
        params![origem.url_base(), cenario, id],
    )?;
    Ok(())
}
/// Distingue lista não observada de observação vazia.
///
/// # Errors
/// Propaga falhas SQLite sem transformar ausência em vazio.
pub fn observado(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<bool, Falha> {
    Ok(banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM processo_relacionamentos_observados WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3)",
        params![origem.url_base(), cenario, id], |r| r.get(0),
    )?)
}
/// Registra vínculo visto no cenário, sem criar identidade de processo.
///
/// # Errors
/// Recusa destino inexistente/autorrelação ou observação ausente.
pub fn registrar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    destino: &str,
) -> Result<(), Falha> {
    if id == destino
        || !id_valido(id)
        || !id_valido(destino)
        || !observado(banco, origem, cenario, id)?
    {
        return Err(Falha::DadoInvalido);
    }
    if !existe(banco, origem, cenario, destino)? {
        return Err(Falha::ProcessoNaoObservado);
    }
    banco.execute(
        "INSERT OR IGNORE INTO processo_relacao (origem, cenario, id_procedimento, id_destino) VALUES (?1, ?2, ?3, ?4)",
        params![origem.url_base(), cenario, id, destino],
    )?;
    Ok(())
}
/// Lista vínculos únicos já observados; `None` indica não observado.
///
/// # Errors
/// Não esconde falhas SQL ou links órfãos.
pub fn listar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Option<Vec<Relacionamento>>, Falha> {
    if !observado(banco, origem, cenario, id)? {
        return Ok(None);
    }
    let mut consulta = banco.prepare(
        "SELECT d.id_procedimento, d.nup
         FROM processo_relacao r JOIN sim_processo d
           ON d.origem=r.origem AND d.cenario=r.cenario AND d.id_procedimento=r.id_destino
         WHERE r.origem=?1 AND r.cenario=?2 AND r.id_procedimento=?3
         ORDER BY d.id_procedimento",
    )?;
    let linhas = consulta
        .query_map(params![origem.url_base(), cenario, id], |r| {
            Ok(Relacionamento {
                id: r.get(0)?,
                numero: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some(linhas))
}
fn forma(numero: &str) -> String {
    numero
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
/// Consulta o destino do AJAX pelo NUP já guardado, sem inventar tipo documental.
///
/// # Errors
/// Falha em duplicatas equivalentes, preservando identidade opaca.
pub fn identificar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    numero: &str,
) -> Result<Option<Relacionamento>, Falha> {
    let pedido = forma(numero);
    if pedido.is_empty() {
        return Err(Falha::DadoInvalido);
    }
    let mut consulta = banco
        .prepare("SELECT id_procedimento, nup FROM sim_processo WHERE origem=?1 AND cenario=?2")?;
    let linhas = consulta
        .query_map(params![origem.url_base(), cenario], |r| {
            Ok(Relacionamento {
                id: r.get(0)?,
                numero: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut encontrados = linhas.into_iter().filter(|d| forma(&d.numero) == pedido);
    let primeiro = encontrados.next();
    if encontrados.next().is_some() {
        return Err(Falha::DestinoAmbiguo);
    }
    Ok(primeiro)
}
/// Não oferece AJAX de um cenário que nunca observou nenhum relacionamento.
///
/// # Errors
/// Propaga falha SQLite.
pub fn existe_contexto(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Result<bool, Falha> {
    Ok(banco.query_row(
        "SELECT EXISTS(SELECT 1 FROM processo_relacionamentos_observados WHERE origem=?1 AND cenario=?2)",
        params![origem.url_base(), cenario], |r| r.get(0),
    )?)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::banco_cassettes::iniciar;
    use crate::processos::{ProcessoSimulado, guardar};
    #[test]
    fn nao_observado_diferente_de_observado_vazio_e_idempotente() {
        let mut db = Connection::open_in_memory().unwrap();
        iniciar(&db).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        for (id, numero) in [("10", "P-01"), ("22", "P-02")] {
            guardar(
                &mut db,
                &origem,
                "caso",
                &ProcessoSimulado {
                    id: id.into(),
                    numero: numero.into(),
                    unidade_origem: "TESTE".into(),
                    aberto_em: vec!["TESTE".into()],
                    anotacao_origem: None,
                    destinos: vec![],
                },
            )
            .unwrap();
        }
        assert!(listar(&db, &origem, "caso", "10").unwrap().is_none());
        observar(&db, &origem, "caso", "10").unwrap();
        observar(&db, &origem, "caso", "10").unwrap();
        assert_eq!(listar(&db, &origem, "caso", "10").unwrap(), Some(vec![]));
        registrar(&db, &origem, "caso", "10", "22").unwrap();
        registrar(&db, &origem, "caso", "10", "22").unwrap();
        assert_eq!(
            listar(&db, &origem, "caso", "10").unwrap().unwrap().len(),
            1
        );
        // A relação aparece nos dois processos quando ambos foram observados.
        observar(&db, &origem, "caso", "22").unwrap();
        assert_eq!(
            listar(&db, &origem, "caso", "22").unwrap(),
            Some(vec![Relacionamento {
                id: "10".into(),
                numero: "P-01".into(),
            }])
        );
        assert!(listar(&db, &origem, "outro", "10").unwrap().is_none());
        assert_eq!(
            identificar(&db, &origem, "caso", "P02")
                .unwrap()
                .unwrap()
                .id,
            "22"
        );
    }
}
