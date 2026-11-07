//! Cadastro externo recarregado por tipo — observação estrutural (#332).
//! Não envia upload nem cadastra documentos.

use super::anexo::{FalhaAnexo, PlanoAnexo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FalhaCadastroExterno {
    FormaNaoObservada,
    TipoNaoOferecido,
    TipoAmbiguo,
    TipoNaoRecarregado,
    PdfNaoOferecido,
    PdfAcimaDoLimite,
    PlanoAnexo(FalhaAnexo),
}

pub(crate) struct CadastroExterno {
    pub(crate) id_tipo: String,
    pub(crate) limite_pdf_bytes: u64,
    pub(crate) plano: PlanoAnexo,
}

/// Inspeciona o cadastro servido *depois* da escolha do tipo (sem escrita).
pub(crate) fn observar_cadastro(
    _html: &str,
    _tipo: &str,
    _tamanho_pdf: u64,
) -> Result<CadastroExterno, FalhaCadastroExterno> {
    Err(FalhaCadastroExterno::FormaNaoObservada)
}

#[cfg(test)]
#[path = "tests_cadastro.rs"]
mod tests;
