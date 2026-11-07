//! Leitura da resposta HTTP/1.x do cliente HTTPS único: status, **todos**
//! os cabeçalhos (inclusive cada `Set-Cookie`) e o corpo como leitor em
//! fluxo (`Content-Length`, `chunked` ou até o fim). Quem consome decide o
//! limite: 8 MB no HTML, 64 KiB na recusa; 3xx nunca tem o corpo lido.
use super::FalhaTransporte;
use std::io::{self, BufRead, Read};

const MAX_LINHA: usize = 16 * 1024;
const MAX_CABECALHOS: usize = 200;

/// Status, cabeçalhos na ordem recebida e o corpo ainda não lido.
pub(crate) struct Resposta<R> {
    pub(crate) status: u16,
    pub(crate) cabecalhos: Vec<(String, String)>,
    pub(crate) corpo: Corpo<R>,
}

impl<R> Resposta<R> {
    /// Primeiro valor do cabeçalho, nome sem distinção de caixa.
    pub(crate) fn cabecalho(&self, nome: &str) -> Option<&str> {
        self.cabecalhos
            .iter()
            .find(|(chave, _)| chave.eq_ignore_ascii_case(nome))
            .map(|(_, valor)| valor.as_str())
    }
}

/// Corpo em fluxo, no enquadramento que a resposta declarou.
pub(crate) struct Corpo<R> {
    leitor: R,
    modo: Modo,
    /// Total de `chunked` além do qual o pedaço seguinte nem é lido: 8 MB
    /// no HTML; o GET binário (`resposta_binaria`) define o seu.
    limite: usize,
}

enum Modo {
    /// 3xx: nada é lido.
    Vazio,
    Fixo {
        declarado: usize,
        restante: usize,
    },
    AteOFim,
    EmPedacos {
        restante: usize,
        total: usize,
        fim: bool,
    },
}

impl<R> Corpo<R> {
    /// `Content-Length` declarado (sem `chunked`), antes de qualquer leitura.
    pub(crate) fn tamanho_declarado(&self) -> Option<usize> {
        match self.modo {
            Modo::Fixo { declarado, .. } => Some(declarado),
            _ => None,
        }
    }

    /// Troca o limite do total em `chunked` (o do GET binário).
    pub(crate) fn limitar_em_pedacos(&mut self, limite: usize) {
        self.limite = limite;
    }
}

/// Marca o erro de leitura de um pedaço que levaria o corpo além do limite.
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

fn invalido() -> io::Error {
    io::Error::from(io::ErrorKind::InvalidData)
}

fn ler_linha_io<R: BufRead>(leitor: &mut R) -> io::Result<String> {
    let mut bytes = Vec::new();
    leitor
        .take(MAX_LINHA as u64 + 1)
        .read_until(b'\n', &mut bytes)?;
    if bytes.len() > MAX_LINHA || !bytes.ends_with(b"\n") {
        return Err(invalido());
    }
    bytes.pop();
    if bytes.ends_with(b"\r") {
        bytes.pop();
    }
    String::from_utf8(bytes).map_err(|_| invalido())
}

fn ler_linha<R: BufRead>(leitor: &mut R) -> Result<String, FalhaTransporte> {
    ler_linha_io(leitor).map_err(|_| FalhaTransporte::Rede)
}

/// Linha de cabeçalho: UTF-8 ou, se não for, ISO-8859-1 (o nome do
/// arquivo no `Content-Disposition` pode vir assim, Refs #60). Sem isso,
/// um nome acentuado derrubaria a resposta inteira como falha de rede.
fn ler_linha_de_cabecalho<R: BufRead>(leitor: &mut R) -> Result<String, FalhaTransporte> {
    let mut bytes = Vec::new();
    leitor
        .take(MAX_LINHA as u64 + 1)
        .read_until(b'\n', &mut bytes)
        .map_err(|_| FalhaTransporte::Rede)?;
    if bytes.len() > MAX_LINHA || !bytes.ends_with(b"\n") {
        return Err(FalhaTransporte::Rede);
    }
    bytes.pop();
    if bytes.ends_with(b"\r") {
        bytes.pop();
    }
    Ok(String::from_utf8(bytes)
        .unwrap_or_else(|erro| erro.into_bytes().into_iter().map(char::from).collect()))
}

