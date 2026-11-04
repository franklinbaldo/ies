//! Documentos movidos para outro processo (Refs #59;
//! contracts/cli/processo-obter-arvore.md). SHIM: ainda recusa.
use super::ProcessoNaArvore;
use super::{ARGUMENTOS, ArvoreLida, DocumentoLido, DocumentoMovido, ErroArvore, Literal};
use crate::instalacao::Instalacao;

pub(super) struct MovidoLido {
    pai: String,
    movido: DocumentoMovido,
}

pub(super) fn ler_movido(_a: &[Literal; ARGUMENTOS]) -> Result<MovidoLido, ErroArvore> {
    Err(ErroArvore::TipoDeNoDaArvoreNaoSuportado)
}

pub(super) fn conferir_movidos(
    _processo: &ProcessoNaArvore,
    lidos: Vec<MovidoLido>,
    _documentos: &[DocumentoLido],
) -> Result<Vec<DocumentoMovido>, ErroArvore> {
    Ok(lidos.into_iter().map(|l| (l.pai, l.movido).1).collect())
}

pub(crate) fn ler_destinos_dos_movidos(
    _instalacao: &Instalacao,
    _html: &str,
    _arvore: &mut ArvoreLida,
) -> Result<(), ErroArvore> {
    Ok(())
}
