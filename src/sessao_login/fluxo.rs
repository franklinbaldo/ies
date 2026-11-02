//! Orquestração de `ies login`: lock, conferência da sessão gravada (um
//! GET), login pela raiz da instalação e gravação da nova geração.
use super::{ErroLogin, PedidoDeLogin, ResultadoDoLogin, TransporteDeLogin};

/// Fluxo completo com transporte injetável; conta cada pedido feito.
pub(crate) fn entrar_via<T: TransporteDeLogin>(
    pedido: &PedidoDeLogin<'_>,
    transporte: &mut T,
) -> Result<ResultadoDoLogin, ErroLogin> {
    let _ = (pedido, transporte);
    let _ = (
        crate::instalacao::Instalacao::host,
        crate::instalacao::Instalacao::url_base,
    );
    todo!("CONTRACT: fluxo do login")
}
