//! Pote de cookies do login, limitado ao host da instalação: nome, valor e
//! `Path`. Todo pedido do login é HTTPS, então `Secure` não restringe.
//! Valores nunca aparecem em `Debug` nem em mensagens.
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

impl PoteDeCookies {
    pub(crate) fn novo(host: &str) -> Self {
        Self {
            host: host.to_ascii_lowercase(),
            cookies: Vec::new(),
        }
    }

    /// Guarda ou remove os `Set-Cookie` da resposta ao pedido `url`.
    pub(crate) fn absorver(&mut self, url: &str, resposta: &RespostaHttp) -> Result<(), ErroLogin> {
        let _ = (url, resposta, &self.host);
        todo!("CONTRACT: absorver Set-Cookie")
    }

    /// `nome=valor; …` dos cookies que o navegador enviaria a `url`.
    pub(crate) fn cabecalho(&self, url: &str) -> String {
        let _ = (url, &self.cookies);
        todo!("CONTRACT: cabeçalho Cookie")
    }
}
