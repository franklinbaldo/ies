//! Propriedades depois da árvore (contracts/cli/processo-obter-propriedades.md).
use super::abertura::{ArvoreAberta, Contexto, Rede};
use super::{EnvioDoProcesso, ErroProcessoObter as E, Propriedades};

pub(super) fn ler<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    aberta: &ArvoreAberta,
    pedidas: bool,
    guardado_primeiro: bool,
) -> Result<Propriedades, E> {
    let _ = (contexto, rede, aberta, pedidas, guardado_primeiro);
    Ok(Propriedades::NaoPublicadasNaBarra)
}
