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
    if let Some(path) = arquivo_de_configuracao()
        && let Some(arquivos) = ler_configuracao_local(&path)
    {
        return Ok(arquivos);
    }
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

#[derive(serde::Serialize, serde::Deserialize)]
struct ConfiguracaoLocal {
    url_controle_file: PathBuf,
    cookie_file: PathBuf,
    origem_file: PathBuf,
}

fn arquivo_de_configuracao() -> Option<PathBuf> {
    std::env::var_os("IES_CONFIG_FILE").filter(|v| !v.is_empty()).map(PathBuf::from)
}

pub fn salvar_arquivos_de_acesso(
    destino: &std::path::Path,
    arquivos: &ArquivosDeAcesso,
) -> Result<(), &'static str> {
    use std::io::Write;
    let campos = [
        &arquivos.url_controle,
        &arquivos.cookie,
        &arquivos.origem,
    ];
    if campos.iter().any(|p| !p.is_absolute() || !p.is_file()) {
        return Err("caminho_privado_ausente_ou_nao_absoluto");
    }
    #[cfg(unix)]
    for path in campos {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(path).map_err(|_| "arquivo_privado_nao_lido")?.permissions();
        if perms.mode() & 0o077 != 0 {
            return Err("permissoes_de_arquivo_privado_inseguras");
        }
    }
    if !destino.is_absolute() {
        return Err("configuracao_deve_ter_caminho_absoluto");
    }
    let config = ConfiguracaoLocal {
        url_controle_file: arquivos.url_controle.clone(),
        cookie_file: arquivos.cookie.clone(),
        origem_file: arquivos.origem.clone(),
    };
    let data = serde_json::to_vec_pretty(&config).map_err(|_| "configuracao_nao_serializada")?;
    let mut opcoes = std::fs::OpenOptions::new();
    opcoes.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opcoes.mode(0o600);
    }
    let mut file = opcoes.open(destino).map_err(|_| "configuracao_ja_existe_ou_sem_permissao")?;
    file.write_all(&data).map_err(|_| "configuracao_nao_gravada")?;
    Ok(())
}

fn ler_configuracao_local(path: &std::path::Path) -> Option<ArquivosDeAcesso> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > 8192 {
        return None;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return None;
        }
    }
    let raw = std::fs::read(path).ok()?;
    let config: ConfiguracaoLocal = serde_json::from_slice(&raw).ok()?;
    Some(ArquivosDeAcesso {
        url_controle: config.url_controle_file,
        cookie: config.cookie_file,
        origem: config.origem_file,
    })
}
