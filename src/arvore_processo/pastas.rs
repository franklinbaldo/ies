//! Pastas da árvore (fatia 3 de #59). Shim do RED: as assinaturas são as
//! finais; o comportamento ainda é a recusa da fatia 1.
use super::{DocumentoLido, ErroArvore, Literal, PastaNaArvore, tela::EnderecoDaArvore};
use crate::instalacao::Instalacao;

/// O pai de um documento: o processo (id decimal) ou `PASTA<k>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Pai {
    Processo(String),
    Pasta(usize),
}

/// Um nó `PASTA` ou `AGUARDE` lido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum NoDePasta {
    Pasta {
        k: usize,
        pai: String,
        rotulo: String,
    },
    Aguarde {
        k: usize,
    },
}

pub(super) fn pai_do_documento(pai: &str) -> Result<Pai, ErroArvore> {
    if super::decimal(pai) {
        return Ok(Pai::Processo(pai.to_owned()));
    }
    Err(ErroArvore::IdDeNoNaoDecimal)
}

pub(super) fn ler_no_de_pasta(_a: &[Literal; 15]) -> Result<NoDePasta, ErroArvore> {
    Err(ErroArvore::TipoDeNoDaArvoreNaoSuportado)
}

pub(super) fn identidade_da_barra(_barra: &str) -> Result<Option<String>, ErroArvore> {
    Ok(None)
}

pub(super) fn conferir_pastas(
    processo: &str,
    _nos: &[NoDePasta],
    _protocolos: &[(usize, String)],
    documentos: &[DocumentoLido],
) -> Result<(Vec<PastaNaArvore>, bool), ErroArvore> {
    if documentos
        .iter()
        .any(|d| d.pai != Pai::Processo(processo.to_owned()))
    {
        return Err(ErroArvore::DocumentoComPaiDiferenteDoProcesso);
    }
    Ok((Vec::new(), false))
}

/// "Abrir todas as Pastas" publicada pela árvore, conferida.
pub(crate) fn ler_acao_de_abrir_pastas(
    _instalacao: &Instalacao,
    _html: &str,
    _id: &str,
) -> Result<Option<EnderecoDaArvore>, ErroArvore> {
    Ok(None)
}
