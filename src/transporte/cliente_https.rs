//! Cliente HTTPS único do IES: login, GET do Controle e POST dos comandos.
//! HTTP/1.1 sobre `native-tls`, uma conexão por pedido, `Connection: close`,
//! 30 s de tempo limite, sem redirecionamento e sem retry. Com proxy no
//! ambiente (`transporte::proxy`), um `CONNECT` por pedido e o TLS com o
//! host do SEI (SNI e certificado) através do túnel. A resposta guarda todos
//! os cabeçalhos e entrega o corpo em fluxo (`resposta_http`): o limite de
//! leitura é de quem consome. Contrato: contracts/cli.md, "Transporte HTTPS".
use super::resposta_http::{Corpo, Resposta};
use super::{FalhaTransporte, RespostaWeb, proxy};
use crate::erro_html::ErroHtml;
use crate::instalacao::host_valido;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

const TEMPO_LIMITE: Duration = Duration::from_secs(30);
/// Identificação honesta do cliente; nenhuma instalação a exigiu.
const AGENTE: &str = concat!("ies/", env!("CARGO_PKG_VERSION"));

/// Fluxo TLS real até o SEI.
pub(crate) type FluxoTls = native_tls::TlsStream<TcpStream>;

/// Corpo em fluxo de uma resposta real.
pub(crate) type CorpoHttps = Corpo<BufReader<FluxoTls>>;

/// Corpo e cabeçalhos próprios de um POST.
pub(crate) struct Envio<'a> {
    pub(crate) tipo: &'a str,
    pub(crate) origem: String,
    pub(crate) referer: &'a str,
    pub(crate) corpo: &'a [u8],
}

/// Host e alvo (`/caminho?consulta`) de uma URL HTTPS sem porta.
fn alvo(url: &str) -> Result<(&str, &str), FalhaTransporte> {
    let resto = url
        .strip_prefix("https://")
        .ok_or(ErroHtml::valor("url_invalida").com_mensagem("URL assinada da caixa inválida"))?;
    let inicio = resto
        .find('/')
        .ok_or(ErroHtml::valor("url_invalida").com_mensagem("URL assinada da caixa inválida"))?;
    let (host, caminho) = resto.split_at(inicio);
    if !host_valido(host) || !url.bytes().all(|b| (33..=126).contains(&b)) || url.contains('#') {
        return Err(ErroHtml::valor("url_invalida")
            .com_mensagem("URL assinada da caixa inválida")
            .into());
    }
    Ok((host, caminho))
}

fn ambiente_do_processo(nome: &str) -> Option<String> {
    std::env::var(nome).ok()
}

/// TLS real com o host do SEI, sobre a conexão direta ou o túnel.
pub(crate) fn tls_nativo(host: &str, tcp: TcpStream) -> Result<FluxoTls, FalhaTransporte> {
    let conector = native_tls::TlsConnector::new().map_err(|_| FalhaTransporte::Rede)?;
    conector
        .connect(host, tcp)
        .map_err(|_| FalhaTransporte::Rede)
}

/// Conexão TCP a `host:443`, direta ou pelo túnel do proxy que `ambiente`
/// declarar (`transporte::proxy`), já com os tempos limite.
pub(crate) fn abrir_conexao(
    host: &str,
    ambiente: impl Fn(&str) -> Option<String>,
) -> Result<TcpStream, FalhaTransporte> {
    if let Some(proxy) = proxy::proxy_para(host, ambiente)? {
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

fn montar_pedido(
    metodo: &str,
    host: &str,
    caminho: &str,
    cookie: &str,
    envio: Option<&Envio<'_>>,
) -> String {
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
    pedido
}

/// Um pedido, uma conexão: `ambiente` decide o proxy e `envolver` faz o
/// TLS (substituível nos testes). Devolve cabeçalhos com o corpo em fluxo.
pub(crate) fn requisitar_via<S: Read + Write>(
    metodo: &str,
    url: &str,
    cookie: &str,
    envio: Option<&Envio<'_>>,
    ambiente: impl Fn(&str) -> Option<String>,
    envolver: impl FnOnce(&str, TcpStream) -> Result<S, FalhaTransporte>,
) -> Result<Resposta<BufReader<S>>, FalhaTransporte> {
    let (host, caminho) = alvo(url)?;
    let pedido = montar_pedido(metodo, host, caminho, cookie, envio);
    let tcp = abrir_conexao(host, ambiente)?;
    let mut fluxo = envolver(host, tcp)?;
    fluxo
        .write_all(pedido.as_bytes())
        .and_then(|()| fluxo.write_all(envio.map_or(&[][..], |e| e.corpo)))
        .and_then(|()| fluxo.flush())
        .map_err(|_| FalhaTransporte::Rede)?;
    Resposta::ler(BufReader::new(fluxo))
}

/// Pedidos reais tentados por este processo (todos os comandos e o
/// login), registrados ANTES do envio: só o relogin das leituras os lê.
static PEDIDOS_DO_PROCESSO: AtomicUsize = AtomicUsize::new(0);

/// Total de pedidos reais tentados até agora pelo processo.
pub(crate) fn pedidos_do_processo() -> usize {
    PEDIDOS_DO_PROCESSO.load(Ordering::Relaxed)
}

/// Pedido real: proxy do ambiente do processo e TLS nativo.
pub(crate) fn requisitar(
    metodo: &str,
    url: &str,
    cookie: &str,
    envio: Option<&Envio<'_>>,
) -> Result<Resposta<BufReader<FluxoTls>>, FalhaTransporte> {
    PEDIDOS_DO_PROCESSO.fetch_add(1, Ordering::Relaxed);
    requisitar_via(metodo, url, cookie, envio, ambiente_do_processo, tls_nativo)
}

/// Resposta como `RespostaWeb` dos comandos, sem ler o corpo.
pub(crate) fn resposta_web<R: BufRead>(resposta: Resposta<R>) -> RespostaWeb<Corpo<R>> {
    let mime = resposta
        .cabecalho("content-type")
        .unwrap_or_default()
        .to_owned();
    RespostaWeb {
        status_code: i32::from(resposta.status),
        mime,
        content_length: resposta.corpo.tamanho_declarado(),
        localizacao: resposta.cabecalho("location").map(str::to_owned),
        corpo: resposta.corpo,
    }
}
