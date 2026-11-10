//! Túnel HTTP `CONNECT` para o cliente HTTPS único (`cliente_https`), em
//! todos os comandos, quando o ambiente declara um proxy. Contrato:
//! contracts/cli.md, "Transporte HTTPS", e contracts/cli/sessao-login.md, "Proxy".
//!
//! Variáveis, nesta ordem, só a primeira não vazia vale: `https_proxy`,
//! `HTTPS_PROXY`, `all_proxy`, `ALL_PROXY` (o destino é sempre HTTPS;
//! `http_proxy` não vale). Exceções: `no_proxy`, senão `NO_PROXY`.
//! Endereço `[http://][usuario[:senha]@]host:porta[/]`; credencial só no
//! `Proxy-Authorization: Basic`, nunca em `Debug` nem em mensagem.
use std::fmt;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Variáveis do proxy, na ordem de preferência.
pub(crate) const VARIAVEIS_DO_PROXY: [&str; 4] =
    ["https_proxy", "HTTPS_PROXY", "all_proxy", "ALL_PROXY"];

/// Variáveis da lista de exceções, na ordem de preferência.
pub(crate) const VARIAVEIS_DE_EXCECAO: [&str; 2] = ["no_proxy", "NO_PROXY"];

/// Limite da resposta do proxy ao `CONNECT` (linha de status e cabeçalhos).
const MAX_RESPOSTA: usize = 16 * 1024;

/// Recusas do proxy, uma por fato.
#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub enum FalhaDoProxy {
    /// O valor da variável não segue `[http://][cred@]host:porta[/]`.
    EnderecoInvalido,
    /// Esquema diferente de `http://` (`https://`, `socks5://`...).
    EsquemaNaoSuportado,
    /// DNS ou conexão TCP ao proxy falhou; nada foi enviado ao SEI.
    Inacessivel,
    /// O proxy respondeu ao `CONNECT` fora do HTTP/1.x esperado.
    RespostaNaoInterpretavel,
    /// O proxy respondeu ao `CONNECT` com status fora de 2xx.
    RecusouOTunel { status: u16 },
}

impl FalhaDoProxy {
    pub fn mensagem(&self) -> &'static str {
        match self {
            Self::EnderecoInvalido => "endereço de proxy do ambiente fora de http://host:porta",
            Self::EsquemaNaoSuportado => "só proxy http:// é suportado; nada foi enviado",
            Self::Inacessivel => "não foi possível conectar ao proxy; nada foi enviado ao SEI",
            Self::RespostaNaoInterpretavel => "o proxy respondeu ao CONNECT fora do HTTP/1.x",
            Self::RecusouOTunel { .. } => "o proxy recusou o túnel CONNECT para o SEI",
        }
    }
}

/// Proxy interpretado; `autorizacao` é o valor Basic já codificado.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Proxy {
    host: String,
    porta: u16,
    autorizacao: Option<String>,
}

/// Leitura para os testes; o transporte usa os campos.
#[cfg(test)]
impl Proxy {
    pub(crate) fn host(&self) -> &str {
        &self.host
    }

    pub(crate) fn porta(&self) -> u16 {
        self.porta
    }

    pub(crate) fn tem_autorizacao(&self) -> bool {
        self.autorizacao.is_some()
    }
}

/// A credencial nunca aparece.
impl fmt::Debug for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Proxy")
            .field("host", &self.host)
            .field("porta", &self.porta)
            .field("autorizacao", &self.autorizacao.as_ref().map(|_| "***"))
            .finish()
    }
}

fn primeira(ambiente: &impl Fn(&str) -> Option<String>, nomes: &[&str]) -> Option<String> {
    nomes
        .iter()
        .filter_map(|nome| ambiente(nome))
        .find(|valor| !valor.trim().is_empty())
}

/// Proxy a usar para `host` (sempre HTTPS, porta 443), segundo `ambiente`.
/// A exceção é conferida antes: um host dispensado ignora até proxy inválido.
pub(crate) fn proxy_para(
    host: &str,
    ambiente: impl Fn(&str) -> Option<String>,
) -> Result<Option<Proxy>, FalhaDoProxy> {
    if primeira(&ambiente, &VARIAVEIS_DE_EXCECAO).is_some_and(|lista| dispensado(host, &lista)) {
        return Ok(None);
    }
    primeira(&ambiente, &VARIAVEIS_DO_PROXY)
        .map(|endereco| interpretar(endereco.trim()))
        .transpose()
}

