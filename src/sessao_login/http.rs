//! Cliente HTTPS mínimo do login: HTTP/1.1 sobre `native-tls`, uma
//! conexão por pedido, `Connection: close`, sem redirecionamento, sem
//! retry, 8 MB durante o recebimento. Existe porque o `minreq` 2.14.1 guarda
//! um valor por nome de cabeçalho (`HashMap::insert` em `read_metadata`) e
//! perderia todos os `Set-Cookie` de uma resposta menos o último. Com proxy
//! no ambiente (`transporte::proxy`), um `CONNECT` por pedido e o TLS com o
//! host do SEI (SNI e certificado) através do túnel.
use super::{RespostaHttp, TransporteDeLogin};
use crate::formulario_web::SubmissaoDerivada;
use crate::instalacao::host_valido;
use crate::transporte::{FalhaTransporte, ler_limitado, proxy};
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const TEMPO_LIMITE: Duration = Duration::from_secs(30);
const MAX_LINHA: usize = 16 * 1024;
const MAX_CABECALHOS: usize = 200;
/// Identificação honesta do cliente; nenhuma instalação a exigiu.
const AGENTE: &str = concat!("ies/", env!("CARGO_PKG_VERSION"));

/// Transporte real do login.
pub(crate) struct HttpsDoLogin;

/// Corpo e cabeçalhos próprios do POST.
struct Envio<'a> {
    tipo: &'a str,
    origem: String,
    referer: &'a str,
    corpo: &'a [u8],
}

impl TransporteDeLogin for HttpsDoLogin {
    fn get(&mut self, url: &str, cookie: &str) -> Result<RespostaHttp, FalhaTransporte> {
        requisitar("GET", url, cookie, None)
    }

    // Consumida numa única tentativa, como no `FnOnce` do envio.
    #[allow(clippy::needless_pass_by_value)]
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaHttp, FalhaTransporte> {
        let envio = Envio {
            tipo: SubmissaoDerivada::TIPO_DE_CONTEUDO,
            origem: submissao.origem(),
            referer: submissao.referer(),
            corpo: submissao.corpo(),
        };
        requisitar("POST", submissao.url(), cookie, Some(&envio))
    }
}

/// Host e alvo (`/caminho?consulta`) de uma URL HTTPS sem porta.
fn alvo(url: &str) -> Result<(&str, &str), FalhaTransporte> {
    let resto = url
        .strip_prefix("https://")
        .ok_or(FalhaTransporte::UrlInvalida)?;
    let inicio = resto.find('/').ok_or(FalhaTransporte::UrlInvalida)?;
    let (host, caminho) = resto.split_at(inicio);
    if !host_valido(host) || !url.bytes().all(|b| (33..=126).contains(&b)) || url.contains('#') {
        return Err(FalhaTransporte::UrlInvalida);
    }
    Ok((host, caminho))
}

fn requisitar(
    metodo: &str,
    url: &str,
    cookie: &str,
    envio: Option<&Envio<'_>>,
) -> Result<RespostaHttp, FalhaTransporte> {
    let (host, caminho) = alvo(url)?;
    let mut pedido = format!(
        "{metodo} {caminho} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: {AGENTE}\r\n\
         Accept: text/html,application/xhtml+xml\r\nAccept-Encoding: identity\r\n\
         Connection: close\r\n"
    );
    if !cookie.is_empty() {
        let _ = write!(pedido, "Cookie: {cookie}\r\n");
    }
    if let Some(envio) = envio {
        let _ = write!(
            pedido,
            "Content-Type: {}\r\nContent-Length: {}\r\nOrigin: {}\r\nReferer: {}\r\n",
            envio.tipo,
            envio.corpo.len(),
            envio.origem,
            envio.referer
        );
    }
    pedido.push_str("\r\n");
    let tcp = abrir_conexao(host, ambiente_do_processo)?;
    let conector = native_tls::TlsConnector::new().map_err(|_| FalhaTransporte::Rede)?;
    let mut tls = conector
        .connect(host, tcp)
        .map_err(|_| FalhaTransporte::Rede)?;
    tls.write_all(pedido.as_bytes())
        .and_then(|()| tls.write_all(envio.map_or(&[][..], |e| e.corpo)))
        .and_then(|()| tls.flush())
        .map_err(|_| FalhaTransporte::Rede)?;
    ler_resposta(&mut BufReader::new(tls))
}

fn ambiente_do_processo(nome: &str) -> Option<String> {
    std::env::var(nome).ok()
}

