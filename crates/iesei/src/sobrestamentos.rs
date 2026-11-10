//! Estado de sobrestamento do processo, separado da unidade de tramitação.
//! Hipótese testável no anti-IES, não transcrição de uma resposta live.
use crate::{instalacao::Instalacao, processos};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sobrestamento {
    pub unidade: String,
    pub motivo: String,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    Processo(processos::Falha),
    ProcessoNaoEncontrado,
    UnidadeNaoAberta,
    JaSobrestado,
    NaoSobrestado,
    OutraUnidade,
    DadoInvalido,
}
impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}
impl From<processos::Falha> for Falha {
    fn from(erro: processos::Falha) -> Self {
        Self::Processo(erro)
    }
}
fn identidade_valida(id: &str, unidade: &str) -> bool {
    !id.is_empty() && id.bytes().all(|c| c.is_ascii_digit()) && !unidade.is_empty()
}
/// Consulta a unidade e o motivo persistidos, sem confundir ausência com outra unidade.
///
/// # Errors
/// Preserva a causa SQLite.
pub fn consultar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> rusqlite::Result<Option<Sobrestamento>> {
    banco
        .query_row(
            "SELECT unidade, motivo FROM sim_sobrestamento
             WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3",
            params![origem.url_base(), cenario, id],
            |linha| {
                Ok(Sobrestamento {
                    unidade: linha.get(0)?,
                    motivo: linha.get(1)?,
                })
            },
        )
        .optional()
}
/// Uma confirmação sobrestá uma única vez; a abertura permanece registrada
/// para ser publicada novamente depois da retomada.
///
/// # Errors
/// Distingue processo inexistente, unidade fechada, estado já sobrestado
/// e erro SQLite.
pub fn sobrestar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
    motivo: &str,
) -> Result<(), Falha> {
    if !identidade_valida(id, unidade) || motivo.trim().is_empty() {
        return Err(Falha::DadoInvalido);
    }
    let tx = banco.unchecked_transaction()?;
    let processo = processos::ler(&tx, origem, cenario, id)?.ok_or(Falha::ProcessoNaoEncontrado)?;
    if consultar(&tx, origem, cenario, id)?.is_some() {
        return Err(Falha::JaSobrestado);
    }
    if !processo.aberto_em.iter().any(|u| u == unidade) {
        return Err(Falha::UnidadeNaoAberta);
    }
    tx.execute(
        "INSERT INTO sim_sobrestamento
         (origem, cenario, id_procedimento, unidade, motivo)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![origem.url_base(), cenario, id, unidade, motivo],
    )?;
    tx.commit()?;
    Ok(())
}
/// Uma confirmação retira o sobrestamento da unidade atual sem retry.
///
/// # Errors
/// Preserva processo, estado anterior e causa SQLite.
pub fn retomar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
) -> Result<(), Falha> {
    if !identidade_valida(id, unidade) {
        return Err(Falha::DadoInvalido);
    }
    let tx = banco.unchecked_transaction()?;
    processos::ler(&tx, origem, cenario, id)?.ok_or(Falha::ProcessoNaoEncontrado)?;
    let estado = consultar(&tx, origem, cenario, id)?.ok_or(Falha::NaoSobrestado)?;
    if estado.unidade != unidade {
        return Err(Falha::OutraUnidade);
    }
    tx.execute(
        "DELETE FROM sim_sobrestamento
         WHERE origem=?1 AND cenario=?2 AND id_procedimento=?3 AND unidade=?4",
        params![origem.url_base(), cenario, id, unidade],
    )?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banco_cassettes;
    use crate::processos::ProcessoSimulado;

    #[test]
    fn uma_transicao_por_confirmacao_com_releitura_e_isolamento() {
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.teste/sei").unwrap();
        let processo = ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        };
        processos::guardar(&mut banco, &origem, "c1", &processo).unwrap();
        assert_eq!(consultar(&banco, &origem, "c1", "10").unwrap(), None);
        sobrestar(&banco, &origem, "c1", "10", "TESTE", "Aguardando").unwrap();
        assert_eq!(
            consultar(&banco, &origem, "c1", "10").unwrap(),
            Some(Sobrestamento {
                unidade: "TESTE".into(),
                motivo: "Aguardando".into()
            })
        );
        assert!(matches!(
            sobrestar(&banco, &origem, "c1", "10", "TESTE", "Outra"),
            Err(Falha::JaSobrestado)
        ));
        assert!(matches!(
            retomar(&banco, &origem, "c1", "10", "OUTRA"),
            Err(Falha::OutraUnidade)
        ));
        assert_eq!(consultar(&banco, &origem, "c2", "10").unwrap(), None);
        retomar(&banco, &origem, "c1", "10", "TESTE").unwrap();
        assert_eq!(consultar(&banco, &origem, "c1", "10").unwrap(), None);
        assert!(matches!(
            retomar(&banco, &origem, "c1", "10", "TESTE"),
            Err(Falha::NaoSobrestado)
        ));
        assert_eq!(
            processos::ler(&banco, &origem, "c1", "10")
                .unwrap()
                .unwrap()
                .aberto_em,
            vec!["TESTE"]
        );
    }
}