fn porta(texto: &str) -> Result<u16, FalhaDoProxy> {
    if texto.is_empty() || !texto.bytes().all(|b| b.is_ascii_digit()) {
        return Err(FalhaDoProxy::EnderecoInvalido);
    }
    texto
        .parse::<u16>()
        .ok()
        .filter(|p| *p != 0)
        .ok_or(FalhaDoProxy::EnderecoInvalido)
}

/// `%XX` decodificado; `%` sem dois hexadecimais é endereço inválido.
fn decodificar_percentual(texto: &str) -> Result<Vec<u8>, FalhaDoProxy> {
    let mut saida = Vec::with_capacity(texto.len());
    let mut resto = texto;
    while let Some(inicio) = resto.find('%') {
        saida.extend_from_slice(&resto.as_bytes()[..inicio]);
        let par = resto
            .get(inicio + 1..inicio + 3)
            .filter(|par| par.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or(FalhaDoProxy::EnderecoInvalido)?;
        saida.push(u8::from_str_radix(par, 16).map_err(|_| FalhaDoProxy::EnderecoInvalido)?);
        resto = &resto[inicio + 3..];
    }
    saida.extend_from_slice(resto.as_bytes());
    Ok(saida)
}

fn host_do_proxy(texto: &str) -> bool {
    !texto.is_empty()
        && texto
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_'))
}

/// `host:porta`, ou `[ipv6]:porta`.
fn servidor(texto: &str) -> Result<(&str, u16), FalhaDoProxy> {
    use FalhaDoProxy::EnderecoInvalido as Invalido;
    if let Some(resto) = texto.strip_prefix('[') {
        let (host, depois) = resto.split_once(']').ok_or(Invalido)?;
        let v6 = !host.is_empty() && host.bytes().all(|b| b.is_ascii_hexdigit() || b == b':');
        if !v6 {
            return Err(Invalido);
        }
        return Ok((host, porta(depois.strip_prefix(':').ok_or(Invalido)?)?));
    }
    let (host, porta_texto) = texto.split_once(':').ok_or(Invalido)?;
    if !host_do_proxy(host) {
        return Err(Invalido);
    }
    Ok((host, porta(porta_texto)?))
}

/// Valor Basic de `usuario[:senha]` (cada parte com `%XX` decodificado).
fn basic(credencial: &str) -> Result<String, FalhaDoProxy> {
    use base64::Engine as _;
    let mut par = Vec::new();
    match credencial.split_once(':') {
        Some((usuario, senha)) => {
            par.extend(decodificar_percentual(usuario)?);
            par.push(b':');
            par.extend(decodificar_percentual(senha)?);
        }
        None => par.extend(decodificar_percentual(credencial)?),
    }
    Ok(base64::engine::general_purpose::STANDARD.encode(par))
}

/// Interpreta `[http://][usuario[:senha]@]host:porta[/]`; IPv6 entre colchetes.
pub(crate) fn interpretar(endereco: &str) -> Result<Proxy, FalhaDoProxy> {
    let autoridade = match endereco.split_once("://") {
        Some((esquema, resto)) if esquema.eq_ignore_ascii_case("http") => resto,
        Some(_) => return Err(FalhaDoProxy::EsquemaNaoSuportado),
        None => endereco,
    };
    let autoridade = autoridade.strip_suffix('/').unwrap_or(autoridade);
    if autoridade
        .bytes()
        .any(|b| matches!(b, b'/' | b'?' | b'#') || !(33..=126).contains(&b))
    {
        return Err(FalhaDoProxy::EnderecoInvalido);
    }
    let (credencial, resto) = match autoridade.rsplit_once('@') {
        Some((credencial, resto)) => (Some(credencial), resto),
        None => (None, autoridade),
    };
    let (host, porta) = servidor(resto)?;
    Ok(Proxy {
        host: host.to_owned(),
        porta,
        autorizacao: credencial.map(basic).transpose()?,
    })
}

/// `host` está na lista de exceções (vírgulas): `*` dispensa todos; `.sufixo`
/// dispensa o domínio e seus subdomínios; outra entrada é host exato. Sem
/// distinção de caixa; entrada com porta ou curinga parcial não casa.
pub(crate) fn dispensado(host: &str, lista: &str) -> bool {
    let host = host.to_ascii_lowercase();
    lista
        .split(',')
        .map(|entrada| entrada.trim().to_ascii_lowercase())
        .filter(|entrada| !entrada.is_empty())
        .any(|entrada| {
            if entrada == "*" {
                return true;
            }
            match entrada.strip_prefix('.') {
                Some(dominio) => {
                    host == dominio
                        || host
                            .strip_suffix(dominio)
                            .is_some_and(|antes| antes.ends_with('.'))
                }
                None => host == entrada,
            }
        })
}

/// Pedido `CONNECT host:443` com a credencial, se houver.
pub(crate) fn pedido_connect(proxy: &Proxy, host: &str) -> String {
    let mut pedido = format!("CONNECT {host}:443 HTTP/1.1\r\nHost: {host}:443\r\n");
    if let Some(basic) = &proxy.autorizacao {
        pedido.push_str("Proxy-Authorization: Basic ");
        pedido.push_str(basic);
        pedido.push_str("\r\n");
    }
    pedido.push_str("\r\n");
    pedido
}

/// Status da linha `HTTP/1.x NNN[ motivo]` do proxy.
fn status_da_linha(linha: &[u8]) -> Option<u16> {
    let resto = linha
        .strip_prefix(b"HTTP/1.0 ")
        .or_else(|| linha.strip_prefix(b"HTTP/1.1 "))?;
    let codigo = resto.get(..3)?;
    if !codigo.iter().all(u8::is_ascii_digit) || !matches!(resto.get(3), None | Some(b' ')) {
        return None;
    }
    std::str::from_utf8(codigo)
        .ok()?
        .parse()
        .ok()
        .filter(|c| (100..600).contains(c))
}

/// Envia o `CONNECT` e lê a resposta até a linha vazia, byte a byte: nada
/// do fluxo além dos cabeçalhos é consumido.
pub(crate) fn abrir_tunel<S: Read + Write>(
    fluxo: &mut S,
    proxy: &Proxy,
    host: &str,
) -> Result<(), FalhaDoProxy> {
    use FalhaDoProxy::RespostaNaoInterpretavel as Ilegivel;
    fluxo
        .write_all(pedido_connect(proxy, host).as_bytes())
        .and_then(|()| fluxo.flush())
        .map_err(|_| FalhaDoProxy::Inacessivel)?;
    let mut resposta = Vec::new();
    let mut byte = [0_u8; 1];
    while !(resposta.ends_with(b"\r\n\r\n") || resposta.ends_with(b"\n\n")) {
        if resposta.len() >= MAX_RESPOSTA || fluxo.read(&mut byte).map_err(|_| Ilegivel)? == 0 {
            return Err(Ilegivel);
        }
        resposta.push(byte[0]);
    }
    let fim = resposta.iter().position(|b| *b == b'\n').unwrap_or(0);
    let linha = &resposta[..fim];
    let status = status_da_linha(linha.strip_suffix(b"\r").unwrap_or(linha)).ok_or(Ilegivel)?;
    if (200..300).contains(&status) {
        Ok(())
    } else {
        Err(FalhaDoProxy::RecusouOTunel { status })
    }
}

/// Conecta ao proxy (cada endereço resolvido é uma tentativa de conexão TCP,
/// não de pedido) e abre o túnel para `host:443`, já com os tempos limite.
pub(crate) fn conectar_via(
    proxy: &Proxy,
    host: &str,
    tempo: Duration,
) -> Result<TcpStream, FalhaDoProxy> {
    let mut tcp = (proxy.host.as_str(), proxy.porta)
        .to_socket_addrs()
        .map_err(|_| FalhaDoProxy::Inacessivel)?
        .find_map(|endereco| TcpStream::connect_timeout(&endereco, tempo).ok())
        .ok_or(FalhaDoProxy::Inacessivel)?;
    tcp.set_read_timeout(Some(tempo))
        .and_then(|()| tcp.set_write_timeout(Some(tempo)))
        .map_err(|_| FalhaDoProxy::Inacessivel)?;
    abrir_tunel(&mut tcp, proxy, host)?;
    Ok(tcp)
}

#[cfg(test)]
#[path = "tests_proxy.rs"]
mod tests_proxy;
