//! `ies unidade colunas`: GET da tela do Nível de detalhe pelo link que a D
//! publica (`ancNivelDetalhe`). A R não o publica: troca para D e restaura R.
use super::ErroVisaoDoControle as E;
use super::envio::{EnvioDoControle, TelaAtual};
use crate::instalacao::Instalacao;
use crate::nivel_de_detalhe::NivelDeDetalhe;
use std::path::Path;

/// Unidade exibida e as 12 opções com `marcada`.
pub(crate) fn ler_colunas<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: TelaAtual,
    unidade_solicitada: Option<&str>,
    cookie_file: &Path,
    envio: &mut V,
) -> Result<(String, NivelDeDetalhe), E> {
    let _ = (instalacao, tela, unidade_solicitada, cookie_file, envio);
    Err(E::VisaoDoControleNaoReconhecida)
}
