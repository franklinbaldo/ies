//! `ies unidade colunas --definir todas|LISTA [--confirmar]`: lê o Nível de
//! detalhe pelo link da D, planeja, e só com `--confirmar` salva uma vez e
//! confere por um GET da mesma tela. A tela do Controle fica como foi
//! encontrada. Orçamentos em contracts/cli/unidade-definir-colunas.md.
use super::definir_erro::DesfechoDasColunas;
use super::envio::{EnvioDoControle, TelaAtual};
use crate::instalacao::Instalacao;
use crate::nivel_de_detalhe::AlvoDasColunas;
use std::path::Path;

/// Sem rede antes da tela do Controle já obtida; nenhum envio é repetido.
pub(crate) fn definir_colunas<V: EnvioDoControle>(
    instalacao: &Instalacao,
    tela: &TelaAtual,
    unidade_solicitada: Option<&str>,
    alvo: &AlvoDasColunas,
    confirmar: bool,
    cookie_file: &Path,
    envio: &mut V,
) -> DesfechoDasColunas {
    let _ = (
        instalacao,
        tela,
        unidade_solicitada,
        alvo,
        confirmar,
        cookie_file,
        envio,
    );
    todo!("RED: definir colunas")
}
