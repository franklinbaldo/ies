//! Configuração mínima por caminhos privados, sem segredos em argumentos CLI.
use std::path::PathBuf;

pub struct ArquivosDeAcesso {
    pub url_controle: PathBuf,
    pub cookie: PathBuf,
    pub origem: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroConfiguracao {
    CaminhoDaUrlDoControleNaoInformado,
    CaminhoDoCookieNaoInformado,
    CaminhoDaOrigemAutorizadaNaoInformado,
}

impl ErroConfiguracao {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::CaminhoDaUrlDoControleNaoInformado => "caminho_url_controle_nao_informado",
            Self::CaminhoDoCookieNaoInformado => "caminho_cookie_nao_informado",
            Self::CaminhoDaOrigemAutorizadaNaoInformado => "caminho_origem_nao_informado",
        }
    }
    pub fn mensagem(self) -> &'static str {
        match self {
            Self::CaminhoDaUrlDoControleNaoInformado => "defina IES_URL_CONTROLE_FILE",
            Self::CaminhoDoCookieNaoInformado => "defina IES_COOKIE_FILE",
            Self::CaminhoDaOrigemAutorizadaNaoInformado => "defina IES_ORIGEM_FILE",
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
            ErroConfiguracao::CaminhoDaUrlDoControleNaoInformado,
        )?,
        cookie: caminho(
            "IES_COOKIE_FILE",
            ErroConfiguracao::CaminhoDoCookieNaoInformado,
        )?,
        origem: caminho(
            "IES_ORIGEM_FILE",
            ErroConfiguracao::CaminhoDaOrigemAutorizadaNaoInformado,
        )?,
    })
}
