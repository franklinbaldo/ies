//! GET binário do cliente HTTPS único (Refs #60, fatia 3;
//! contracts/cli/documento-obter-externo.md): o mesmo pedido, uma
//! conexão, sem redirect e sem retry, mas com o limite do total em
//! `chunked` dado por quem lê o anexo, e só os cabeçalhos que a leitura do
//! anexo confere (nunca `Set-Cookie`). O corpo continua em fluxo: quem
//! consome limita, faz o hash e grava em blocos.
use super::FalhaTransporte;
use super::cliente_https::{self, CorpoHttps};
use super::resposta_http::{Corpo, Resposta};
use std::io::BufRead;

/// Cabeçalhos guardados, em minúsculas; os demais são descartados.
const GUARDADOS: [&str; 4] = [
    "content-type",
    "content-length",
    "content-disposition",
    "location",
];

/// Status, os cabeçalhos guardados (todas as ocorrências, na ordem) e o
/// corpo ainda não lido.
pub(crate) struct RespostaBinaria<R> {
    pub(crate) status: u16,
    pub(crate) cabecalhos: Vec<(String, String)>,
    pub(crate) corpo: R,
}

impl<R> RespostaBinaria<R> {
    /// Todas as ocorrências do cabeçalho, nome sem distinção de caixa.
    pub(crate) fn valores(&self, nome: &str) -> Vec<&str> {
        self.cabecalhos
            .iter()
            .filter(|(chave, _)| chave.eq_ignore_ascii_case(nome))
            .map(|(_, valor)| valor.as_str())
            .collect()
    }
}

/// A resposta lida, com o limite de `chunked` trocado e os cabeçalhos
/// filtrados.
pub(crate) fn binaria<R: BufRead>(
    resposta: Resposta<R>,
    limite: usize,
) -> RespostaBinaria<Corpo<R>> {
    let Resposta {
        status,
        cabecalhos,
        mut corpo,
    } = resposta;
    corpo.limitar_em_pedacos(limite);
    let cabecalhos = cabecalhos
        .into_iter()
        .filter(|(nome, _)| GUARDADOS.iter().any(|g| nome.eq_ignore_ascii_case(g)))
        .map(|(nome, valor)| (nome.to_ascii_lowercase(), valor))
        .collect();
    RespostaBinaria {
        status,
        cabecalhos,
        corpo,
    }
}

/// GET real único do link do anexo (proxy do ambiente, TLS nativo).
pub(crate) fn get_binario(
    url: &str,
    cookie: &str,
    limite: usize,
) -> Result<RespostaBinaria<CorpoHttps>, FalhaTransporte> {
    cliente_https::requisitar("GET", url, cookie, None).map(|r| binaria(r, limite))
}

#[cfg(test)]
#[path = "tests_resposta_binaria.rs"]
mod tests_resposta_binaria;
