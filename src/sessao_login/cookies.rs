//! Pote de cookies do login, limitado ao host da instalação: nome, valor e
//! `Path`. Todo pedido do login é HTTPS, então `Secure` não restringe.
//! Valores nunca aparecem em `Debug` nem em mensagens.
use super::redirecionamento::caminho_da_url;
use super::{ErroLogin, RespostaHttp};
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
struct Cookie {
    nome: String,
    valor: String,
    caminho: String,
}

pub(crate) struct PoteDeCookies {
    host: String,
    cookies: Vec<Cookie>,
}

impl fmt::Debug for PoteDeCookies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoteDeCookies")
            .field(
                "nomes",
                &self
                    .cookies
                    .iter()
                    .map(|c| c.nome.as_str())
                    .collect::<Vec<_>>(),
            )
            .field("valores", &"<redigido>")
            .finish_non_exhaustive()
    }
}

/// O que um `Set-Cookie` pede ao pote.
enum Efeito {
    Guardar(Cookie),
    Remover {
        nome: String,
        caminho: String,
    },
    /// `Domain` que não contém o host: ignorado (RFC 6265, 5.3).
    Ignorar,
}

/// Nome: `token` da RFC 7230, sem separadores.
fn nome_valido(nome: &str) -> bool {
    !nome.is_empty()
        && nome.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

/// `cookie-octet` da RFC 6265 menos `:`, que o arquivo de cookie recusa.
fn valor_valido(valor: &str) -> bool {
    valor
        .bytes()
        .all(|b| (0x21..=0x7e).contains(&b) && !matches!(b, b'"' | b',' | b';' | b'\\' | b':'))
}

/// `Path` padrão (RFC 6265, 5.1.4): o diretório do caminho do pedido.
fn caminho_padrao(url: &str) -> String {
    let caminho = caminho_da_url(url).unwrap_or("/");
    match caminho.rsplit_once('/') {
        Some((pasta, _)) if !pasta.is_empty() => pasta.to_owned(),
        _ => "/".to_owned(),
    }
}

/// Ano de `Expires` (o único campo de quatro dígitos da data HTTP).
fn ano_de_expires(valor: &str) -> Option<u64> {
    valor
        .split([' ', '-', ','])
        .find(|parte| parte.len() == 4 && parte.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|ano| ano.parse().ok())
}

fn ano_corrente() -> u64 {
    let segundos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    // Ano gregoriano médio: 365,2425 dias.
    1970 + segundos / 31_556_952
}

fn dominio_contem_o_host(dominio: &str, host: &str) -> bool {
    dominio == host
        || host
            .strip_suffix(dominio)
            .is_some_and(|antes| antes.ends_with('.'))
}

fn interpretar(set_cookie: &str, url: &str, host: &str) -> Result<Efeito, ErroLogin> {
    let mut partes = set_cookie.split(';');
    let par = partes.next().unwrap_or_default().trim();
    let (nome, valor) = par
        .split_once('=')
        .ok_or(ErroLogin::CookieRecebidoForaDoFormatoAceito)?;
    let (nome, valor) = (nome.trim(), valor.trim());
    if !nome_valido(nome) || !valor_valido(valor) {
        return Err(ErroLogin::CookieRecebidoForaDoFormatoAceito);
    }
    let mut caminho = None;
    let mut vencido = false;
    for atributo in partes {
        let (chave, conteudo) = atributo.split_once('=').unwrap_or((atributo, ""));
        let (chave, conteudo) = (chave.trim(), conteudo.trim());
        if chave.eq_ignore_ascii_case("path") && conteudo.starts_with('/') {
            caminho = Some(conteudo.to_owned());
        } else if chave.eq_ignore_ascii_case("domain") {
            let dominio = conteudo.trim_start_matches('.').to_ascii_lowercase();
            if !dominio.is_empty() && !dominio_contem_o_host(&dominio, host) {
                return Ok(Efeito::Ignorar);
            }
        } else if chave.eq_ignore_ascii_case("max-age") {
            vencido |= conteudo.starts_with('-') || conteudo == "0";
        } else if chave.eq_ignore_ascii_case("expires") {
            vencido |= ano_de_expires(conteudo).is_some_and(|ano| ano < ano_corrente());
        }
    }
    let caminho = caminho.unwrap_or_else(|| caminho_padrao(url));
    let nome = nome.to_owned();
    Ok(if vencido {
        Efeito::Remover { nome, caminho }
    } else {
        Efeito::Guardar(Cookie {
            nome,
            valor: valor.to_owned(),
            caminho,
        })
    })
}

/// `path-match` da RFC 6265, 5.1.4.
fn caminho_casa(cookie: &str, pedido: &str) -> bool {
    pedido == cookie
        || (pedido.starts_with(cookie)
            && (cookie.ends_with('/')
                || pedido
                    .strip_prefix(cookie)
                    .is_some_and(|resto| resto.starts_with('/'))))
}

impl PoteDeCookies {
    pub(crate) fn novo(host: &str) -> Self {
        Self {
            host: host.to_ascii_lowercase(),
            cookies: Vec::new(),
        }
    }

    /// Guarda ou remove os `Set-Cookie` da resposta ao pedido `url`.
    /// Mesmo nome e `Path` substitui no lugar.
    pub(crate) fn absorver(&mut self, url: &str, resposta: &RespostaHttp) -> Result<(), ErroLogin> {
        for set_cookie in resposta.valores("set-cookie") {
            match interpretar(set_cookie, url, &self.host)? {
                Efeito::Ignorar => {}
                Efeito::Remover { nome, caminho } => self
                    .cookies
                    .retain(|c| !(c.nome == nome && c.caminho == caminho)),
                Efeito::Guardar(novo) => {
                    match self
                        .cookies
                        .iter_mut()
                        .find(|c| c.nome == novo.nome && c.caminho == novo.caminho)
                    {
                        Some(existente) => *existente = novo,
                        None => self.cookies.push(novo),
                    }
                }
            }
        }
        Ok(())
    }

    /// `nome=valor; …` dos cookies que o navegador enviaria a `url`: só no
    /// host da instalação, `Path` mais longo primeiro, depois a ordem de
    /// chegada.
    pub(crate) fn cabecalho(&self, url: &str) -> String {
        let no_host = url
            .strip_prefix("https://")
            .map(|resto| {
                resto
                    .split_once(['/', '?'])
                    .map_or(resto, |(antes, _)| antes)
            })
            .is_some_and(|host| host.eq_ignore_ascii_case(&self.host));
        let Some(caminho) = caminho_da_url(url).filter(|_| no_host) else {
            return String::new();
        };
        let mut aplicaveis: Vec<&Cookie> = self
            .cookies
            .iter()
            .filter(|c| caminho_casa(&c.caminho, caminho))
            .collect();
        aplicaveis.sort_by_key(|c| std::cmp::Reverse(c.caminho.len()));
        aplicaveis
            .iter()
            .map(|c| format!("{}={}", c.nome, c.valor))
            .collect::<Vec<_>>()
            .join("; ")
    }
}
