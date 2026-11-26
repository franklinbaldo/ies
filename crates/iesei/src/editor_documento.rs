//! Oferta de editor e publicação de revisões em estado SQLite compartilhado (#89).
//! Hipótese sintética: não descreve autorização nem endpoints de um SEI real.
use crate::documento_versoes;
use crate::instalacao::Instalacao;
use rusqlite::{Connection, OptionalExtension, params};
use std::num::NonZeroU32;

/// Oferta efetivamente representada no simulador; assinatura não é veto de edição.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Oferta {
    pub editavel: bool,
    pub assinaturas: u32,
}

/// Só a versão e o conteúdo da base que o usuário conferiu podem ser gravados.
#[derive(Clone, Copy, Debug)]
pub struct Edicao<'a> {
    pub versao_base: NonZeroU32,
    pub html_base: &'a str,
    pub html_editado: &'a str,
    pub confirmar_assinaturas: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Publicacao {
    pub versao_nova: NonZeroU32,
    pub assinaturas_anteriores: u32,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroEditorDocumento {
    Sqlite(#[from] rusqlite::Error),
    NaoObservado,
    NaoOferecido,
    VersaoDivergente,
    ConteudoDivergente,
    AssinaturasNaoConfirmadas,
    DadoInvalido,
}

/// Armazena a oferta explícita de editor, sem inferi-la da assinatura.
///
/// # Errors
/// Recusa ausência de documento e preserva falhas SQLite.
pub fn registrar_oferta(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    oferta: Oferta,
) -> Result<(), ErroEditorDocumento> {
    if cenario.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) || id.is_empty() {
        return Err(ErroEditorDocumento::DadoInvalido);
    }
    if documento_versoes::ler(banco, instalacao, cenario, id)
        .map_err(|e| match e {
            documento_versoes::ErroDocumentoVersoes::Sqlite(err) => {
                ErroEditorDocumento::Sqlite(err)
            }
            documento_versoes::ErroDocumentoVersoes::DadoInvalido => {
                ErroEditorDocumento::DadoInvalido
            }
        })?
        .is_none()
    {
        return Err(ErroEditorDocumento::NaoObservado);
    }
    banco.execute(
        "INSERT INTO documento_editor
         (origem, cenario, id_documento, editavel, assinaturas)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(origem, cenario, id_documento) DO UPDATE SET
         editavel=excluded.editavel, assinaturas=excluded.assinaturas",
        params![
            instalacao.url_base(),
            cenario,
            id,
            oferta.editavel,
            i64::from(oferta.assinaturas)
        ],
    )?;
    Ok(())
}

/// Nenhuma linha de oferta significa desconhecida, nunca edição autorizada.
///
/// # Errors
/// Preserva falha SQL e rejeita contagem persistida inválida.
pub fn consultar_oferta(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Option<Oferta>, ErroEditorDocumento> {
    let linha: Option<(bool, i64)> = banco
        .query_row(
            "SELECT editavel, assinaturas FROM documento_editor
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3",
            params![instalacao.url_base(), cenario, id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    linha
        .map(|(editavel, assinaturas)| {
            Ok(Oferta {
                editavel,
                assinaturas: u32::try_from(assinaturas)
                    .map_err(|_| ErroEditorDocumento::DadoInvalido)?,
            })
        })
        .transpose()
}

/// Síntese tipada do estado documental para a prévia de anexação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SituacaoAssinaturas {
    SemPendencias,
    Pendentes,
    NaoObservadas,
}

/// Documento ativo sem linha de assinaturas impede uma resposta S/N conclusiva.
/// Zero observado é pendência, não ausência de observação.
/// Documentos cancelados não participam da consulta.
///
/// # Errors
/// Preserva falha SQLite e recusa identidade de processo inválida.
pub fn situacao_assinaturas_anexacao(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
) -> Result<SituacaoAssinaturas, ErroEditorDocumento> {
    if cenario.is_empty()
        || id_procedimento.is_empty()
        || !id_procedimento.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(ErroEditorDocumento::DadoInvalido);
    }
    let (desconhecidas, pendentes): (i64, i64) = banco.query_row(
        "SELECT
           COALESCE(MAX(CASE WHEN editor.id_documento IS NULL THEN 1 ELSE 0 END), 0),
           COALESCE(MAX(CASE WHEN editor.assinaturas = 0 THEN 1 ELSE 0 END), 0)
         FROM documento_simulado documento
         LEFT JOIN documento_editor editor
           ON editor.origem=documento.origem
          AND editor.cenario=documento.cenario
          AND editor.id_documento=documento.id_documento
         WHERE documento.origem=?1 AND documento.cenario=?2
           AND documento.id_procedimento=?3
           AND NOT EXISTS(
               SELECT 1 FROM documento_cancelamento cancelamento
               WHERE cancelamento.origem=documento.origem
                 AND cancelamento.cenario=documento.cenario
                 AND cancelamento.id_documento=documento.id_documento
           )",
        params![instalacao.url_base(), cenario, id_procedimento],
        |linha| Ok((linha.get(0)?, linha.get(1)?)),
    )?;
    Ok(if pendentes != 0 {
        SituacaoAssinaturas::Pendentes
    } else if desconhecidas != 0 {
        SituacaoAssinaturas::NaoObservadas
    } else {
        SituacaoAssinaturas::SemPendencias
    })
}

/// Seção opaca, na ordem publicada pelo formulário clássico.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Secao {
    pub nome: String,
    pub principal: bool,
    pub html: String,
}

/// Retorna as seções observadas desta revisão; vazio significa legado.
///
/// # Errors
/// Propaga a falha SQLite.
pub fn ler_secoes(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    versao: NonZeroU32,
) -> rusqlite::Result<Vec<Secao>> {
    let mut stmt = banco.prepare(
        "SELECT nome, principal, html FROM documento_revisao_secao
         WHERE origem=?1 AND cenario=?2 AND id_documento=?3 AND versao=?4
         ORDER BY posicao",
    )?;
    let linhas = stmt.query_map(
        params![instalacao.url_base(), cenario, id, i64::from(versao.get())],
        |r| {
            Ok(Secao {
                nome: r.get(0)?,
                principal: r.get(1)?,
                html: r.get(2)?,
            })
        },
    )?;
    linhas.collect()
}

fn inserir_secoes(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    versao: NonZeroU32,
    secoes: &[Secao],
    html_corpo: &str,
) -> Result<(), ErroEditorDocumento> {
    let mut principais = secoes.iter().filter(|s| s.principal);
    if principais.next().is_none_or(|s| s.html != html_corpo) || principais.next().is_some() {
        return Err(ErroEditorDocumento::DadoInvalido);
    }
    for (posicao, secao) in secoes.iter().enumerate() {
        if !secao
            .nome
            .strip_prefix("txaEditor_")
            .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err(ErroEditorDocumento::DadoInvalido);
        }
        banco.execute(
            "INSERT INTO documento_revisao_secao
             (origem, cenario, id_documento, versao, posicao, nome, principal, html)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                instalacao.url_base(),
                cenario,
                id,
                i64::from(versao.get()),
                i64::try_from(posicao).map_err(|_| ErroEditorDocumento::DadoInvalido)?,
                secao.nome,
                secao.principal,
                secao.html
            ],
        )?;
    }
    Ok(())
}

