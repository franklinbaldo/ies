//! `ies unidade listar-processos [--pagina N]` na visão D, deixando a tela
//! como foi encontrada. Orçamentos no contrato da visão detalhada.
use super::ErroVisaoDoControle as E;
use super::envio::{EnvioDoControle, TelaAtual};
use crate::caixa_detalhada::LeituraDetalhada;
use crate::instalacao::Instalacao;
use std::path::Path;

/// A lista D (página observada ou a página N), com a categoria das linhas
/// que as tabelas R do mesmo GET mostraram.
pub(crate) fn listar_na_visao_detalhada<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: TelaAtual,
    unidade_solicitada: Option<&str>,
    pagina: Option<usize>,
    cookie_file: &Path,
    envio: &mut V,
) -> Result<LeituraDetalhada, E> {
    let _ = (instalacao, tela, unidade_solicitada, pagina, cookie_file, envio);
    Err(E::VisaoDoControleNaoReconhecida)
}
