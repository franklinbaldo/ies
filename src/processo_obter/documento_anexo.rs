//! Passo 5 do documento externo (Refs #60, fatia 3;
//! contracts/cli/documento-obter-externo.md): GET binário do link
//! `documento_download_anexo` que a árvore publicou para o nó, contado
//! como as demais requisições, e a leitura pelo `anexo_documento`. 302
//! para o login é sessão encerrada, como no resto do fluxo.
use super::super::abertura::{Rede, fora_de_palpite};
use super::super::{EnvioDoProcesso, ErroProcessoObter};
use super::erro::ErroDocumentoObter as E;
use crate::anexo_documento::{AnexoLido, DestinoDoAnexo, ErroAnexo, LimiteDoAnexo, ler_anexo};
use crate::arvore_processo::EnderecoDoConteudo;
use std::path::Path;

/// O limite do anexo e o `--saida` (o parcial é gravado durante a leitura).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PedidoDoAnexo<'a> {
    pub(crate) limite: LimiteDoAnexo,
    pub(crate) saida: Option<&'a Path>,
}

pub(super) fn ler_anexo_do_link<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    endereco: &EnderecoDoConteudo,
    numero: &str,
    pedido: PedidoDoAnexo<'_>,
) -> Result<AnexoLido, E> {
    let _ = (rede, endereco, numero, pedido);
    let _ = (fora_de_palpite, ler_anexo::<std::io::Empty>);
    let _: Option<(DestinoDoAnexo, ErroAnexo, ErroProcessoObter)> = None;
    todo!()
}
