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

/// O quinto pedido e a leitura: tipo vazio, o texto (tipos de texto sem
/// `--saida`) e o arquivo.
pub(super) fn ler<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    endereco: &EnderecoDoConteudo,
    numero: &str,
    pedido: PedidoDoAnexo<'_>,
) -> Result<(String, String, Option<AnexoLido>), E> {
    let transporte = |falha| E::Processo(ErroProcessoObter::Transporte(falha));
    rede.requisicoes.registrar();
    let resposta = rede
        .envio
        .get_binario(
            endereco.as_str(),
            &rede.cookie,
            pedido.limite.para_o_transporte(),
        )
        .map_err(transporte)?;
    if resposta.status == 302 {
        let local = resposta
            .valores("location")
            .first()
            .map(|v| (*v).to_owned());
        return Err(E::Processo(fora_de_palpite(local.as_deref())));
    }
    let destino = pedido
        .saida
        .map_or(DestinoDoAnexo::SoMetadados, DestinoDoAnexo::Diretorio);
    let lido = ler_anexo(resposta, numero, pedido.limite, destino).map_err(|erro| match erro {
        ErroAnexo::TelaDeLogin => E::Processo(ErroProcessoObter::SessaoEncerrada),
        ErroAnexo::Transporte(falha) => transporte(falha),
        outro => E::Anexo(outro),
    })?;
    Ok((
        String::new(),
        lido.texto.clone().unwrap_or_default(),
        Some(lido),
    ))
}
