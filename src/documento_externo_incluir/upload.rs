//! #332 — transmissão multipart HTTPS de um PDF em fluxo, POST único.
//! A operação recebe a URL exclusivamente do cadastro observado; a CLI
//! ainda mantém --confirmar desabilitado até ensaio em TREINAMENTO.
use super::{
    multipart::MultipartPdf,
    recibo::{FalhaUpload, ReciboUpload, interpretar_http},
};
use crate::transporte::cliente_https::{self, EnvioEmFluxo};
use crate::transporte::{FalhaTransporte, RespostaWeb};
use std::io::Read;

/// Os cabeçalhos e o leitor pertencem à mesma tentativa de upload.
/// A URL e o cookie são privados, portanto este pedido não implementa Debug.
pub(crate) struct PedidoUpload<'a> {
    pub(crate) url: &'a str,
    pub(crate) cookie: &'a str,
    pub(crate) origem: &'a str,
    pub(crate) referer: &'a str,
    pub(crate) tipo_conteudo: &'a str,
    pub(crate) tamanho: u64,
    pub(crate) corpo: &'a mut dyn Read,
}

/// Compartilha o mesmo corpo multipart tanto com ensaios offline quanto
/// com o cliente HTTPS real; resultado incerto nunca reenvia o arquivo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn enviar_via<R: Read>(
    plano: &MultipartPdf,
    arquivo: impl Read,
    url: &str,
    cookie: &str,
    origem: &str,
    referer: &str,
    requisitar: impl FnOnce(PedidoUpload<'_>) -> Result<RespostaWeb<R>, FalhaTransporte>,
    nome: &str,
    tamanho_bytes: u64,
) -> Result<ReciboUpload, FalhaUpload> {
    let mut corpo = plano.corpo(arquivo);
    let resposta = requisitar(PedidoUpload {
        url,
        cookie,
        origem,
        referer,
        tipo_conteudo: plano.tipo_conteudo(),
        tamanho: plano.tamanho_total(),
        corpo: &mut corpo,
    })
    .map_err(FalhaUpload::EnvioSemResposta)?;
    interpretar_http(resposta, nome, tamanho_bytes)
}

/// Um único POST real, usando o mesmo cliente HTTPS, proxy e TLS do IES.
/// O chamador prepara o arquivo e o plano antes de autorizar esta escrita.
#[allow(clippy::too_many_arguments)]
pub(crate) fn enviar_https(
    plano: &MultipartPdf,
    arquivo: impl Read,
    url: &str,
    cookie: &str,
    origem: &str,
    referer: &str,
    nome: &str,
    tamanho_bytes: u64,
) -> Result<ReciboUpload, FalhaUpload> {
    enviar_via(
        plano,
        arquivo,
        url,
        cookie,
        origem,
        referer,
        |pedido| {
            let envio = EnvioEmFluxo {
                tipo: pedido.tipo_conteudo,
                origem: pedido.origem,
                referer: pedido.referer,
                tamanho: pedido.tamanho,
                corpo: pedido.corpo,
            };
            cliente_https::requisitar_em_fluxo(pedido.url, pedido.cookie, envio)
                .map(cliente_https::resposta_web)
        },
        nome,
        tamanho_bytes,
    )
}