/// Registra a estrutura inicial observada de uma revisão existente.
///
/// # Errors
/// Recusa identidade ausente ou estrutura incompatível com o corpo.
pub fn registrar_secoes(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    versao: NonZeroU32,
    secoes: &[Secao],
) -> Result<(), ErroEditorDocumento> {
    let tx = banco.unchecked_transaction()?;
    let corpo: String = tx
        .query_row(
            "SELECT html_corpo FROM documento_revisao
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3 AND versao=?4",
            params![instalacao.url_base(), cenario, id, i64::from(versao.get())],
            |r| r.get(0),
        )
        .optional()?
        .ok_or(ErroEditorDocumento::NaoObservado)?;
    inserir_secoes(&tx, instalacao, cenario, id, versao, secoes, &corpo)?;
    tx.commit()?;
    Ok(())
}

/// Uma transação insere N+1 preservando N, compara base com a versão atual
/// e registra perda de assinaturas somente após consentimento explícito.
/// POST repetido não altera revisão previamente publicada.
///
/// # Errors
/// Diferencia ausência de oferta, conflito de versão, conflito de conteúdo
/// e falta de consentimento. Propaga a causa SQLite original.
pub fn publicar(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    edicao: Edicao<'_>,
) -> Result<Publicacao, ErroEditorDocumento> {
    publicar_interno(banco, instalacao, cenario, id, edicao, None)
}

