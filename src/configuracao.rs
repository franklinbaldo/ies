//! Configuração mínima por caminhos privados, sem segredos em argumentos CLI.
use std::path::PathBuf;

#[derive(Clone)]
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
    #[cfg(unix)]
    ArquivoConfiguracaoPermissoesAbertas,
    ArquivoConfiguracaoLinkSimbolico,
    ArquivoConfiguracaoJsonInvalido,
    ArquivoConfiguracaoCampoInvalido,
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
    /// O sistema não consultou o arquivo privado (o motivo do sistema).
    #[cfg(unix)]
    ArquivoPrivadoNaoLido(std::io::ErrorKind),
    #[cfg(unix)]
    PermissoesDeArquivoPrivadoInseguras,
    ConfiguracaoDeveTerCaminhoAbsoluto,
    /// A serialização falhou (a categoria do erro de `serde_json`).
    ConfiguracaoNaoSerializada(serde_json::error::Category),
    ConfiguracaoNaoGravada,
    ConfiguracaoJaExisteOuSemPermissao,
}

pub fn salvar_arquivos_de_acesso(
    destino: &std::path::Path,
    arquivos: &ArquivosDeAcesso,
) -> Result<(), ErroGravacaoDaConfiguracao> {
    use crate::arquivo_privado::{self, FalhaDaGravacao, Politica};
    let campos = [&arquivos.url_controle, &arquivos.cookie, &arquivos.origem];
    if campos.iter().any(|p| !p.is_absolute() || !p.is_file()) {
        return Err(ErroGravacaoDaConfiguracao::CaminhoPrivadoAusenteOuNaoAbsoluto);
    }
    #[cfg(unix)]
    for path in campos {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(path)
            .map_err(|erro| ErroGravacaoDaConfiguracao::ArquivoPrivadoNaoLido(erro.kind()))?
            .permissions();
        if perms.mode() & 0o077 != 0 {
            return Err(ErroGravacaoDaConfiguracao::PermissoesDeArquivoPrivadoInseguras);
        }
    }
    if !destino.is_absolute() {
        return Err(ErroGravacaoDaConfiguracao::ConfiguracaoDeveTerCaminhoAbsoluto);
    }
    let config = ConfiguracaoLocal {
        url_controle: arquivos.url_controle.clone(),
        cookie: arquivos.cookie.clone(),
        origem: arquivos.origem.clone(),
    };
    let data = serde_json::to_vec_pretty(&config)
        .map_err(|erro| ErroGravacaoDaConfiguracao::ConfiguracaoNaoSerializada(erro.classify()))?;
    // Gravação privada única (#211): arquivo novo 0600, sem link no
    // destino, em diretório fechado a outros, publicado inteiro ou não.
    arquivo_privado::gravar(destino, &data, Politica::exclusiva().diretorio_do_ies()).map_err(
        |falha| match falha {
            FalhaDaGravacao::EscritaFalhou
            | FalhaDaGravacao::ErroDeIoNaEscrita(_)
            | FalhaDaGravacao::ConfirmacaoFalhou(_)
            | FalhaDaGravacao::AcimaDoLimite => ErroGravacaoDaConfiguracao::ConfiguracaoNaoGravada,
            #[cfg(unix)]
            FalhaDaGravacao::DiretorioGravavelPorOutros => {
                ErroGravacaoDaConfiguracao::ConfiguracaoJaExisteOuSemPermissao
            }
            FalhaDaGravacao::SemNomeDeArquivo
            | FalhaDaGravacao::DiretorioAusente
            | FalhaDaGravacao::DiretorioNaoConsultado(_)
            | FalhaDaGravacao::DestinoJaExiste
            | FalhaDaGravacao::DestinoNaoEhArquivoRegular
            | FalhaDaGravacao::CriacaoFalhou(_)
            | FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio
            | FalhaDaGravacao::NivelDoCaminhoIlegivel(_)
            | FalhaDaGravacao::CriacaoDoDiretorioFalhou(_)
            | FalhaDaGravacao::CriacaoDoTemporarioFalhou(_)
            | FalhaDaGravacao::NomesDeTemporarioEsgotados => {
                ErroGravacaoDaConfiguracao::ConfiguracaoJaExisteOuSemPermissao
            }
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
    #[cfg(unix)]
    PermissoesAbertas,
    /// Só Unix: o caminho passou a apontar para outro arquivo (inode ou
    /// dispositivo) entre a abertura e a conferência.
    #[cfg(unix)]
    ArquivoTrocadoDuranteALeitura,
    /// Erro do sistema ao abrir, consultar ou ler o arquivo (o motivo).
    ErroDeIoNaLeitura(std::io::ErrorKind),
    JsonInvalido(serde_json::error::Category),
    CampoInvalido,
}

impl From<FalhaConfiguracaoLocal> for ErroConfiguracao {
    fn from(causa: FalhaConfiguracaoLocal) -> Self {
        match causa {
            FalhaConfiguracaoLocal::LinkSimbolico => Self::ArquivoConfiguracaoLinkSimbolico,
            #[cfg(unix)]
            FalhaConfiguracaoLocal::PermissoesAbertas => Self::ArquivoConfiguracaoPermissoesAbertas,
            FalhaConfiguracaoLocal::JsonInvalido(_) => Self::ArquivoConfiguracaoJsonInvalido,
            FalhaConfiguracaoLocal::CampoInvalido => Self::ArquivoConfiguracaoCampoInvalido,
            FalhaConfiguracaoLocal::Ausente
            | FalhaConfiguracaoLocal::MetadadosNaoLidos
            | FalhaConfiguracaoLocal::NaoRegular
            | FalhaConfiguracaoLocal::AcimaDoLimite
            | FalhaConfiguracaoLocal::ErroDeIoNaLeitura(_) => {
                Self::ArquivoDeConfiguracaoNaoPodeSerLido
            }
            #[cfg(unix)]
            FalhaConfiguracaoLocal::ArquivoTrocadoDuranteALeitura => {
                Self::ArquivoDeConfiguracaoNaoPodeSerLido
            }
        }
    }
}

fn ler_configuracao_local(
    path: &std::path::Path,
) -> Result<ArquivosDeAcesso, FalhaConfiguracaoLocal> {
    use std::io::Read as _;

    let info = std::fs::symlink_metadata(path).map_err(|erro| {
        if erro.kind() == std::io::ErrorKind::NotFound {
            FalhaConfiguracaoLocal::Ausente
        } else {
            FalhaConfiguracaoLocal::MetadadosNaoLidos
        }
    })?;
    if info.file_type().is_symlink() {
        return Err(FalhaConfiguracaoLocal::LinkSimbolico);
    }
    if !info.is_file() {
        return Err(FalhaConfiguracaoLocal::NaoRegular);
    }
    if info.len() > 8_192 {
        return Err(FalhaConfiguracaoLocal::AcimaDoLimite);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if info.permissions().mode() & 0o077 != 0 {
            return Err(FalhaConfiguracaoLocal::PermissoesAbertas);
        }
    }
    // Validar também o descritor aberto e reler a entrada: não aceitar
    // troca de arquivo/symlink entre a inspeção do caminho e a abertura.
    let arquivo = std::fs::File::open(path)
        .map_err(|erro| FalhaConfiguracaoLocal::ErroDeIoNaLeitura(erro.kind()))?;
    let aberto = arquivo
        .metadata()
        .map_err(|erro| FalhaConfiguracaoLocal::ErroDeIoNaLeitura(erro.kind()))?;
    let no_caminho = std::fs::symlink_metadata(path)
        .map_err(|erro| FalhaConfiguracaoLocal::ErroDeIoNaLeitura(erro.kind()))?;
    if no_caminho.file_type().is_symlink() {
        return Err(FalhaConfiguracaoLocal::LinkSimbolico);
    }
    if !aberto.is_file() || !no_caminho.is_file() {
        return Err(FalhaConfiguracaoLocal::NaoRegular);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
        if info.ino() != aberto.ino()
            || info.dev() != aberto.dev()
            || no_caminho.ino() != aberto.ino()
            || no_caminho.dev() != aberto.dev()
        {
            return Err(FalhaConfiguracaoLocal::ArquivoTrocadoDuranteALeitura);
        }
        if aberto.permissions().mode() & 0o077 != 0 {
            return Err(FalhaConfiguracaoLocal::PermissoesAbertas);
        }
    }
    let mut raw = Vec::new();
    arquivo
        .take(8_193)
        .read_to_end(&mut raw)
        .map_err(|erro| FalhaConfiguracaoLocal::ErroDeIoNaLeitura(erro.kind()))?;
    if raw.len() > 8_192 {
        return Err(FalhaConfiguracaoLocal::AcimaDoLimite);
    }
    let json: serde_json::Value = serde_json::from_slice(&raw)
        .map_err(|erro| FalhaConfiguracaoLocal::JsonInvalido(erro.classify()))?;
    let objeto = json
        .as_object()
        .ok_or(FalhaConfiguracaoLocal::CampoInvalido)?;
    let campo = |nome| -> Result<PathBuf, FalhaConfiguracaoLocal> {
        let texto = objeto
            .get(nome)
            .and_then(serde_json::Value::as_str)
            .ok_or(FalhaConfiguracaoLocal::CampoInvalido)?;
        let path = PathBuf::from(texto);
        if !path.is_absolute() {
            return Err(FalhaConfiguracaoLocal::CampoInvalido);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroArquivoPrivado {
    CaminhoNaoAbsoluto(ArquivoPrivado),
    Ausente(ArquivoPrivado),
    LinkSimbolico(ArquivoPrivado),
    NaoRegular(ArquivoPrivado),
    Vazio(ArquivoPrivado),
    #[cfg(unix)]
    PermissoesInseguras(ArquivoPrivado),
    MetadadosNaoLidos(ArquivoPrivado),
}

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
