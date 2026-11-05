//! Gravação atômica da sessão (temporário `0600`, `fsync`, `rename`), o
//! marcador de geração e o lock entre processos, só com a `std`.
//! `<url-controle>.lock` e `<url-controle>.geracao` ficam ao lado do
//! arquivo da URL do Controle.
use super::ErroLogin;
use crate::configuracao::{ArquivoPrivado, ArquivosDeAcesso};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Intervalo entre tentativas de obter o lock.
const PASSO_DA_ESPERA: Duration = Duration::from_millis(50);

/// Lock exclusivo do login; liberado quando sai de escopo.
#[derive(Debug)]
pub(crate) struct TravaDoLogin {
    _arquivo: File,
}

fn abrir_privado(opcoes: &mut OpenOptions) -> &mut OpenOptions {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opcoes.mode(0o600);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // O_NOFOLLOW: o último componente não pode ser link simbólico.
        opcoes.custom_flags(0o400_000);
    }
    opcoes
}

impl TravaDoLogin {
    /// Espera até `espera` pelo lock de `<url-controle>.lock` (`flock`
    /// exclusivo da `std`). Nada é pedido ao SEI sem ele.
    pub(crate) fn obter(url_controle: &Path, espera: Duration) -> Result<Self, ErroLogin> {
        let caminho = caminho_ao_lado(url_controle, ".lock");
        let arquivo = abrir_privado(OpenOptions::new().write(true).create(true))
            .open(&caminho)
            .map_err(|_| ErroLogin::ArquivoDeLockDoLoginInacessivel)?;
        let limite = Instant::now() + espera;
        loop {
            match arquivo.try_lock() {
                Ok(()) => return Ok(Self { _arquivo: arquivo }),
                Err(std::fs::TryLockError::WouldBlock) if Instant::now() < limite => {
                    std::thread::sleep(PASSO_DA_ESPERA);
                }
                Err(std::fs::TryLockError::WouldBlock) => {
                    return Err(ErroLogin::LoginEmAndamentoEmOutroProcesso);
                }
                Err(std::fs::TryLockError::Error(_)) => {
                    return Err(ErroLogin::ArquivoDeLockDoLoginInacessivel);
                }
            }
        }
    }
}

/// `<arquivo><sufixo>` no mesmo diretório.
pub(crate) fn caminho_ao_lado(arquivo: &Path, sufixo: &str) -> PathBuf {
    let mut nome = arquivo.as_os_str().to_owned();
    nome.push(sufixo);
    PathBuf::from(nome)
}

/// Geração da sessão: o SHA-256 hexadecimal do cookie sem espaços nas
/// pontas, o mesmo `cookie_sha256` do marcador. O índice local usa-a como
/// chave opaca dos links assinados (ADR 0003); o cookie nunca sai daqui.
pub(crate) fn geracao_da_sessao(cookie: &str) -> String {
    sha256(cookie)
}

/// Geração dos arquivos de sessão como estão agora: os SHA-256 do cookie
/// e da URL do Controle (os do marcador), sem rede. `None` se um deles
/// falta ou está vazio. Comparada antes do relogin de uma leitura, vale
/// também para a sessão preparada fora do `ies login` (sem marcador).
pub(crate) fn geracao_gravada(arquivos: &ArquivosDeAcesso) -> Option<String> {
    let ler = |caminho: &Path| {
        crate::transporte::arquivo_privado(caminho, 8_192)
            .ok()
            .map(|texto| texto.trim().to_owned())
            .filter(|texto| !texto.is_empty())
    };
    let cookie = ler(&arquivos.cookie)?;
    let url = ler(&arquivos.url_controle)?;
    Some(format!("{}:{}", sha256(&cookie), sha256(&url)))
}

fn sha256(texto: &str) -> String {
    crate::hex::hex_minusculo(&Sha256::digest(texto.trim().as_bytes()))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Marcador {
    geracao: u64,
    instalacao: String,
    cookie_sha256: String,
    url_controle_sha256: String,
}

fn ler_marcador(arquivos: &ArquivosDeAcesso) -> Result<Option<Marcador>, ()> {
    let caminho = caminho_ao_lado(&arquivos.url_controle, ".geracao");
    match crate::transporte::arquivo_privado(&caminho, 4096) {
        Ok(texto) => serde_json::from_str(&texto).map(Some).map_err(|_| ()),
        Err(crate::transporte::FalhaTransporte::ArquivoPrivadoAusenteOuInacessivel) => Ok(None),
        Err(_) => Err(()),
    }
}

/// `false` se o marcador existe e não corresponde a estes conteúdos (ou é
/// ilegível); `true` sem marcador (sessão preparada fora do `ies login`).
pub(crate) fn mesma_geracao(arquivos: &ArquivosDeAcesso, url: &str, cookie: &str) -> bool {
    match ler_marcador(arquivos) {
        Ok(None) => true,
        Ok(Some(m)) => m.cookie_sha256 == sha256(cookie) && m.url_controle_sha256 == sha256(url),
        Err(()) => false,
    }
}

/// Destino ausente ou arquivo regular; link simbólico nunca é seguido.
fn conferir_destino(caminho: &Path, papel: ArquivoPrivado) -> Result<(), ErroLogin> {
    match std::fs::symlink_metadata(caminho) {
        Ok(info) if info.file_type().is_symlink() || !info.is_file() => {
            Err(ErroLogin::DestinoDaSessaoNaoEhArquivoRegular(papel))
        }
        Ok(_) => Ok(()),
        Err(erro) if erro.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ErroLogin::GravacaoDaSessaoFalhou(papel)),
    }
}

