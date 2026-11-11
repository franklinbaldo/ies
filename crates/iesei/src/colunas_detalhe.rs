//! Estado tipado das colunas disponíveis no Nível de detalhe, derivado de
//! observações ou hipóteses explícitas e reconstruído em SQLite efêmero.
//! Uma opção ausente é indisponível; ausência de configuração não é vazio.
use crate::instalacao::Instalacao;
use crate::observacao::UnidadeExibida;
use rusqlite::{Connection, params};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Coluna {
    Atribuicao,
    Anotacao,
    TipoProcesso,
    Especificacao,
    Prioridade,
    Interessados,
    Observacao,
    ControlePrazo,
    RetornoDevolver,
    RetornoAguardando,
    UltimaMovimentacao,
    Marcadores,
}

impl Coluna {
    pub const TODAS: [Self; 12] = [
        Self::Atribuicao,
        Self::Anotacao,
        Self::TipoProcesso,
        Self::Especificacao,
        Self::Prioridade,
        Self::Interessados,
        Self::Observacao,
        Self::ControlePrazo,
        Self::RetornoDevolver,
        Self::RetornoAguardando,
        Self::UltimaMovimentacao,
        Self::Marcadores,
    ];

    #[must_use]
    pub const fn chave(self) -> &'static str {
        match self {
            Self::Atribuicao => "atribuicao",
            Self::Anotacao => "anotacao",
            Self::TipoProcesso => "tipo",
            Self::Especificacao => "especificacao",
            Self::Prioridade => "prioridade",
            Self::Interessados => "interessados",
            Self::Observacao => "observacao",
            Self::ControlePrazo => "controle_de_prazo",
            Self::RetornoDevolver => "para_devolver",
            Self::RetornoAguardando => "aguardando_retorno",
            Self::UltimaMovimentacao => "ultima_movimentacao",
            Self::Marcadores => "marcadores",
        }
    }

    #[must_use]
    pub const fn campo(self) -> &'static str {
        match self {
            Self::Atribuicao => "chkSinAtribuicao",
            Self::Anotacao => "chkSinAnotacao",
            Self::TipoProcesso => "chkSinTipoProcesso",
            Self::Especificacao => "chkSinEspecificacao",
            Self::Prioridade => "chkSinNivelPrioritarios",
            Self::Interessados => "chkSinInteressados",
            Self::Observacao => "chkSinObservacao",
            Self::ControlePrazo => "chkSinControlePrazo",
            Self::RetornoDevolver => "chkSinRetornoDevolver",
            Self::RetornoAguardando => "chkSinRetornoAguardando",
            Self::UltimaMovimentacao => "chkSinUltimaMovimentacao",
            Self::Marcadores => "chkSinMarcadores",
        }
    }

    #[must_use]
    pub const fn rotulo(self) -> &'static str {
        match self {
            Self::Atribuicao => "Atribuição",
            Self::Anotacao => "Anotação",
            Self::TipoProcesso => "Tipo do Processo",
            Self::Especificacao => "Especificação",
            Self::Prioridade => "Prioridade",
            Self::Interessados => "Interessados",
            Self::Observacao => "Observação",
            Self::ControlePrazo => "Controle de Prazo",
            Self::RetornoDevolver => "Para Devolver",
            Self::RetornoAguardando => "Aguardando Retorno",
            Self::UltimaMovimentacao => "Última Movimentação na Unidade",
            Self::Marcadores => "Marcadores",
        }
    }

    #[must_use]
    pub fn pela_chave(chave: &str) -> Option<Self> {
        Self::TODAS
            .into_iter()
            .find(|coluna| coluna.chave() == chave)
    }
}

