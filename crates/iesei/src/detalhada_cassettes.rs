//! Página detalhada D do Controle no SQLite efêmero compartilhado.
//! A faixa observada pode começar depois da primeira página.
use crate::instalacao::Instalacao;
use crate::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinhaDetalhada {
    pub id: String,
    pub numero: String,
    pub visualizado: Option<bool>,
    pub sigiloso: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControleDetalhado {
    pub instalacao: Instalacao,
    pub unidade: UnidadeExibida,
    pub processos: TabelaObservada<LinhaDetalhada>,
}

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    ObservacaoInvalida,
}

impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
}

fn inteiro(valor: usize) -> Result<i64, Falha> {
    i64::try_from(valor).map_err(|_| Falha::ObservacaoInvalida)
}

fn natural(valor: i64) -> Result<usize, Falha> {
    usize::try_from(valor).map_err(|_| Falha::ObservacaoInvalida)
}

/// Persiste a página D ativa e desativa a R anterior no mesmo cenário.
/// # Errors
/// Recusa totais, faixas, IDs e estados incoerentes; propaga erro SQLite.
pub fn guardar(
    banco: &mut Connection,
    cenario: &str,
    controle: &ControleDetalhado,
) -> Result<(), Falha> {
    let tabela = &controle.processos;
    let total = tabela.total_declarado.ok_or(Falha::ObservacaoInvalida)?;
    if cenario.is_empty() || !tabela.pagina_verificavel() {
        return Err(Falha::ObservacaoInvalida);
    }
    for linha in &tabela.linhas {
        if linha.id.is_empty()
            || !linha.id.bytes().all(|b| b.is_ascii_digit())
            || linha.numero.is_empty()
            || linha.visualizado.is_some() != linha.sigiloso.is_some()
        {
            return Err(Falha::ObservacaoInvalida);
        }
    }
    let origem = controle.instalacao.url_base();
    let inicio = tabela
        .faixa_observada
        .map(|f| inteiro(f.inicio()))
        .transpose()?;
    let fim = tabela
        .faixa_observada
        .map(|f| inteiro(f.fim()))
        .transpose()?;
    let tx = banco.transaction()?;
    tx.execute(
        "INSERT INTO controle_detalhado (origem, cenario, unidade, total, inicio, fim)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(origem, cenario) DO UPDATE SET
         unidade=excluded.unidade, total=excluded.total,
         inicio=excluded.inicio, fim=excluded.fim",
        params![
            origem,
            cenario,
            controle.unidade.as_str(),
            inteiro(total)?,
            inicio,
            fim
        ],
    )?;
    tx.execute(
        "DELETE FROM controle_detalhado_linha WHERE origem=?1 AND cenario=?2",
        params![origem, cenario],
    )?;
    for (posicao, linha) in tabela.linhas.iter().enumerate() {
        tx.execute(
            "INSERT INTO controle_detalhado_linha
             (origem, cenario, posicao, id, numero, visualizado, sigiloso)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                origem,
                cenario,
                inteiro(posicao)?,
                linha.id,
                linha.numero,
                linha.visualizado,
                linha.sigiloso
            ],
        )?;
    }
    // A visão ativa é uma só, não dois snapshots conflitantes.
    tx.execute(
        "DELETE FROM controle_linha WHERE origem=?1 AND cenario=?2",
        params![origem, cenario],
    )?;
    tx.execute(
        "DELETE FROM controle_resumido WHERE origem=?1 AND cenario=?2",
        params![origem, cenario],
    )?;
    tx.commit()?;
    Ok(())
}

/// Reconstrói apenas as posições efetivamente persistidas no SQLite.
/// # Errors
/// Propaga erro SQLite ou recusa página incoerente sem inferir cobertura.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<Option<ControleDetalhado>, Falha> {
    let origem = instalacao.url_base();
    let meta: Option<(String, i64, Option<i64>, Option<i64>)> = banco
        .query_row(
            "SELECT unidade, total, inicio, fim FROM controle_detalhado
             WHERE origem=?1 AND cenario=?2",
            params![origem, cenario],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let Some((nome, total, inicio, fim)) = meta else {
        return Ok(None);
    };
    let unidade = UnidadeExibida::nova(&nome).ok_or(Falha::ObservacaoInvalida)?;
    let faixa = match (inicio, fim) {
        (None, None) => None,
        (Some(i), Some(f)) => {
            Some(FaixaObservada::nova(natural(i)?, natural(f)?).ok_or(Falha::ObservacaoInvalida)?)
        }
        _ => return Err(Falha::ObservacaoInvalida),
    };
    let mut consulta = banco.prepare(
        "SELECT id, numero, visualizado, sigiloso
         FROM controle_detalhado_linha WHERE origem=?1 AND cenario=?2
         ORDER BY posicao",
    )?;
    let linhas = consulta
        .query_map(params![origem, cenario], |r| {
            Ok(LinhaDetalhada {
                id: r.get(0)?,
                numero: r.get(1)?,
                visualizado: r.get(2)?,
                sigiloso: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let tabela = TabelaObservada::nova(Some(natural(total)?), faixa, linhas)
        .map_err(|_| Falha::ObservacaoInvalida)?;
    if !tabela.pagina_verificavel() {
        return Err(Falha::ObservacaoInvalida);
    }
    Ok(Some(ControleDetalhado {
        instalacao: instalacao.clone(),
        unidade,
        processos: tabela,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banco_cassettes::{self, ControleResumido};

    fn instalacao() -> Instalacao {
        Instalacao::interpretar("https://sei.test/sei").unwrap()
    }

    fn resumo(instalacao: &Instalacao) -> ControleResumido {
        ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        }
    }

    fn detalhe(instalacao: &Instalacao) -> ControleDetalhado {
        ControleDetalhado {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            processos: TabelaObservada::nova(
                Some(150),
                FaixaObservada::nova(101, 101),
                vec![LinhaDetalhada {
                    id: "901".to_owned(),
                    numero: "000901/2026".to_owned(),
                    visualizado: Some(true),
                    sigiloso: Some(false),
                }],
            )
            .unwrap(),
        }
    }

    #[test]
    fn alternar_visao_no_mesmo_estado_nao_mantem_snapshot_obsoleto() {
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let origem = instalacao();
        banco_cassettes::guardar(&mut banco, "sessao", &resumo(&origem)).unwrap();

        guardar(&mut banco, "sessao", &detalhe(&origem)).unwrap();
        assert!(banco_cassettes::ler(&banco, &origem, "sessao").unwrap().is_none());
        let atual = ler(&banco, &origem, "sessao").unwrap().unwrap();
        assert_eq!(atual.processos.faixa_observada.unwrap().inicio(), 101);
        assert_eq!(atual.processos.total_declarado, Some(150));

        banco_cassettes::guardar(&mut banco, "sessao", &resumo(&origem)).unwrap();
        assert!(ler(&banco, &origem, "sessao").unwrap().is_none());
        assert!(banco_cassettes::ler(&banco, &origem, "sessao").unwrap().is_some());
    }

    #[test]
    fn pagina_detalhada_sem_faixa_nao_vira_cobertura() {
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let origem = instalacao();
        let mut incompleta = detalhe(&origem);
        incompleta.processos.faixa_observada = None;
        assert!(matches!(
            guardar(&mut banco, "sessao", &incompleta),
            Err(Falha::ObservacaoInvalida)
        ));
        assert!(ler(&banco, &origem, "sessao").unwrap().is_none());
    }
}