fn status_da_linha(linha: &str) -> Option<u16> {
    let resto = linha.strip_prefix("HTTP/1.")?;
    let mut partes = resto.splitn(3, ' ');
    let versao = partes.next()?;
    let codigo = partes.next()?;
    if !matches!(versao, "0" | "1")
        || codigo.len() != 3
        || !codigo.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    codigo.parse().ok().filter(|c| (100..600).contains(c))
}

impl<R: BufRead> Corpo<R> {
    /// Lê até `buf.len()` bytes de uma fonte de `restante` bytes conhecidos.
    fn ler_parte(&mut self, buf: &mut [u8], restante: usize) -> io::Result<usize> {
        let quanto = buf.len().min(restante);
        let lido = self.leitor.read(&mut buf[..quanto])?;
        if lido == 0 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }
        Ok(lido)
    }

    fn ler_em_pedacos(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let Modo::EmPedacos {
            restante,
            total,
            fim,
        } = self.modo
        else {
            return Ok(0);
        };
        if fim {
            return Ok(0);
        }
        let restante = if restante == 0 {
            let linha = ler_linha_io(&mut self.leitor)?;
            let tamanho = linha.split(';').next().unwrap_or_default().trim();
            let tamanho = usize::from_str_radix(tamanho, 16).map_err(|_| invalido())?;
            if tamanho == 0 {
                // Trailers terminam em linha vazia; EOF/truncamento é falha.
                while !ler_linha_io(&mut self.leitor)?.is_empty() {}
                self.modo = Modo::EmPedacos {
                    restante: 0,
                    total,
                    fim: true,
                };
                return Ok(0);
            }
            if total.saturating_add(tamanho) > self.limite {
                return Err(io::Error::other(CorpoExcessivo));
            }
            tamanho
        } else {
            restante
        };
        let lido = self.ler_parte(buf, restante)?;
        if lido == restante && !ler_linha_io(&mut self.leitor)?.is_empty() {
            return Err(invalido());
        }
        self.modo = Modo::EmPedacos {
            restante: restante - lido,
            total: total + lido,
            fim: false,
        };
        Ok(lido)
    }
}

impl<R: BufRead> Read for Corpo<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        match self.modo {
            Modo::Vazio | Modo::Fixo { restante: 0, .. } => Ok(0),
            Modo::AteOFim => self.leitor.read(buf),
            Modo::Fixo {
                declarado,
                restante,
            } => {
                let lido = self.ler_parte(buf, restante)?;
                self.modo = Modo::Fixo {
                    declarado,
                    restante: restante - lido,
                };
                Ok(lido)
            }
            Modo::EmPedacos { .. } => self.ler_em_pedacos(buf),
        }
    }
}

impl<R: BufRead> Resposta<R> {
    /// Lê status e cabeçalhos; o corpo fica no leitor, em fluxo.
    pub(crate) fn ler(mut leitor: R) -> Result<Self, FalhaTransporte> {
        let status = status_da_linha(&ler_linha(&mut leitor)?).ok_or(FalhaTransporte::Rede)?;
        let mut cabecalhos = Vec::new();
        loop {
            let linha = ler_linha_de_cabecalho(&mut leitor)?;
            if linha.is_empty() {
                break;
            }
            let (nome, valor) = linha.split_once(':').ok_or(FalhaTransporte::Rede)?;
            if cabecalhos.len() >= MAX_CABECALHOS || nome.is_empty() || nome.contains(' ') {
                return Err(FalhaTransporte::Rede);
            }
            cabecalhos.push((nome.to_owned(), valor.trim().to_owned()));
        }
        let mut resposta = Self {
            status,
            cabecalhos,
            corpo: Corpo {
                leitor,
                modo: Modo::Vazio,
                limite: crate::MAX_HTML_BYTES,
            },
        };
        if (300..400).contains(&status) {
            return Ok(resposta);
        }
        let em_pedacos = resposta
            .cabecalho("transfer-encoding")
            .is_some_and(|v| v.eq_ignore_ascii_case("chunked"));
        resposta.corpo.modo = if em_pedacos {
            Modo::EmPedacos {
                restante: 0,
                total: 0,
                fim: false,
            }
        } else if let Some(tamanho) = resposta.cabecalho("content-length") {
            let declarado: usize = tamanho.parse().map_err(|_| FalhaTransporte::Rede)?;
            Modo::Fixo {
                declarado,
                restante: declarado,
            }
        } else {
            Modo::AteOFim
        };
        Ok(resposta)
    }
}

#[cfg(test)]
#[path = "tests_resposta_http.rs"]
mod tests_resposta_http;
