//! Túnel HTTP `CONNECT` para o cliente HTTPS do login, quando o ambiente
//! declara um proxy. Contrato: contracts/cli/sessao-login.md, "Proxy".
//!
//! Variáveis, nesta ordem, só a primeira não vazia vale: `https_proxy`,
//! `HTTPS_PROXY`, `all_proxy`, `ALL_PROXY` (o destino é sempre HTTPS;
//! `http_proxy` não vale). Exceções: `no_proxy`, senão `NO_PROXY`.
//! Endereço `[http://][usuario[:senha]@]host:porta[/]`; credencial só no
//! `Proxy-Authorization: Basic`, nunca em `Debug` nem em mensagem.
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Variáveis do proxy, na ordem de preferência.
pub(crate) const VARIAVEIS_DO_PROXY: [&str; 4] =
    ["https_proxy", "HTTPS_PROXY", "all_proxy", "ALL_PROXY"];

/// Variáveis da lista de exceções, na ordem de preferência.
pub(crate) const VARIAVEIS_DE_EXCECAO: [&str; 2] = ["no_proxy", "NO_PROXY"];

/// Recusas do proxy, uma por fato.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub fn subcodigo(&self) -> &'static str {
        "nao_implementado"
    }

    pub fn mensagem(&self) -> &'static str {
        "nao_implementado"
    }
}

/// Proxy interpretado; `autorizacao` é o valor Basic já codificado.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Proxy {
    host: String,
    porta: u16,
    autorizacao: Option<String>,
}

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

/// Proxy a usar para `host` (sempre HTTPS, porta 443), segundo `ambiente`.
pub(crate) fn proxy_para(
    _host: &str,
    _ambiente: impl Fn(&str) -> Option<String>,
) -> Result<Option<Proxy>, FalhaDoProxy> {
    Ok(None)
}

/// Interpreta o endereço do proxy.
pub(crate) fn interpretar(_endereco: &str) -> Result<Proxy, FalhaDoProxy> {
    Err(FalhaDoProxy::EnderecoInvalido)
}

/// `host` está na lista de exceções.
pub(crate) fn dispensado(_host: &str, _lista: &str) -> bool {
    false
}

/// Pedido `CONNECT host:443` com a credencial, se houver.
pub(crate) fn pedido_connect(_proxy: &Proxy, _host: &str) -> String {
    String::new()
}

/// Envia o `CONNECT` e lê a resposta até a linha vazia, byte a byte: nada
/// do fluxo além dos cabeçalhos é consumido.
pub(crate) fn abrir_tunel<S: Read + Write>(
    _fluxo: &mut S,
    _proxy: &Proxy,
    _host: &str,
) -> Result<(), FalhaDoProxy> {
    Err(FalhaDoProxy::RespostaNaoInterpretavel)
}

/// Conecta ao proxy e abre o túnel para `host:443`.
pub(crate) fn conectar_via(
    _proxy: &Proxy,
    _host: &str,
    _tempo: Duration,
) -> Result<TcpStream, FalhaDoProxy> {
    Err(FalhaDoProxy::Inacessivel)
}

#[cfg(test)]
#[path = "tests_proxy.rs"]
mod tests_proxy;
