//! Passo 5 do documento externo (Refs #60, fatia 3;
//! contracts/cli/documento-obter-externo.md): GET binário do link
//! `documento_download_anexo` que a árvore publicou para o nó, contado
//! como as demais requisições, e a leitura pelo `anexo_documento`. 302
//! para o login é sessão encerrada, como no resto do fluxo.
use super::super::abertura::{Rede, fora_de_palpite};
use super::super::{EnvioDoProcesso, ErroProcessoObter};
use crate::anexo_documento::{AnexoLido, DestinoDoAnexo, ErroAnexo, LimiteDoAnexo, ler_anexo};
use crate::arvore_processo::EnderecoDoConteudo;
use std::path::Path;

/// O limite do anexo e o `--saida` (o parcial é gravado durante a leitura).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PedidoDoAnexo<'a> {
    pub(crate) limite: LimiteDoAnexo,
    pub(crate) saida: Option<&'a Path>,
    pub(crate) cache_host: Option<&'a str>,
}

/// O quinto pedido e a leitura: tipo vazio, o texto (tipos de texto sem
/// `--saida`) e o arquivo.
pub(super) fn ler<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    endereco: &EnderecoDoConteudo,
    numero: &str,
    pedido: PedidoDoAnexo<'_>,
) -> Result<(String, String, Option<AnexoLido>), super::erro::ErroDocumentoObter> {
    let transporte =
        |falha| super::erro::ErroDocumentoObter::Processo(ErroProcessoObter::Transporte(falha));
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
        return Err(super::erro::ErroDocumentoObter::Processo(fora_de_palpite(
            local.as_deref(),
        )));
    }
    let raiz = if pedido.cache_host.is_some() {
        Some(
            crate::cache_documento::raiz()
                .map_err(|_| super::erro::ErroDocumentoObter::Anexo(ErroAnexo::EscritaFalhou))?,
        )
    } else {
        None
    };
    let diretorio = raiz
        .as_ref()
        .zip(pedido.cache_host)
        .map(|(raiz, host)| raiz.join("ies").join(host).join(numero));
    let destino = match (diretorio.as_deref(), pedido.cache_host, pedido.saida) {
        (Some(diretorio), Some(host), _) => DestinoDoAnexo::Cache { diretorio, host },
        (None, _, Some(dir)) => DestinoDoAnexo::Diretorio(dir),
        _ => DestinoDoAnexo::SoMetadados,
    };
    let lido = ler_anexo(resposta, numero, pedido.limite, destino).map_err(|erro| match erro {
        ErroAnexo::TelaDeLogin(crate::erro_publico::SessaoEncerrada) => {
            super::erro::ErroDocumentoObter::Processo(ErroProcessoObter::SessaoEncerrada(
                crate::erro_publico::SessaoEncerrada,
            ))
        }
        ErroAnexo::Transporte(falha) => transporte(falha),
        outro @ (ErroAnexo::SemTipoDeConteudo
        | ErroAnexo::TipoNaoPermitido
        | ErroAnexo::CabecalhoRepetido
        | ErroAnexo::DisposicaoForaDaFormaObservada(_)
        | ErroAnexo::AcimaDoLimite
        | ErroAnexo::Vazio
        | ErroAnexo::ConteudoComLinkAssinado
        | ErroAnexo::RecusadoPeloSei(_)
        | ErroAnexo::Corpo(_)
        | ErroAnexo::DiretorioAusente
        | ErroAnexo::ArquivoJaExiste
        | ErroAnexo::EscritaFalhou
        | ErroAnexo::GravacaoDoParcialFalhou(_)
        | ErroAnexo::PublicacaoDoCacheFalhou(_)
        | ErroAnexo::CorpoNaoUtf8NoCharsetDeclarado(_)) => {
            super::erro::ErroDocumentoObter::Anexo(outro)
        }
    })?;
    Ok((
        String::new(),
        lido.texto.clone().unwrap_or_default(),
        Some(lido),
    ))
}
