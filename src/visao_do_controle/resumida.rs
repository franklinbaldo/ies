//! `ies unidade recebidos|gerados [--pagina N]` quando a sessão está na D:
//! troca para R, lê, e restaura a D.
use super::ErroVisaoDoControle as E;
use super::envio::{EnvioDoControle, TelaAtual};
use crate::dominio::Categoria;
use crate::instalacao::Instalacao;
use std::path::Path;

/// HTML da visão R com a página pedida da categoria (ou a lembrada).
pub(crate) fn obter_na_visao_resumida<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: TelaAtual,
    unidade_solicitada: Option<&str>,
    categoria: Categoria,
    pagina: Option<usize>,
    cookie_file: &Path,
    envio: &mut V,
) -> Result<String, E> {
    let _ = (
        instalacao,
        tela,
        unidade_solicitada,
        categoria,
        pagina,
        cookie_file,
        envio,
    );
    Err(E::VisaoDoControleNaoReconhecida)
}
