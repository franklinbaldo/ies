//! A sequência de `ies processo concluir` (contracts/cli/processo-concluir.md):
//! abertura, decisão antes de escrever, uma escrita e a leitura de
//! verificação, com a tabela de reconciliação do contrato.
use super::erro::ErroConcluir as C;
use super::{PedidoDaConclusao, ProcessoConcluido};
use crate::indice_local::IndiceLocal;
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso};

/// Sem rede além da abertura na prévia; uma escrita, nunca repetida.
pub(crate) fn concluir_processo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaConclusao<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoConcluido, C> {
    let _ = (sessao, pedido, indice, envio);
    Err(C::UnidadeDaSessaoNaoPublicada)
}
