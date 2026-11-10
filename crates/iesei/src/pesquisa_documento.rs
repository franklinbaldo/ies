//! Resultado negativo explicitamente observado no cenário de busca documental.
//! Ausência de linha não comprova ausência de documentos no SEI real.
use crate::instalacao::Instalacao;
use crate::observacao::UnidadeExibida;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resultado {
    SemResultado,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroPesquisaDocumento {
    Sqlite(#[from] rusqlite::Error),
    EntradaInvalida,
}

fn valido(cenario: &str, termo: &str) -> bool {
    !cenario.is_empty() && !termo.trim().is_empty() && !termo.chars().any(char::is_control)
}

/// Registra somente uma busca cuja resposta negativa foi observada no cenário.
///
/// # Errors
/// Retorna a causa SQLite ou a invalidez do termo/contexto.
pub fn registrar_sem_resultado(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    termo: &str,
) -> Result<(), ErroPesquisaDocumento> {
    if !valido(cenario, termo) {
        return Err(ErroPesquisaDocumento::EntradaInvalida);
    }
    banco.execute(
        "INSERT INTO pesquisa_documento_sem_resultado
         (origem, cenario, unidade, termo) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(origem, cenario, unidade, termo) DO NOTHING",
        params![instalacao.url_base(), cenario, unidade.as_str(), termo],
    )?;
    Ok(())
}

/// `None` significa desconhecido, não uma pesquisa que encontrou zero itens.
///
/// # Errors
/// Propaga erro SQL e recusa contexto inválido.
pub fn consultar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    termo: &str,
) -> Result<Option<Resultado>, ErroPesquisaDocumento> {
    if !valido(cenario, termo) {
        return Err(ErroPesquisaDocumento::EntradaInvalida);
    }
    let achado: Option<i64> = banco
        .query_row(
            "SELECT 1 FROM pesquisa_documento_sem_resultado
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND termo=?4",
            params![instalacao.url_base(), cenario, unidade.as_str(), termo],
            |linha| linha.get(0),
        )
        .optional()?;
    Ok(achado.map(|_| Resultado::SemResultado))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registro_idempotente_e_ausencia_nao_inferida() {
        let banco = Connection::open_in_memory().unwrap();
        crate::banco_cassettes::iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let unidade = UnidadeExibida::nova("TESTE").unwrap();
        assert!(
            consultar(&banco, &origem, "cenario", &unidade, "sem")
                .unwrap()
                .is_none()
        );
        registrar_sem_resultado(&banco, &origem, "cenario", &unidade, "sem").unwrap();
        registrar_sem_resultado(&banco, &origem, "cenario", &unidade, "sem").unwrap();
        assert_eq!(
            consultar(&banco, &origem, "cenario", &unidade, "sem").unwrap(),
            Some(Resultado::SemResultado)
        );
        assert!(
            consultar(&banco, &origem, "cenario", &unidade, "outro")
                .unwrap()
                .is_none()
        );
        assert!(
            consultar(
                &banco,
                &origem,
                "cenario",
                &UnidadeExibida::nova("OUTRA").unwrap(),
                "sem"
            )
            .unwrap()
            .is_none()
        );
    }
}

/// Um documento da página observada. A identidade e o conteúdo vivem em
/// `documento_simulado`; aqui ficam somente os dados da listagem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinhaPaginada {
    pub id_documento: String,
    pub rotulo: String,
    pub unidade: Option<String>,
    pub inclusao: Option<String>,
    pub id_anexo: Option<String>,
}

/// A faixa vem da barra da resposta; pode conter menos linhas que a faixa.
/// Uma página não registrada é desconhecida, nunca vazia por inferência.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginaObservada {
    pub pagina: u32,
    pub inicio: u64,
    pub fim: u64,
    pub total_declarado: u64,
    pub linhas: Vec<LinhaPaginada>,
}

