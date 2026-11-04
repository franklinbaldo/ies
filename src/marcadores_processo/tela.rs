//! A tela "Marcadores do Processo" (`andamento_marcador_gerenciar`), lida
//! sem envio. Shim do RED: ainda não lê.
use super::{ErroMarcadores as E, TelaDosMarcadores};

pub(crate) fn ler_marcadores(_html: &str, _nup: &str, _id: &str) -> Result<TelaDosMarcadores, E> {
    Err(E::HtmlDosMarcadoresNaoInterpretado)
}
