//! Os quatro passos da resolução e a abertura (tela do processo → árvore).
use super::{
    EnvioDoProcesso, ErroProcessoObter, PedidoDoProcesso, ProcessoAberto, SessaoDoProcesso,
};
use crate::indice_local::IndiceLocal;

pub(crate) fn obter_processo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoProcesso<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoAberto, ErroProcessoObter> {
    let _ = (sessao, pedido, indice, envio);
    Err(ErroProcessoObter::SessaoEncerrada)
}
