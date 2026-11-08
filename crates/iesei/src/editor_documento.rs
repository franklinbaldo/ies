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

#[derive(Debug)]
pub enum Falha {
    Sqlite(rusqlite::Error),
    NaoObservado,
    NaoOferecido,
    VersaoDivergente,
    ConteudoDivergente,
    AssinaturasNaoConfirmadas,
    DadoInvalido,
}

impl From<rusqlite::Error> for Falha {
    fn from(erro: rusqlite::Error) -> Self {
        Self::Sqlite(erro)
    }
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
) -> Result<(), Falha> {
    if cenario.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) || id.is_empty() {
        return Err(Falha::DadoInvalido);
    }
    if documento_versoes::ler(banco, instalacao, cenario, id)
        .map_err(|e| match e {
            documento_versoes::Falha::Sqlite(err) => Falha::Sqlite(err),
            documento_versoes::Falha::DadoInvalido => Falha::DadoInvalido,
        })?
        .is_none()
    {
        return Err(Falha::NaoObservado);
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
) -> Result<Option<Oferta>, Falha> {
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
                assinaturas: u32::try_from(assinaturas).map_err(|_| Falha::DadoInvalido)?,
            })
        })
        .transpose()
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
) -> Result<Publicacao, Falha> {
    if edicao.html_editado.trim().is_empty()
        || edicao.html_editado == edicao.html_base
        || edicao.html_editado.contains("infra_hash")
    {
        return Err(Falha::DadoInvalido);
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
        .ok_or(Falha::NaoObservado)?;
    if !editavel {
        return Err(Falha::NaoOferecido);
    }
    let assinaturas = u32::try_from(assinaturas).map_err(|_| Falha::DadoInvalido)?;
    let atual: (i64, String, String) = tx
        .query_row(
            "SELECT versao, html_corpo, unidade FROM documento_revisao
             WHERE origem=?1 AND cenario=?2 AND id_documento=?3
             ORDER BY versao DESC LIMIT 1",
            params![origem, cenario, id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or(Falha::NaoObservado)?;
    if u32::try_from(atual.0).ok() != Some(edicao.versao_base.get()) {
        return Err(Falha::VersaoDivergente);
    }
    if atual.1 != edicao.html_base {
        return Err(Falha::ConteudoDivergente);
    }
    if assinaturas > 0 && !edicao.confirmar_assinaturas {
        return Err(Falha::AssinaturasNaoConfirmadas);
    }
    let proxima = edicao
        .versao_base
        .get()
        .checked_add(1)
        .and_then(NonZeroU32::new)
        .ok_or(Falha::DadoInvalido)?;
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