/// Temporário `0600` no diretório do destino, gravado e sincronizado.
fn preparar(destino: &Path, conteudo: &str, papel: ArquivoPrivado) -> Result<PathBuf, ErroLogin> {
    let falha = || ErroLogin::GravacaoDaSessaoFalhou(papel);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let nome = destino.file_name().ok_or_else(falha)?.to_string_lossy();
    let temporario =
        destino.with_file_name(format!(".{nome}.ies-{}-{nanos}.tmp", std::process::id()));
    let mut arquivo = abrir_privado(OpenOptions::new().write(true).create_new(true))
        .open(&temporario)
        .map_err(|_| falha())?;
    let gravado = arquivo
        .write_all(conteudo.as_bytes())
        .and_then(|()| arquivo.sync_all());
    if gravado.is_err() {
        let _ = std::fs::remove_file(&temporario);
        return Err(falha());
    }
    Ok(temporario)
}

/// Depois de `ies unidade trocar` (#205): a mesma sessão (cookie) passa a
/// abrir pelo Controle da unidade nova. Grava a URL e, se houver marcador,
/// o marcador com o mesmo número de geração, o mesmo cookie e a URL nova,
/// para `ies login` reconhecer a sessão. O cookie não é regravado.
/// Chamado com o lock do login obtido.
pub(crate) fn regravar_url_do_controle(
    arquivos: &ArquivosDeAcesso,
    url: &str,
    cookie: &str,
) -> Result<(), ErroLogin> {
    let geracao_caminho = caminho_ao_lado(&arquivos.url_controle, ".geracao");
    let papel = ArquivoPrivado::UrlControle;
    conferir_destino(&arquivos.url_controle, papel)?;
    conferir_destino(&geracao_caminho, papel)?;
    let marcador = ler_marcador(arquivos)
        .map_err(|()| ErroLogin::GravacaoDaSessaoFalhou(papel))?
        .map(|m| Marcador {
            cookie_sha256: sha256(cookie),
            url_controle_sha256: sha256(url),
            ..m
        });
    let mut destinos = vec![(arquivos.url_controle.clone(), format!("{url}\n"))];
    if let Some(m) = marcador {
        let json =
            serde_json::to_string(&m).map_err(|_| ErroLogin::GravacaoDaSessaoFalhou(papel))?;
        destinos.push((geracao_caminho, format!("{json}\n")));
    }
    let mut temporarios = Vec::new();
    for (destino, conteudo) in &destinos {
        match preparar(destino, conteudo, papel) {
            Ok(t) => temporarios.push(t),
            Err(erro) => {
                for t in &temporarios {
                    let _ = std::fs::remove_file(t);
                }
                return Err(erro);
            }
        }
    }
    for (i, ((destino, _), temporario)) in destinos.iter().zip(&temporarios).enumerate() {
        if std::fs::rename(temporario, destino).is_err() {
            for restante in &temporarios[i..] {
                let _ = std::fs::remove_file(restante);
            }
            return Err(ErroLogin::GravacaoDaSessaoFalhou(papel));
        }
    }
    if let Some(pasta) = arquivos
        .url_controle
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        let _ = File::open(pasta).and_then(|d| d.sync_all());
    }
    Ok(())
}

/// Grava cookie, URL e, por último, o marcador da nova geração. Chamado
/// com o lock do login obtido.
pub(crate) fn gravar(
    arquivos: &ArquivosDeAcesso,
    instalacao: &str,
    url: &str,
    cookie: &str,
) -> Result<u64, ErroLogin> {
    let geracao_caminho = caminho_ao_lado(&arquivos.url_controle, ".geracao");
    let destinos = [
        (&arquivos.cookie, ArquivoPrivado::Cookie),
        (&arquivos.url_controle, ArquivoPrivado::UrlControle),
        (&geracao_caminho, ArquivoPrivado::UrlControle),
    ];
    for (destino, papel) in destinos {
        conferir_destino(destino, papel)?;
    }
    let anterior = ler_marcador(arquivos)
        .ok()
        .flatten()
        .map_or(0, |m| m.geracao);
    let marcador = Marcador {
        geracao: anterior + 1,
        instalacao: instalacao.to_owned(),
        cookie_sha256: sha256(cookie),
        url_controle_sha256: sha256(url),
    };
    let json = serde_json::to_string(&marcador)
        .map_err(|_| ErroLogin::GravacaoDaSessaoFalhou(ArquivoPrivado::UrlControle))?;
    let conteudos = [
        format!("{cookie}\n"),
        format!("{url}\n"),
        format!("{json}\n"),
    ];
    let mut temporarios = Vec::new();
    for ((destino, papel), conteudo) in destinos.iter().zip(&conteudos) {
        match preparar(destino, conteudo, *papel) {
            Ok(temporario) => temporarios.push(temporario),
            Err(erro) => {
                for temporario in &temporarios {
                    let _ = std::fs::remove_file(temporario);
                }
                return Err(erro);
            }
        }
    }
    for (i, ((destino, papel), temporario)) in destinos.iter().zip(&temporarios).enumerate() {
        if std::fs::rename(temporario, destino).is_err() {
            for restante in &temporarios[i..] {
                let _ = std::fs::remove_file(restante);
            }
            return Err(ErroLogin::GravacaoDaSessaoFalhou(*papel));
        }
    }
    if let Some(pasta) = arquivos
        .url_controle
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        // Durabilidade dos `rename`; falha aqui não desfaz a gravação.
        let _ = File::open(pasta).and_then(|d| d.sync_all());
    }
    Ok(marcador.geracao)
}
