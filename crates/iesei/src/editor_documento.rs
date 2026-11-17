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