/// Publica o corpo e todas as seções efetivamente transportadas na mesma revisão.
///
/// # Errors
/// Recusa a alteração da identidade das seções ou da versão base.
pub fn publicar_secoes(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    edicao: Edicao<'_>,
    secoes: &[Secao],
) -> Result<Publicacao, ErroEditorDocumento> {
    publicar_interno(banco, instalacao, cenario, id, edicao, Some(secoes))
}

fn publicar_interno(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
    edicao: Edicao<'_>,
    secoes: Option<&[Secao]>,
) -> Result<Publicacao, ErroEditorDocumento> {
    if edicao.html_editado == edicao.html_base {
        return Err(ErroEditorDocumento::DadoInvalido);
    }
    let origem = instalacao.url_base();
    let tx = banco.unchecked_transaction()?;
    let (editavel, assinaturas): (bool, i64) = tx
        .query_row(
            "SELECT editavel, assinaturas FROM documento_editor
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3",
            params![origem, cenario, id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or(ErroEditorDocumento::NaoObservado)?;
    if !editavel {
        return Err(ErroEditorDocumento::NaoOferecido);
    }
    let assinaturas = u32::try_from(assinaturas).map_err(|_| ErroEditorDocumento::DadoInvalido)?;
    let atual: (i64, String, String) = tx
        .query_row(
            "SELECT versao, html_corpo, unidade FROM documento_revisao
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3
             ORDER BY versao DESC LIMIT 1",
            params![origem, cenario, id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or(ErroEditorDocumento::NaoObservado)?;
    if u32::try_from(atual.0).ok() != Some(edicao.versao_base.get()) {
        return Err(ErroEditorDocumento::VersaoDivergente);
    }
    if atual.1 != edicao.html_base {
        return Err(ErroEditorDocumento::ConteudoDivergente);
    }
    if assinaturas > 0 && !edicao.confirmar_assinaturas {
        return Err(ErroEditorDocumento::AssinaturasNaoConfirmadas);
    }
    let anteriores = ler_secoes(&tx, instalacao, cenario, id, edicao.versao_base)?;
    match secoes {
        None if anteriores.is_empty() => {}
        Some(novas)
            if !anteriores.is_empty()
                && anteriores.len() == novas.len()
                && anteriores
                    .iter()
                    .zip(novas)
                    .all(|(a, b)| a.nome == b.nome && a.principal == b.principal) => {}
        _ => return Err(ErroEditorDocumento::DadoInvalido),
    }
    let proxima = edicao
        .versao_base
        .get()
        .checked_add(1)
        .and_then(NonZeroU32::new)
        .ok_or(ErroEditorDocumento::DadoInvalido)?;
    tx.execute(
        "INSERT INTO documento_revisao
         (origem, cenario, id_documento, versao, usuario, unidade, modificacao, html_corpo)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            origem,
            cenario,
            id,
            i64::from(proxima.get()),
            "Editor de ensaio",
            atual.2,
            "08/11/2026",
            edicao.html_editado,
        ],
    )?;
    if let Some(secoes) = secoes {
        inserir_secoes(
            &tx,
            instalacao,
            cenario,
            id,
            proxima,
            secoes,
            edicao.html_editado,
        )?;
    }
    // Uma edição confirmada invalida assinaturas anteriores no simulador.
    tx.execute(
        "UPDATE documento_editor SET assinaturas=0
         WHERE origem=?1 AND cenario=?2 AND id_documento=?3",
        params![origem, cenario, id],
    )?;
    tx.commit()?;
    Ok(Publicacao {
        versao_nova: proxima,
        assinaturas_anteriores: assinaturas,
    })
}

#[cfg(test)]
mod testes_secoes {
    use super::*;
    use crate::banco_cassettes::iniciar;
    use crate::documento_versoes::{DocumentoSimulado, Revisao, guardar, ler};

    #[test]
    fn publica_secoes_e_corpo_atomicamente_preservando_versao_anterior() {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        let v2 = NonZeroU32::new(2).unwrap();
        let v3 = NonZeroU32::new(3).unwrap();
        let documento = DocumentoSimulado {
            instalacao: origem.clone(),
            id_documento: "42".into(),
            id_procedimento: "7".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: "0011519".into(),
            tipo: "Despacho".into(),
            revisoes: vec![Revisao {
                versao: v2,
                usuario: "TESTE".into(),
                unidade: "TESTE".into(),
                modificacao: "26/11/2026".into(),
                html_corpo: "<p>Antes</p>".into(),
            }],
        };
        guardar(&mut banco, "ensaio", &documento).unwrap();
        registrar_oferta(
            &banco,
            &origem,
            "ensaio",
            "42",
            Oferta {
                editavel: true,
                assinaturas: 2,
            },
        )
        .unwrap();
        let secoes = vec![
            Secao {
                nome: "txaEditor_4".into(),
                principal: false,
                html: "<h1>Fixo\r\r\n</h1>".into(),
            },
            Secao {
                nome: "txaEditor_5".into(),
                principal: false,
                html: String::new(),
            },
            Secao {
                nome: "txaEditor_12".into(),
                principal: true,
                html: "<p>Antes</p>".into(),
            },
        ];
        registrar_secoes(&banco, &origem, "ensaio", "42", v2, &secoes).unwrap();
        guardar(&mut banco, "ensaio", &documento).unwrap();
        assert_eq!(
            ler_secoes(&banco, &origem, "ensaio", "42", v2).unwrap(),
            secoes
        );
        let edicao = Edicao {
            versao_base: v2,
            html_base: "<p>Antes</p>",
            html_editado: "<p>Depois</p>",
            confirmar_assinaturas: true,
        };
        let mut novas = secoes.clone();
        novas[0].html = "<h1>Fixo\n</h1>".into();
        novas[2].html = "<p>Depois</p>".into();
        assert!(publicar(&banco, &origem, "ensaio", "42", edicao).is_err());
        assert!(
            publicar_secoes(&banco, &origem, "ensaio", "42", edicao, &novas[1..])
                .is_err()
        );
        banco
            .execute_batch(
                "CREATE TRIGGER falha_secao BEFORE INSERT ON documento_revisao_secao
                 WHEN NEW.versao=3 AND NEW.nome='txaEditor_5'
                 BEGIN SELECT RAISE(ABORT, 'falha sintetica'); END;",
            )
            .unwrap();
        assert!(
            publicar_secoes(&banco, &origem, "ensaio", "42", edicao, &novas).is_err()
        );
        assert_eq!(
            ler(&banco, &origem, "ensaio", "42").unwrap(),
            Some(documento)
        );
        assert!(
            ler_secoes(&banco, &origem, "ensaio", "42", v3)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            consultar_oferta(&banco, &origem, "ensaio", "42")
                .unwrap()
                .unwrap()
                .assinaturas,
            2
        );
        banco.execute_batch("DROP TRIGGER falha_secao").unwrap();
        let publicada =
            publicar_secoes(&banco, &origem, "ensaio", "42", edicao, &novas).unwrap();
        assert_eq!(publicada.versao_nova, v3);
        assert_eq!(
            ler_secoes(&banco, &origem, "ensaio", "42", v2).unwrap(),
            secoes
        );
        assert_eq!(
            ler_secoes(&banco, &origem, "ensaio", "42", v3).unwrap(),
            novas
        );
        assert!(
            publicar_secoes(&banco, &origem, "ensaio", "42", edicao, &novas).is_err()
        );
        // Reimportar uma revisão divergente não reutiliza seções antigas.
        // Exercita também o modo SQLite sem enforcement de FK.
        banco.pragma_update(None, "foreign_keys", false).unwrap();
        let mut reimportado = ler(&banco, &origem, "ensaio", "42")
            .unwrap()
            .unwrap();
        reimportado.revisoes[0].html_corpo = "<p>Substituido</p>".into();
        guardar(&mut banco, "ensaio", &reimportado).unwrap();
        assert!(
            ler_secoes(&banco, &origem, "ensaio", "42", v2)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            ler_secoes(&banco, &origem, "ensaio", "42", v3).unwrap(),
            novas
        );
    }
}
