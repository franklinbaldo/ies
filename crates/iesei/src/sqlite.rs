//! Projeção privada de observações, distinta das tabelas de sessão.
use crate::instalacao::Instalacao;
use crate::observacao::{UnidadeExibida, VisaoDoControle};
use rusqlite::{Connection, OptionalExtension, params};

/// 3 → 4: um fato por instalação e geração de sessão. Nenhuma URL/cookie.
pub const MIGRACAO_CONTROLE: &str = "CREATE TABLE controle_unidade (
    origem TEXT NOT NULL,
    contexto TEXT NOT NULL,
    unidade_exibida TEXT NOT NULL,
    visao TEXT NOT NULL CHECK (visao IN ('R', 'D')),
    observada_em REAL NOT NULL,
    PRIMARY KEY (origem, contexto)
) STRICT;";

#[derive(Clone, Debug, PartialEq)]
pub struct UnidadeObservada {
    pub unidade: UnidadeExibida,
    pub visao: VisaoDoControle,
    pub observada_em: f64,
}

/// Falhas de leitura preservam o erro SQLite e distinguem dados inválidos.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum FalhaDeLeitura {
    Sqlite(#[from] rusqlite::Error),
    VisaoInvalida,
    UnidadeInvalida,
}

/// Registra somente fatos reconhecidos pelo leitor da tela, sem HTTP.
///
/// # Errors
/// Retorna o erro SQLite original quando a gravação é recusada.
pub fn registrar_unidade(
    banco: &Connection,
    instalacao: &Instalacao,
    contexto: &str,
    unidade: &UnidadeExibida,
    visao: VisaoDoControle,
    agora: f64,
) -> rusqlite::Result<()> {
    let vista = match visao {
        VisaoDoControle::Resumida => "R",
        VisaoDoControle::Detalhada => "D",
    };
    banco.execute(
        "INSERT INTO controle_unidade
         (origem, contexto, unidade_exibida, visao, observada_em)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (origem, contexto)
         DO UPDATE SET unidade_exibida = excluded.unidade_exibida,
                       visao = excluded.visao,
                       observada_em = excluded.observada_em",
        params![
            instalacao.url_base(),
            contexto,
            unidade.as_str(),
            vista,
            agora
        ],
    )?;
    Ok(())
}

/// Uma leitura sem linha não significa unidade ou lista vazia.
///
/// # Errors
/// Distingue erro SQL, visão não reconhecida e unidade inválida armazenada.
pub fn consultar_unidade(
    banco: &Connection,
    instalacao: &Instalacao,
    contexto: &str,
) -> Result<Option<UnidadeObservada>, FalhaDeLeitura> {
    let linha: Option<(String, String, f64)> = banco
        .query_row(
            "SELECT unidade_exibida, visao, observada_em FROM controle_unidade
             WHERE origem = ?1 AND contexto = ?2",
            params![instalacao.url_base(), contexto],
            |linha| Ok((linha.get(0)?, linha.get(1)?, linha.get(2)?)),
        )
        .optional()?;
    let Some((nome, forma, observada_em)) = linha else {
        return Ok(None);
    };
    let unidade = UnidadeExibida::nova(&nome).ok_or(FalhaDeLeitura::UnidadeInvalida)?;
    let visao = match forma.as_str() {
        "R" => VisaoDoControle::Resumida,
        "D" => VisaoDoControle::Detalhada,
        _ => return Err(FalhaDeLeitura::VisaoInvalida),
    };
    Ok(Some(UnidadeObservada {
        unidade,
        visao,
        observada_em,
    }))
}
