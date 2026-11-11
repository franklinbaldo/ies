//! Cliente HTTPS único do IES: login, GET do Controle e POST dos comandos,
//! sobre `ureq` com `native-tls`. Uma tentativa por pedido, sem
//! redirecionamento e sem retry; proxy do ambiente (`HTTPS_PROXY`) pelo
//! próprio `ureq`. O corpo é entregue em fluxo (`resposta_http`): o limite
//! de leitura é de quem consome. Contrato: contracts/cli.md, "Transporte HTTPS".
use super::resposta_http::{Corpo, Resposta};
use super::{FalhaTransporte, RespostaWeb};
use crate::instalacao::host_valido;
use std::io::{BufRead, BufReader, Read};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

const TEMPO_LIMITE: Duration = Duration::from_secs(30);
/// Identificação honesta do cliente; nenhuma instalação a exigiu.
const AGENTE: &str = concat!("ies/", env!("CARGO_PKG_VERSION"));

/// Corpo em fluxo de uma resposta real.
pub(crate) type CorpoHttps = Corpo<BufReader<ureq::BodyReader<'static>>>;

/// Corpo e cabeçalhos próprios de um POST.
pub(crate) struct Envio<'a> {
    pub(crate) tipo: &'a str,
    pub(crate) origem: String,
    pub(crate) referer: &'a str,
    pub(crate) corpo: &'a [u8],
}

/// Corpo de POST multipart preparado sem copiar o arquivo inteiro.
pub(crate) struct EnvioEmFluxo<'a, R: Read> {
    pub(crate) tipo: &'a str,
    pub(crate) origem: &'a str,
    pub(crate) referer: &'a str,
    pub(crate) tamanho: u64,
    pub(crate) corpo: R,
}

/// Só URL HTTPS absoluta, sem porta, fragmento ou byte fora do ASCII visível.
fn validar(url: &str) -> Result<(), FalhaTransporte> {
    let host = url
        .strip_prefix("https://")
        .and_then(|resto| resto.split_once('/'))
        .map(|(host, _)| host)
        .ok_or(FalhaTransporte::UrlInvalida)?;
    if !host_valido(host) || !url.bytes().all(|b| (33..=126).contains(&b)) || url.contains('#') {
        return Err(FalhaTransporte::UrlInvalida);
    }
    Ok(())
}

fn agente() -> &'static ureq::Agent {
    static AGENTE_HTTP: OnceLock<ureq::Agent> = OnceLock::new();
    AGENTE_HTTP.get_or_init(|| {
        ureq::Agent::config_builder()
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .provider(ureq::tls::TlsProvider::NativeTls)
                    // Raízes do sistema (e `SSL_CERT_FILE`), como o TLS nativo.
                    .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                    .build(),
            )
            .proxy(ureq::Proxy::try_from_env())
            .max_redirects(0)
            .max_redirects_will_error(false)
            .http_status_as_error(false)
            .user_agent(AGENTE)
            .timeout_connect(Some(TEMPO_LIMITE))
            .timeout_per_read(Some(TEMPO_LIMITE))
            .timeout_per_write(Some(TEMPO_LIMITE))
            .build()
            .into()
    })
}

fn falha(erro: &ureq::Error) -> FalhaTransporte {
    match erro {
        ureq::Error::BadUri(_) => FalhaTransporte::UrlInvalida,
        ureq::Error::InvalidProxyUrl | ureq::Error::ConnectProxyFailed(_) => FalhaTransporte::Proxy,
        ureq::Error::StatusCode(_)
        | ureq::Error::Http(_)
        | ureq::Error::Protocol(_)
        | ureq::Error::Io(_)
        | ureq::Error::Timeout(_)
        | ureq::Error::HostNotFound
        | ureq::Error::RedirectFailed
        | ureq::Error::ConnectionFailed
        | ureq::Error::BodyExceedsLimit(_)
        | ureq::Error::TooManyRedirects
        | ureq::Error::Tls(_)
        | ureq::Error::Pem(_)
        | ureq::Error::NativeTls(_)
        | ureq::Error::Der(_)
        | ureq::Error::RequireHttpsOnly(_)
        | ureq::Error::LargeResponseHeader(..)
        | ureq::Error::TlsRequired
        | ureq::Error::Other(_)
        | _ => FalhaTransporte::Rede,
    }
}

fn resposta(
    resposta: ureq::http::Response<ureq::Body>,
) -> Resposta<BufReader<ureq::BodyReader<'static>>> {
    let status = resposta.status().as_u16();
    let cabecalhos = resposta
        .headers()
        .iter()
        .map(|(nome, valor)| {
            (
                nome.as_str().to_owned(),
                super::resposta_http::texto_do_cabecalho(valor.as_bytes()),
            )
        })
        .collect();
    let corpo = Corpo::novo(
        BufReader::new(resposta.into_body().into_reader()),
        (300..400).contains(&status),
    );
    Resposta::nova(status, cabecalhos, corpo)
}

/// Pedidos reais tentados por este processo (todos os comandos e o
/// login), registrados ANTES do envio: só o relogin das leituras os lê.
static PEDIDOS_DO_PROCESSO: AtomicUsize = AtomicUsize::new(0);

/// Total de pedidos reais tentados até agora pelo processo.
pub(crate) fn pedidos_do_processo() -> usize {
    PEDIDOS_DO_PROCESSO.load(Ordering::Relaxed)
}

/// Um pedido, uma tentativa, sem redirecionamento nem retry.
pub(crate) fn requisitar(
    metodo: &str,
    url: &str,
    cookie: &str,
    envio: Option<&Envio<'_>>,
) -> Result<Resposta<BufReader<ureq::BodyReader<'static>>>, FalhaTransporte> {
    validar(url)?;
    PEDIDOS_DO_PROCESSO.fetch_add(1, Ordering::Relaxed);
    let resultado = match envio {
        Some(e) if metodo == "POST" => cabecalhos(agente().post(url), cookie)
            .header("Content-Type", e.tipo)
            .header("Origin", &e.origem)
            .header("Referer", e.referer)
            .send(e.corpo),
        _ => cabecalhos(agente().get(url), cookie).call(),
    };
    resultado.map(resposta).map_err(|erro| falha(&erro))
}

/// POST multipart de tamanho conhecido, transmitido diretamente do leitor.
/// Uma única tentativa. A operação avalia o efeito de eventual interrupção.
pub(crate) fn requisitar_em_fluxo<R: Read>(
    url: &str,
    cookie: &str,
    envio: EnvioEmFluxo<'_, R>,
) -> Result<Resposta<BufReader<ureq::BodyReader<'static>>>, FalhaTransporte> {
    validar(url)?;
    PEDIDOS_DO_PROCESSO.fetch_add(1, Ordering::Relaxed);
    let mut corpo = envio.corpo.take(envio.tamanho);
    cabecalhos(agente().post(url), cookie)
        .header("Content-Type", envio.tipo)
        .header("Content-Length", envio.tamanho.to_string())
        .header("Origin", envio.origem)
        .header("Referer", envio.referer)
        .send(ureq::SendBody::from_reader(&mut corpo))
        .map(resposta)
        .map_err(|erro| falha(&erro))
}

fn cabecalhos<B>(pedido: ureq::RequestBuilder<B>, cookie: &str) -> ureq::RequestBuilder<B> {
    let pedido = pedido
        .header("Accept", "text/html,application/xhtml+xml")
        .header("Accept-Encoding", "identity");
    if cookie.is_empty() {
        pedido
    } else {
        pedido.header("Cookie", cookie)
    }
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
