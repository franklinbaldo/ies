//! Configuração mínima por caminhos privados, sem segredos em argumentos CLI.
use std::path::PathBuf;

pub struct ArquivosDeAcesso {
    pub url_controle: PathBuf,
    pub cookie: PathBuf,
    pub origem: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroConfiguracao {
    UrlDoControleSemCaminho,
    CookieSemCaminhoConfigurado,
    OrigemAutorizadaNaoConfigurada,
}

impl ErroConfiguracao {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::UrlDoControleSemCaminho => "caminho_url_controle_nao_informado",
            Self::CookieSemCaminhoConfigurado => "caminho_cookie_nao_informado",
            Self::OrigemAutorizadaNaoConfigurada => "caminho_origem_nao_informado",
        }
    }
    pub fn mensagem(self) -> &'static str {
        match self {
            Self::UrlDoControleSemCaminho => "defina IES_URL_CONTROLE_FILE",
            Self::CookieSemCaminhoConfigurado => "defina IES_COOKIE_FILE",
            Self::OrigemAutorizadaNaoConfigurada => "defina IES_ORIGEM_FILE",
        }
    }
}

fn caminho(nome: &str, erro: ErroConfiguracao) -> Result<PathBuf, ErroConfiguracao> {
    std::env::var_os(nome)
        .filter(|valor| !valor.is_empty())
        .map(PathBuf::from)
        .ok_or(erro)
}

pub fn arquivos_de_acesso() -> Result<ArquivosDeAcesso, ErroConfiguracao> {
    Ok(ArquivosDeAcesso {
        url_controle: caminho(
            "IES_URL_CONTROLE_FILE",
            ErroConfiguracao::UrlDoControleSemCaminho,
        )?,
        cookie: caminho(
            "IES_COOKIE_FILE",
            ErroConfiguracao::CookieSemCaminhoConfigurado,
        )?,
        origem: caminho(
            "IES_ORIGEM_FILE",
            ErroConfiguracao::OrigemAutorizadaNaoConfigurada,
        )?,
    })
}