/// Conexão TCP a `host:443`, direta ou pelo túnel do proxy que `ambiente`
/// declarar (`transporte::proxy`), já com os tempos limite.
pub(crate) fn abrir_conexao(
    host: &str,
    ambiente: impl Fn(&str) -> Option<String>,
) -> Result<TcpStream, FalhaTransporte> {
    if let Some(proxy) = proxy::proxy_para(host, ambiente).map_err(FalhaTransporte::Proxy)? {
        return proxy::conectar_via(&proxy, host, TEMPO_LIMITE).map_err(FalhaTransporte::Proxy);
    }
    // Cada endereço resolvido é uma tentativa de conexão TCP, não de pedido:
    // nada foi enviado antes de uma delas abrir.
    let tcp = (host, 443)
        .to_socket_addrs()
        .map_err(|_| FalhaTransporte::Rede)?
        .find_map(|endereco| TcpStream::connect_timeout(&endereco, TEMPO_LIMITE).ok())
        .ok_or(FalhaTransporte::Rede)?;
    tcp.set_read_timeout(Some(TEMPO_LIMITE))
        .and_then(|()| tcp.set_write_timeout(Some(TEMPO_LIMITE)))
        .map_err(|_| FalhaTransporte::Rede)?;
    Ok(tcp)
}

fn ler_linha<R: BufRead>(leitor: &mut R) -> Result<String, FalhaTransporte> {
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
    String::from_utf8(bytes).map_err(|_| FalhaTransporte::Rede)
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

fn corpo_em_pedacos<R: BufRead>(leitor: &mut R) -> Result<Vec<u8>, FalhaTransporte> {
    let mut corpo = Vec::new();
    loop {
        let linha = ler_linha(leitor)?;
        let tamanho = linha.split(';').next().unwrap_or_default().trim();
        let tamanho = usize::from_str_radix(tamanho, 16).map_err(|_| FalhaTransporte::Rede)?;
        if tamanho == 0 {
            // Trailers até a linha vazia (ou o fim da conexão).
            while !ler_linha(leitor).unwrap_or_default().is_empty() {}
            return Ok(corpo);
        }
        if corpo.len() + tamanho > crate::MAX_HTML_BYTES {
            return Err(FalhaTransporte::RespostaExcessiva);
        }
        let inicio = corpo.len();
        corpo.resize(inicio + tamanho, 0);
        leitor
            .read_exact(&mut corpo[inicio..])
            .map_err(|_| FalhaTransporte::Rede)?;
        if !ler_linha(leitor)?.is_empty() {
            return Err(FalhaTransporte::Rede);
        }
    }
}

/// Lê status, todos os cabeçalhos e o corpo (não lido em 3xx).
pub(crate) fn ler_resposta<R: BufRead>(leitor: &mut R) -> Result<RespostaHttp, FalhaTransporte> {
    let status = status_da_linha(&ler_linha(leitor)?).ok_or(FalhaTransporte::Rede)?;
    let mut cabecalhos = Vec::new();
    loop {
        let linha = ler_linha(leitor)?;
        if linha.is_empty() {
            break;
        }
        let (nome, valor) = linha.split_once(':').ok_or(FalhaTransporte::Rede)?;
        if cabecalhos.len() >= MAX_CABECALHOS || nome.is_empty() || nome.contains(' ') {
            return Err(FalhaTransporte::Rede);
        }
        cabecalhos.push((nome.to_owned(), valor.trim().to_owned()));
    }
    let mut resposta = RespostaHttp {
        status,
        cabecalhos,
        corpo: Vec::new(),
    };
    if (300..400).contains(&status) {
        return Ok(resposta);
    }
    let em_pedacos = resposta
        .cabecalho("transfer-encoding")
        .is_some_and(|v| v.eq_ignore_ascii_case("chunked"));
    resposta.corpo = if em_pedacos {
        corpo_em_pedacos(leitor)?
    } else if let Some(tamanho) = resposta.cabecalho("content-length") {
        let tamanho: usize = tamanho.parse().map_err(|_| FalhaTransporte::Rede)?;
        if tamanho > crate::MAX_HTML_BYTES {
            return Err(FalhaTransporte::RespostaExcessiva);
        }
        let mut corpo = Vec::new();
        leitor
            .take(tamanho as u64)
            .read_to_end(&mut corpo)
            .map_err(|_| FalhaTransporte::Rede)?;
        if corpo.len() != tamanho {
            return Err(FalhaTransporte::Rede);
        }
        corpo
    } else {
        ler_limitado(leitor, crate::MAX_HTML_BYTES)?
    };
    Ok(resposta)
}
