//! Leitura informativa de metadados do processo com degradação segura para
//! leitura parcial quando pastas estão incompletas (Refs #342).
use super::{
    EnvioDoProcesso, ErroProcessoObter, PedidoDoProcesso, ProcessoAberto, SessaoDoProcesso,
    obter_processo,
};
use crate::indice_local::IndiceLocal;

/// Abre o processo para consulta informativa de metadados, habilitando
/// a leitura parcial segura se pastas estiverem incompletas.
pub(crate) fn obter_processo_informativo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoProcesso<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoAberto, ErroProcessoObter> {
    let mut pedido_informativo = *pedido;
    pedido_informativo.leitura_das_pastas = super::LeituraDasPastas::Informativa;
    obter_processo(sessao, &pedido_informativo, indice, envio)
}
