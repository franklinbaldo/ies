//! Resposta HTTP do cliente HTTPS único: status, **todos** os cabeçalhos
//! (inclusive cada `Set-Cookie`) e o corpo como leitor em fluxo, já
//! desenquadrado pelo `ureq`. Quem consome decide o limite: 8 MB no HTML,
//! 64 KiB na recusa; 3xx nunca tem o corpo lido.
use super::FalhaTransporte;
use std::io::{self, Read};

/// Status, cabeçalhos na ordem recebida e o corpo ainda não lido.
pub(crate) struct Resposta<R> {
    pub(crate) status: u16,
    pub(crate) cabecalhos: Vec<(String, String)>,
    pub(crate) corpo: Corpo<R>,
}

impl<R> Resposta<R> {
    pub(crate) fn nova(status: u16, cabecalhos: Vec<(String, String)>, corpo: Corpo<R>) -> Self {
        let declarado = cabecalhos
            .iter()
            .find(|(chave, _)| chave.eq_ignore_ascii_case("content-length"))
            .and_then(|(_, valor)| valor.trim().parse().ok());
        Self {
            status,
            cabecalhos,
            corpo: Corpo { declarado, ..corpo },
        }
    }

    /// Primeiro valor do cabeçalho, nome sem distinção de caixa.
    pub(crate) fn cabecalho(&self, nome: &str) -> Option<&str> {
        self.cabecalhos
            .iter()
            .find(|(chave, _)| chave.eq_ignore_ascii_case(nome))
            .map(|(_, valor)| valor.as_str())
    }
}

/// Corpo em fluxo, com limite de bytes lidos.
pub(crate) struct Corpo<R> {
    leitor: R,
    declarado: Option<usize>,
    lido: usize,
    /// Total além do qual a leitura falha: 8 MB no HTML; o GET binário
    /// (`resposta_binaria`) define o seu.
    limite: usize,
    /// 3xx: nada é lido.
    vazio: bool,
}

impl<R> Corpo<R> {
    pub(crate) fn novo(leitor: R, vazio: bool) -> Self {
        Self {
            leitor,
            declarado: None,
            lido: 0,
            limite: crate::MAX_HTML_BYTES,
            vazio,
        }
    }

    /// `Content-Length` declarado, antes de qualquer leitura.
    pub(crate) fn tamanho_declarado(&self) -> Option<usize> {
        self.declarado.filter(|_| !self.vazio)
    }

    /// Troca o limite de bytes lidos (o do GET binário).
    pub(crate) fn limitar_em_pedacos(&mut self, limite: usize) {
        self.limite = limite;
    }
}

impl<R: Read> Read for Corpo<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.vazio || buf.is_empty() {
            return Ok(0);
        }
        let lido = self.leitor.read(buf)?;
        self.lido += lido;
        if self.lido > self.limite {
            return Err(io::Error::other(CorpoExcessivo));
        }
        Ok(lido)
    }
}

/// Valor de cabeçalho em texto: UTF-8 quando válido, senão ISO-8859-1
/// (nome de arquivo do SEI em `Content-Disposition`).
pub(crate) fn texto_do_cabecalho(bytes: &[u8]) -> String {
    std::str::from_utf8(bytes).map_or_else(
        |_| bytes.iter().copied().map(char::from).collect(),
        str::to_owned,
    )
}

/// Marca o erro de leitura que levaria o corpo além do limite.
#[derive(Debug)]
struct CorpoExcessivo;

impl std::fmt::Display for CorpoExcessivo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("corpo acima do limite")
    }
}

impl std::error::Error for CorpoExcessivo {}

/// Erro de leitura do corpo como falha: o limite é `RespostaExcessiva`.
pub(crate) fn falha_de_leitura(erro: &io::Error) -> FalhaTransporte {
    let interno = erro
        .get_ref()
        .and_then(|e| e.downcast_ref::<CorpoExcessivo>());
    if interno.is_some() {
        FalhaTransporte::RespostaExcessiva
    } else {
        FalhaTransporte::Rede
    }
}

/// Resposta sintética para testes, sem rede.
#[cfg(test)]
pub(crate) fn sintetica(
    status: u16,
    cabecalhos: &[(&str, &str)],
    corpo: &[u8],
) -> Resposta<io::Cursor<Vec<u8>>> {
    Resposta::nova(
        status,
        cabecalhos
            .iter()
            .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
            .collect(),
        Corpo::novo(
            io::Cursor::new(corpo.to_vec()),
            (300..400).contains(&status),
        ),
    )
}