/// Uma opção oferecida; checkbox ausente não é opção desmarcada.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Opcao {
    pub coluna: Coluna,
    pub marcada: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Configuracao {
    pub unidade: UnidadeExibida,
    pub opcoes: Vec<Opcao>,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum Erro {
    Sqlite(#[from] rusqlite::Error),
    DadoInvalido,
}

/// Substitui a configuração de um cenário sem conservar lixo de ensaios antigos.
/// # Errors
/// Recusa opções vazias, repetidas, cenário vazio e falhas SQL.
pub fn guardar(
    banco: &mut Connection,
    instalacao: &Instalacao,
    cenario: &str,
    configuracao: &Configuracao,
) -> Result<(), Erro> {
    if cenario.is_empty()
        || configuracao.opcoes.is_empty()
        || configuracao.opcoes.len() > Coluna::TODAS.len()
    {
        return Err(Erro::DadoInvalido);
    }
    let mut vistas = std::collections::HashSet::new();
    if !configuracao
        .opcoes
        .iter()
        .all(|opcao| vistas.insert(opcao.coluna))
    {
        return Err(Erro::DadoInvalido);
    }
    let tx = banco.transaction()?;
    let origem = instalacao.url_base();
    tx.execute(
        "DELETE FROM colunas_detalhe WHERE origem=?1 AND cenario=?2",
        params![origem, cenario],
    )?;
    for (posicao, opcao) in configuracao.opcoes.iter().enumerate() {
        let posicao = i64::try_from(posicao).map_err(|_| Erro::DadoInvalido)?;
        tx.execute(
            "INSERT INTO colunas_detalhe
             (origem, cenario, unidade, posicao, coluna, marcada)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                origem,
                cenario,
                configuracao.unidade.as_str(),
                posicao,
                opcao.coluna.chave(),
                opcao.marcada
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Aplica o conjunto exato de opções marcadas ao estado observado.
/// Não cria um estado vazio a partir de ausência de observação.
/// # Errors
/// Recusa opções não oferecidas, repetidas e falhas SQL.
pub fn definir(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    marcadas: &[Coluna],
) -> Result<Option<Configuracao>, Erro> {
    let Some(mut configuracao) = ler(banco, instalacao, cenario)? else {
        return Ok(None);
    };
    let mut vistas = std::collections::HashSet::new();
    if !marcadas.iter().all(|coluna| {
        vistas.insert(*coluna)
            && configuracao.opcoes.iter().any(|opcao| opcao.coluna == *coluna)
    }) {
        return Err(Erro::DadoInvalido);
    }
    let tx = banco.unchecked_transaction()?;
    for opcao in &mut configuracao.opcoes {
        opcao.marcada = marcadas.contains(&opcao.coluna);
        tx.execute(
            "UPDATE colunas_detalhe SET marcada=?1
             WHERE origem=?2 AND cenario=?3 AND coluna=?4",
            params![
                opcao.marcada,
                instalacao.url_base(),
                cenario,
                opcao.coluna.chave()
            ],
        )?;
    }
    tx.commit()?;
    Ok(Some(configuracao))
}

/// Recupera apenas a configuração que o cenário efetivamente contém.
/// # Errors
/// Recusa colunas desconhecidas, unidades inconsistentes e falhas SQL.
pub fn ler(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<Option<Configuracao>, Erro> {
    let mut consulta = banco.prepare(
        "SELECT unidade, coluna, marcada FROM colunas_detalhe
         WHERE origem=?1 AND cenario=?2 ORDER BY posicao",
    )?;
    let linhas = consulta.query_map(params![instalacao.url_base(), cenario], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
        ))
    })?;
    let mut unidade: Option<UnidadeExibida> = None;
    let mut opcoes = Vec::new();
    for linha in linhas {
        let (nome, chave, marcada) = linha?;
        let nome = UnidadeExibida::nova(&nome).ok_or(Erro::DadoInvalido)?;
        if unidade.as_ref().is_some_and(|anterior| *anterior != nome) {
            return Err(Erro::DadoInvalido);
        }
        unidade = Some(nome);
        let coluna = Coluna::pela_chave(&chave).ok_or(Erro::DadoInvalido)?;
        if !matches!(marcada, 0 | 1) || opcoes.iter().any(|o: &Opcao| o.coluna == coluna) {
            return Err(Erro::DadoInvalido);
        }
        opcoes.push(Opcao {
            coluna,
            marcada: marcada == 1,
        });
    }
    Ok(unidade.map(|unidade| Configuracao { unidade, opcoes }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banco_cassettes;

    #[test]
    fn sqlite_recria_estado_tipado_sem_concluir_vazio_na_ausencia() {
        let mut db = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&db).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        assert_eq!(ler(&db, &origem, "cenario").unwrap(), None);
        let configuracao = Configuracao {
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            opcoes: vec![
                Opcao {
                    coluna: Coluna::Atribuicao,
                    marcada: true,
                },
                Opcao {
                    coluna: Coluna::Anotacao,
                    marcada: false,
                },
            ],
        };
        guardar(&mut db, &origem, "cenario", &configuracao).unwrap();
        guardar(&mut db, &origem, "cenario", &configuracao).unwrap();
        assert_eq!(
            ler(&db, &origem, "cenario").unwrap(),
            Some(configuracao.clone())
        );
        assert_eq!(ler(&db, &origem, "outro").unwrap(), None);
        let repetida = Configuracao {
            opcoes: vec![
                Opcao {
                    coluna: Coluna::Atribuicao,
                    marcada: true,
                },
                Opcao {
                    coluna: Coluna::Atribuicao,
                    marcada: false,
                },
            ],
            ..configuracao
        };
        assert!(matches!(
            guardar(&mut db, &origem, "cenario", &repetida),
            Err(Erro::DadoInvalido)
        ));
        assert_eq!(
            ler(&db, &origem, "cenario").unwrap().unwrap().opcoes.len(),
            2
        );
    }
}