fn validar_pagina(pagina: &PaginaObservada) -> bool {
    use std::collections::HashSet;
    let mut ids = HashSet::new();
    pagina.pagina > 0
        && pagina.inicio > 0
        && pagina.fim >= pagina.inicio
        && pagina.total_declarado >= pagina.fim
        && u64::try_from(pagina.linhas.len()).is_ok_and(|n| n <= pagina.fim - pagina.inicio + 1)
        && pagina.linhas.iter().all(|linha| {
            !linha.id_documento.is_empty()
                && linha.id_documento.bytes().all(|b| b.is_ascii_digit())
                && !linha.rotulo.trim().is_empty()
                && ids.insert(linha.id_documento.as_str())
                && linha
                    .id_anexo
                    .as_ref()
                    .is_none_or(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
        })
}

fn inteiro(numero: u64) -> Result<i64, ErroPesquisaDocumento> {
    i64::try_from(numero).map_err(|_| ErroPesquisaDocumento::EntradaInvalida)
}

/// Persiste a página e suas linhas como uma observação substituível, atômica
/// e idempotente, sem materializar documentos novos.
///
/// # Errors
/// Recusa faixa/linhas inconsistentes e propaga a causa SQLite.
pub fn registrar_pagina(
    banco: &mut Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    termo: &str,
    pagina: &PaginaObservada,
) -> Result<(), ErroPesquisaDocumento> {
    if !valido(cenario, termo) || !validar_pagina(pagina) {
        return Err(ErroPesquisaDocumento::EntradaInvalida);
    }
    let origem = instalacao.url_base();
    let numero = i64::from(pagina.pagina);
    let inicio = inteiro(pagina.inicio)?;
    let fim = inteiro(pagina.fim)?;
    let total = inteiro(pagina.total_declarado)?;
    let tx = banco.transaction()?;
    tx.execute(
        "INSERT INTO pesquisa_documento_pagina
         (origem, cenario, unidade, termo, pagina, inicio, fim, total)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(origem, cenario, unidade, termo, pagina)
         DO UPDATE SET inicio=excluded.inicio, fim=excluded.fim, total=excluded.total",
        params![
            origem,
            cenario,
            unidade.as_str(),
            termo,
            numero,
            inicio,
            fim,
            total
        ],
    )?;
    tx.execute(
        "DELETE FROM pesquisa_documento_pagina_linha
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND termo=?4 AND pagina=?5",
        params![origem, cenario, unidade.as_str(), termo, numero],
    )?;
    for (indice, linha) in pagina.linhas.iter().enumerate() {
        tx.execute(
            "INSERT INTO pesquisa_documento_pagina_linha
             (origem, cenario, unidade, termo, pagina, posicao,
              id_documento, rotulo, unidade_linha, inclusao, id_anexo)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                origem,
                cenario,
                unidade.as_str(),
                termo,
                numero,
                i64::try_from(indice).map_err(|_| ErroPesquisaDocumento::EntradaInvalida)?,
                linha.id_documento,
                linha.rotulo,
                linha.unidade,
                linha.inclusao,
                linha.id_anexo,
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Recupera apenas a página explicitamente registrada neste cenário.
///
/// # Errors
/// Propaga falha SQLite e recusa números armazenados inconsistentes.
pub fn consultar_pagina(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    termo: &str,
    numero: u32,
) -> Result<Option<PaginaObservada>, ErroPesquisaDocumento> {
    if !valido(cenario, termo) || numero == 0 {
        return Err(ErroPesquisaDocumento::EntradaInvalida);
    }
    let origem = instalacao.url_base();
    let dados: Option<(i64, i64, i64)> = banco
        .query_row(
            "SELECT inicio, fim, total FROM pesquisa_documento_pagina
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND termo=?4 AND pagina=?5",
            params![origem, cenario, unidade.as_str(), termo, i64::from(numero)],
            |linha| Ok((linha.get(0)?, linha.get(1)?, linha.get(2)?)),
        )
        .optional()?;
    let Some((inicio, fim, total)) = dados else {
        return Ok(None);
    };
    let mut consulta = banco.prepare(
        "SELECT id_documento, rotulo, unidade_linha, inclusao, id_anexo
         FROM pesquisa_documento_pagina_linha
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND termo=?4 AND pagina=?5
         ORDER BY posicao",
    )?;
    let registros = consulta.query_map(
        params![origem, cenario, unidade.as_str(), termo, i64::from(numero)],
        |r| {
            Ok(LinhaPaginada {
                id_documento: r.get(0)?,
                rotulo: r.get(1)?,
                unidade: r.get(2)?,
                inclusao: r.get(3)?,
                id_anexo: r.get(4)?,
            })
        },
    )?;
    let linhas = registros.collect::<rusqlite::Result<Vec<_>>>()?;
    let pagina = PaginaObservada {
        pagina: numero,
        inicio: u64::try_from(inicio).map_err(|_| ErroPesquisaDocumento::EntradaInvalida)?,
        fim: u64::try_from(fim).map_err(|_| ErroPesquisaDocumento::EntradaInvalida)?,
        total_declarado: u64::try_from(total)
            .map_err(|_| ErroPesquisaDocumento::EntradaInvalida)?,
        linhas,
    };
    if !validar_pagina(&pagina) {
        return Err(ErroPesquisaDocumento::EntradaInvalida);
    }
    Ok(Some(pagina))
}

/// Resolve o deslocamento publicado pelo formulário (`hdnInicio`).
/// Não adivinha uma página se ela não foi armazenada.
///
/// # Errors
/// Propaga falha SQLite e deslocamento fora da capacidade do banco.
pub fn consultar_por_inicio(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    unidade: &UnidadeExibida,
    termo: &str,
    deslocamento: u64,
) -> Result<Option<PaginaObservada>, ErroPesquisaDocumento> {
    let inicio = inteiro(
        deslocamento
            .checked_add(1)
            .ok_or(ErroPesquisaDocumento::EntradaInvalida)?,
    )?;
    let numero: Option<i64> = banco
        .query_row(
            "SELECT pagina FROM pesquisa_documento_pagina
         WHERE origem=?1 AND cenario=?2 AND unidade=?3 AND termo=?4 AND inicio=?5",
            params![
                instalacao.url_base(),
                cenario,
                unidade.as_str(),
                termo,
                inicio
            ],
            |r| r.get(0),
        )
        .optional()?;
    numero
        .map(|n| {
            let n = u32::try_from(n).map_err(|_| ErroPesquisaDocumento::EntradaInvalida)?;
            consultar_pagina(banco, instalacao, cenario, unidade, termo, n)?
                .ok_or(ErroPesquisaDocumento::EntradaInvalida)
        })
        .transpose()
}
