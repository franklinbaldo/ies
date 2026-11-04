//! Marcadores depois das propriedades (contracts/cli/processo-obter-marcadores.md).
//! Shim do RED: ainda não lê.
use super::abertura::{ArvoreAberta, Contexto, Rede};
use super::{EnvioDoProcesso, ErroProcessoObter as E, Marcadores};

pub(super) fn ler<V: EnvioDoProcesso>(
    _contexto: &Contexto<'_>,
    _rede: &mut Rede<'_, V>,
    _aberta: &ArvoreAberta,
    _pedidos: bool,
    _guardado_primeiro: bool,
) -> Result<Marcadores, E> {
    Ok(Marcadores::Omitidos)
}
