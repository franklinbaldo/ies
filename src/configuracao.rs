//! Configuração mínima por caminhos privados, sem segredos em argumentos CLI.
use std::path::PathBuf;

pub struct ArquivosDeAcesso {
    pub url_controle: PathBuf,
    pub cookie: PathBuf,
    pub origem: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroConfiguracao {
    UrlDoControleSemCaminho,
    CookieSemCaminhoConfigurado,
    OrigemAutorizadaNaoConfigurada,
    ArquivoDeConfiguracaoNaoPodeSerLido,
    ArquivoConfiguracaoPermissoesAbertas,
    ArquivoConfiguracaoLinkSimbolico,
    ArquivoConfiguracaoJsonInvalido,
    ArquivoConfiguracaoCampoInvalido,
}

impl ErroConfiguracao {
    pub fn mensagem(self) -> &'static str {
        match self {
            Self::UrlDoControleSemCaminho => "defina IES_URL_CONTROLE_FILE",
            Self::CookieSemCaminhoConfigurado => "defina IES_COOKIE_FILE",
            Self::OrigemAutorizadaNaoConfigurada => "defina IES_ORIGEM_FILE",
            Self::ArquivoDeConfiguracaoNaoPodeSerLido => {
                "arquivo IES_CONFIG_FILE ausente, não regular, excessivo ou ilegível"
            }
            Self::ArquivoConfiguracaoPermissoesAbertas => {
                "IES_CONFIG_FILE tem permissões inseguras"
            }
            Self::ArquivoConfiguracaoLinkSimbolico => "IES_CONFIG_FILE é link simbólico",
            Self::ArquivoConfiguracaoJsonInvalido => "IES_CONFIG_FILE não contém JSON válido",
            Self::ArquivoConfiguracaoCampoInvalido => {
                "IES_CONFIG_FILE contém campo de caminho inválido"
            }
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
    if let Some(path) = arquivo_de_configuracao() {
        return ler_configuracao_local(&path).map_err(ErroConfiguracao::from);
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
    url_controle: PathBuf,
    cookie: PathBuf,
    origem: PathBuf,
}

fn arquivo_de_configuracao() -> Option<PathBuf> {
    std::env::var_os("IES_CONFIG_FILE")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Por que `ies setup` não gravou a configuração (nenhum caminho privado
/// ou conteúdo vai no erro).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroGravacaoDaConfiguracao {
    CaminhoPrivadoAusenteOuNaoAbsoluto,
    ArquivoPrivadoNaoLido,
    PermissoesDeArquivoPrivadoInseguras,
    ConfiguracaoDeveTerCaminhoAbsoluto,
    ConfiguracaoNaoSerializada,
    ConfiguracaoNaoGravada,
    ConfiguracaoJaExisteOuSemPermissao,
}

pub fn salvar_arquivos_de_acesso(
    destino: &std::path::Path,
    arquivos: &ArquivosDeAcesso,
) -> Result<(), ErroGravacaoDaConfiguracao> {
    use crate::arquivo_privado::{self, FalhaDaGravacao as F, Politica};
    use ErroGravacaoDaConfiguracao as E;
    let campos = [&arquivos.url_controle, &arquivos.cookie, &arquivos.origem];
    if campos.iter().any(|p| !p.is_absolute() || !p.is_file()) {
        return Err(E::CaminhoPrivadoAusenteOuNaoAbsoluto);
    }
    #[cfg(unix)]
    for path in campos {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(path)
            .map_err(|_| E::ArquivoPrivadoNaoLido)?
            .permissions();
        if perms.mode() & 0o077 != 0 {
            return Err(E::PermissoesDeArquivoPrivadoInseguras);
        }
    }
    if !destino.is_absolute() {
        return Err(E::ConfiguracaoDeveTerCaminhoAbsoluto);
    }
    let config = ConfiguracaoLocal {
        url_controle: arquivos.url_controle.clone(),
        cookie: arquivos.cookie.clone(),
        origem: arquivos.origem.clone(),
    };
    let data = serde_json::to_vec_pretty(&config).map_err(|_| E::ConfiguracaoNaoSerializada)?;
    // Gravação privada única (#211): arquivo novo 0600, sem link no
    // destino, em diretório fechado a outros, publicado inteiro ou não.
    arquivo_privado::gravar(destino, &data, Politica::exclusiva().diretorio_do_ies()).map_err(
        |falha| match falha {
            F::EscritaFalhou | F::ConfirmacaoFalhou | F::AcimaDoLimite => E::ConfiguracaoNaoGravada,
            F::SemNomeDeArquivo
            | F::DiretorioAusente
            | F::DiretorioGravavelPorOutros
            | F::DestinoJaExiste
            | F::DestinoNaoEhArquivoRegular
            | F::CriacaoFalhou => E::ConfiguracaoJaExisteOuSemPermissao,
        },
    )
}

/// Causas internas sem caminho ou conteúdo privado associado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FalhaConfiguracaoLocal {
    Ausente,
    MetadadosNaoLidos,
    LinkSimbolico,
    NaoRegular,
    AcimaDoLimite,
    // Só construída na conferência de permissões Unix.
    #[cfg_attr(not(unix), allow(dead_code))]
    PermissoesAbertas,
    LeituraFalhou,
    JsonInvalido,
    CampoInvalido,
}

impl From<FalhaConfiguracaoLocal> for ErroConfiguracao {
    fn from(causa: FalhaConfiguracaoLocal) -> Self {
        match causa {
            FalhaConfiguracaoLocal::LinkSimbolico => Self::ArquivoConfiguracaoLinkSimbolico,
            FalhaConfiguracaoLocal::PermissoesAbertas => Self::ArquivoConfiguracaoPermissoesAbertas,
            FalhaConfiguracaoLocal::JsonInvalido => Self::ArquivoConfiguracaoJsonInvalido,
            FalhaConfiguracaoLocal::CampoInvalido => Self::ArquivoConfiguracaoCampoInvalido,
            FalhaConfiguracaoLocal::Ausente
            | FalhaConfiguracaoLocal::MetadadosNaoLidos
            | FalhaConfiguracaoLocal::NaoRegular
            | FalhaConfiguracaoLocal::AcimaDoLimite
            | FalhaConfiguracaoLocal::LeituraFalhou => Self::ArquivoDeConfiguracaoNaoPodeSerLido,
        }
    }
}

fn ler_configuracao_local(
    path: &std::path::Path,
) -> Result<ArquivosDeAcesso, FalhaConfiguracaoLocal> {
    use FalhaConfiguracaoLocal as F;
    use std::io::Read as _;

    let info = std::fs::symlink_metadata(path).map_err(|erro| {
        if erro.kind() == std::io::ErrorKind::NotFound {
            F::Ausente
        } else {
            F::MetadadosNaoLidos
        }
    })?;
    if info.file_type().is_symlink() {
        return Err(F::LinkSimbolico);
    }
    if !info.is_file() {
        return Err(F::NaoRegular);
    }
    if info.len() > 8_192 {
        return Err(F::AcimaDoLimite);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if info.permissions().mode() & 0o077 != 0 {
            return Err(F::PermissoesAbertas);
        }
    }
    // Validar também o descritor aberto e reler a entrada: não aceitar
    // troca de arquivo/symlink entre a inspeção do caminho e a abertura.
    let arquivo = std::fs::File::open(path).map_err(|_| F::LeituraFalhou)?;
    let aberto = arquivo.metadata().map_err(|_| F::LeituraFalhou)?;
    let no_caminho = std::fs::symlink_metadata(path).map_err(|_| F::LeituraFalhou)?;
    if no_caminho.file_type().is_symlink() {
        return Err(F::LinkSimbolico);
    }
    if !aberto.is_file() || !no_caminho.is_file() {
        return Err(F::NaoRegular);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
        if info.ino() != aberto.ino()
            || info.dev() != aberto.dev()
            || no_caminho.ino() != aberto.ino()
            || no_caminho.dev() != aberto.dev()
        {
            return Err(F::LeituraFalhou);
        }
        if aberto.permissions().mode() & 0o077 != 0 {
            return Err(F::PermissoesAbertas);
        }
    }
    let mut raw = Vec::new();
    arquivo
        .take(8_193)
        .read_to_end(&mut raw)
        .map_err(|_| F::LeituraFalhou)?;
    if raw.len() > 8_192 {
        return Err(F::AcimaDoLimite);
    }
    let json: serde_json::Value = serde_json::from_slice(&raw).map_err(|_| F::JsonInvalido)?;
    let objeto = json.as_object().ok_or(F::CampoInvalido)?;
    let campo = |nome| -> Result<PathBuf, F> {
        let texto = objeto
            .get(nome)
            .and_then(serde_json::Value::as_str)
            .ok_or(F::CampoInvalido)?;
        let path = PathBuf::from(texto);
        if !path.is_absolute() {
            return Err(F::CampoInvalido);
        }
        Ok(path)
    };
    Ok(ArquivosDeAcesso {
        url_controle: campo("url_controle")?,
        cookie: campo("cookie")?,
        origem: campo("origem")?,
    })
}

#[cfg(test)]
#[path = "configuracao/tests.rs"]
mod tests;

/// Papel dos caminhos de acesso; não contém conteúdo nem nome do arquivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArquivoPrivado {
    UrlControle,
    Cookie,
    Origem,
}
impl ArquivoPrivado {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroArquivoPrivado {
    CaminhoNaoAbsoluto(ArquivoPrivado),
    Ausente(ArquivoPrivado),
    LinkSimbolico(ArquivoPrivado),
    NaoRegular(ArquivoPrivado),
    Vazio(ArquivoPrivado),
    #[cfg_attr(not(unix), allow(dead_code))]
    PermissoesInseguras(ArquivoPrivado),
    MetadadosNaoLidos(ArquivoPrivado),
}
impl ErroArquivoPrivado {}

/// Consulta apenas metadados: não lê conteúdo, credenciais nem acessa o SEI.
pub fn verificar_arquivos_privados(arquivos: &ArquivosDeAcesso) -> Result<(), ErroArquivoPrivado> {
    for (tipo, caminho) in [
        (ArquivoPrivado::UrlControle, arquivos.url_controle.as_path()),
        (ArquivoPrivado::Cookie, arquivos.cookie.as_path()),
        (ArquivoPrivado::Origem, arquivos.origem.as_path()),
    ] {
        if !caminho.is_absolute() {
            return Err(ErroArquivoPrivado::CaminhoNaoAbsoluto(tipo));
        }
        let info = std::fs::symlink_metadata(caminho).map_err(|erro| {
            if erro.kind() == std::io::ErrorKind::NotFound {
                ErroArquivoPrivado::Ausente(tipo)
            } else {
                ErroArquivoPrivado::MetadadosNaoLidos(tipo)
            }
        })?;
        if info.file_type().is_symlink() {
            return Err(ErroArquivoPrivado::LinkSimbolico(tipo));
        }
        if !info.is_file() {
            return Err(ErroArquivoPrivado::NaoRegular(tipo));
        }
        if info.len() == 0 {
            return Err(ErroArquivoPrivado::Vazio(tipo));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if info.permissions().mode() & 0o077 != 0 {
                return Err(ErroArquivoPrivado::PermissoesInseguras(tipo));
            }
        }
    }
    Ok(())
}
