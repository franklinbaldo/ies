//! `ies login`: autentica na interface Web da instalação configurada e grava
//! a sessão nos arquivos privados que os outros comandos leem.
//! Contrato: contracts/cli/sessao-login.md. O Todos é oráculo de
//! comportamento (session.py), não arquitetura portada.
//!
//! Só este fluxo segue redirecionamentos, por um interpretador explícito e
//! limitado; os GET/POST dos demais comandos continuam recusando 3xx.
mod cookies;
mod credenciais;
pub(crate) mod entrada_do_login;
mod erro;
mod fluxo;
mod gravacao;
pub(crate) mod http;
pub(crate) mod redirecionamento;
pub(crate) mod relogin;
pub(crate) mod tela;

pub(crate) use credenciais::Credenciais;
pub(crate) use erro::{CampoDasCredenciais, ErroLogin};
pub(crate) use fluxo::entrar_colhendo;
#[cfg(test)]
pub(crate) use fluxo::entrar_via;
#[cfg(test)]
pub(crate) use gravacao::mesma_geracao;
pub(crate) use gravacao::{geracao_da_sessao, marcador_confere};
pub(crate) use gravacao::{regravar_url_do_controle, travar_o_login};

use crate::configuracao::ArquivosDeAcesso;
use crate::formulario_web::SubmissaoDerivada;
use crate::transporte::FalhaTransporte;
use std::fmt;
use std::path::Path;
use std::time::Duration;

/// Saltos de redirecionamento aceitos por cadeia (antes e depois do POST).
pub(crate) const MAX_REDIRECIONAMENTOS: usize = 8;

/// Resposta HTTP do login: status, **todos** os cabeçalhos (vários
/// `Set-Cookie`) e o corpo já limitado (vazio em 3xx, nunca lido).
pub(crate) struct RespostaHttp {
    pub(crate) status: u16,
    pub(crate) cabecalhos: Vec<(String, String)>,
    pub(crate) corpo: Vec<u8>,
}

impl RespostaHttp {
    /// Primeiro valor do cabeçalho, nome sem distinção de caixa.
    pub(crate) fn cabecalho(&self, nome: &str) -> Option<&str> {
        self.cabecalhos
            .iter()
            .find(|(chave, _)| chave.eq_ignore_ascii_case(nome))
            .map(|(_, valor)| valor.as_str())
    }

    pub(crate) fn valores<'a>(&'a self, nome: &'a str) -> impl Iterator<Item = &'a str> {
        self.cabecalhos
            .iter()
            .filter(move |(chave, _)| chave.eq_ignore_ascii_case(nome))
            .map(|(_, valor)| valor.as_str())
    }
}

/// Cabeçalhos e corpo podem trazer cookie e URL assinada: nunca no `Debug`.
impl fmt::Debug for RespostaHttp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RespostaHttp")
            .field("status", &self.status)
            .field("cabecalhos", &self.cabecalhos.len())
            .field("corpo", &self.corpo.len())
            .finish()
    }
}

/// Transporte injetável do login: uma chamada é um pedido, sem retry nem
/// redirecionamento oculto. O POST consome a submissão derivada.
pub(crate) trait TransporteDeLogin {
    fn get(&mut self, url: &str, cookie: &str) -> Result<RespostaHttp, FalhaTransporte>;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaHttp, FalhaTransporte>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Conexao {
    LoginRealizado,
    SessaoExistenteValida,
}

impl Conexao {
    pub(crate) fn rotulo(self) -> &'static str {
        match self {
            Self::LoginRealizado => "login_realizado",
            Self::SessaoExistenteValida => "sessao_existente_valida",
        }
    }
}

/// O que a saída publica: nada de credencial, cookie ou URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResultadoDoLogin {
    pub(crate) instalacao: String,
    pub(crate) unidade_exibida: String,
    pub(crate) conexao: Conexao,
    pub(crate) requisicoes: usize,
}

/// Entrada de `entrar_via`: caminhos, opções e a espera máxima do lock.
pub(crate) struct PedidoDeLogin<'a> {
    pub(crate) arquivos: &'a ArquivosDeAcesso,
    pub(crate) credenciais: Option<&'a Path>,
    pub(crate) forcar: bool,
    pub(crate) unidade_esperada: Option<&'a str>,
    pub(crate) espera_do_lock: Duration,
}

#[cfg(test)]
#[path = "sessao_login/tests_apoio.rs"]
mod tests_apoio;

#[cfg(test)]
#[path = "sessao_login/tests_fluxo.rs"]
mod tests_fluxo;

#[cfg(test)]
#[path = "sessao_login/tests_recusas.rs"]
mod tests_recusas;

#[cfg(test)]
#[path = "sessao_login/tests_sessao.rs"]
mod tests_sessao;

#[cfg(test)]
#[path = "sessao_login/tests_snapshot.rs"]
mod tests_snapshot;

#[cfg(test)]
#[path = "sessao_login/tests_colheita.rs"]
mod tests_colheita;

#[cfg(test)]
#[path = "sessao_login/tests_relogin.rs"]
mod tests_relogin;

#[cfg(test)]
#[path = "sessao_login/tests_entrada_do_login.rs"]
mod tests_entrada_do_login;
