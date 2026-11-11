//! Gravação atômica da sessão (temporário `0600`, `fsync`, `rename`, pela
//! gravação privada única `crate::arquivo_privado`), o
//! marcador de geração e o lock entre processos, só com a `std`.
//! `<url-controle>.lock` e `<url-controle>.geracao` ficam ao lado do
//! arquivo da URL do Controle.
use super::ErroLogin;
use crate::arquivo_privado::{FalhaDaGravacao, Parcial, Politica};
use crate::configuracao::{ArquivoPrivado, ArquivosDeAcesso};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
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
            .map_err(|erro| ErroLogin::ArquivoDeLockDoLoginInacessivel(erro.kind()))?;
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
                Err(std::fs::TryLockError::Error(erro)) => {
                    return Err(ErroLogin::ArquivoDeLockDoLoginInacessivel(erro.kind()));
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

/// Exceção inventariada (#183): o marcador de geração não é um arquivo de
/// sessão de transporte. O login o lê diretamente para comparar pares URL/
/// cookie e decidir reaproveitamento ou relogin; não é um GET da aplicação.
///
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
    hex::encode(Sha256::digest(texto.trim().as_bytes()))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Marcador {
    geracao: u64,
    instalacao: String,
    cookie_sha256: String,
    url_controle_sha256: String,
}

// Exceção inventariada (#183): leitura local do marcador de consistência
// próprio da escrita atômica. O formato é só deste módulo; a leitura
// coerente (`ReferenciasDaSessao`, #222) o confere por `marcador_confere`.
fn ler_marcador(arquivos: &ArquivosDeAcesso) -> Result<Option<Marcador>, ErroMarcadorDaSessao> {
    let caminho = caminho_ao_lado(&arquivos.url_controle, ".geracao");
    match crate::transporte::arquivo_privado(&caminho, 4096) {
        Ok(texto) => serde_json::from_str(&texto).map(Some).map_err(|erro| {
            ErroMarcadorDaSessao::ForaDoFormato {
                linha: erro.line(),
                coluna: erro.column(),
            }
        }),
        Err(crate::transporte::FalhaTransporte::ArquivoPrivadoAusenteOuInacessivel(_)) => Ok(None),
        Err(falha) => Err(ErroMarcadorDaSessao::Ilegivel(falha)),
    }
}

/// O marcador `.geracao` existe e não foi lido. O conteúdo (hashes) não
/// entra no erro; do JSON fica só a posição.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroMarcadorDaSessao {
    Ilegivel(#[source] crate::transporte::FalhaTransporte),
    ForaDoFormato { linha: usize, coluna: usize },
}

/// O texto de um marcador confere com a URL e o cookie (os SHA-256 dele).
/// JSON inválido não confere. Usado pela leitura coerente (#222).
pub(crate) fn marcador_confere(marcador: &str, url: &str, cookie: &str) -> bool {
    serde_json::from_str::<Marcador>(marcador)
        .is_ok_and(|m| m.cookie_sha256 == sha256(cookie) && m.url_controle_sha256 == sha256(url))
}

/// `false` se o marcador existe e não corresponde a estes conteúdos (ou é
/// ilegível); `true` sem marcador (sessão preparada fora do `ies login`).
pub(crate) fn mesma_geracao(arquivos: &ArquivosDeAcesso, url: &str, cookie: &str) -> bool {
    match ler_marcador(arquivos) {
        Ok(None) => true,
        Ok(Some(m)) => m.cookie_sha256 == sha256(cookie) && m.url_controle_sha256 == sha256(url),
        Err(_ilegivel) => false,
    }
}

/// A falha da gravação privada única nos erros da sessão: link no
/// nome final é destino não regular; ancestral link é aceito.
fn traduzir(falha: FalhaDaGravacao, papel: ArquivoPrivado) -> ErroLogin {
    match falha {
        FalhaDaGravacao::DestinoNaoEhArquivoRegular => {
            ErroLogin::DestinoDaSessaoNaoEhArquivoRegular(papel)
        }
        #[cfg(unix)]
        FalhaDaGravacao::DiretorioGravavelPorOutros => {
            ErroLogin::DiretorioDaSessaoGravavelPorOutros(papel)
        }
        FalhaDaGravacao::SemNomeDeArquivo
        | FalhaDaGravacao::DiretorioAusente
        | FalhaDaGravacao::DiretorioNaoConsultado(_)
        | FalhaDaGravacao::DestinoJaExiste
        | FalhaDaGravacao::AcimaDoLimite
        | FalhaDaGravacao::CriacaoFalhou(_)
        | FalhaDaGravacao::EscritaFalhou
        | FalhaDaGravacao::ErroDeIoNaEscrita(_)
        | FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio
        | FalhaDaGravacao::NivelDoCaminhoIlegivel(_)
        | FalhaDaGravacao::CriacaoDoDiretorioFalhou(_)
        | FalhaDaGravacao::CriacaoDoTemporarioFalhou(_)
        | FalhaDaGravacao::NomesDeTemporarioEsgotados
        | FalhaDaGravacao::ConfirmacaoFalhou(_) => ErroLogin::GravacaoDaSessaoFalhou(papel),
    }
}

/// Temporário `0600` no diretório do destino, gravado e sincronizado
/// (`crate::arquivo_privado`, substituição atómica); confere antes que o
/// destino está ausente ou é arquivo regular e que o diretório real não é
/// gravável por outros.
fn preparar(destino: &Path, conteudo: &str, papel: ArquivoPrivado) -> Result<Parcial, ErroLogin> {
    let traduzir = |falha| traduzir(falha, papel);
    let mut parcial =
        Parcial::abrir(destino, Politica::substituicao().diretorio_do_ies()).map_err(traduzir)?;
    parcial.escrever(conteudo.as_bytes()).map_err(traduzir)?;
    parcial.sincronizar().map_err(traduzir)?;
    Ok(parcial)
}

/// Publica os parciais em ordem; o primeiro que falha descarta os
/// restantes (no `Drop`), sem publicar nenhum deles.
fn publicar(parciais: Vec<(Parcial, ArquivoPrivado)>) -> Result<(), ErroLogin> {
    for (parcial, papel) in parciais {
        parcial
            .confirmar()
            .map_err(|falha| traduzir(falha, papel))?;
    }
    Ok(())
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
    let marcador = ler_marcador(arquivos)
        .map_err(ErroLogin::MarcadorDaSessaoIlegivel)?
        .map(|m| Marcador {
            cookie_sha256: sha256(cookie),
            url_controle_sha256: sha256(url),
            ..m
        });
    let mut parciais = vec![(
        preparar(&arquivos.url_controle, &format!("{url}\n"), papel)?,
        papel,
    )];
    if let Some(m) = marcador {
        let json = serde_json::to_string(&m)
            .map_err(|erro| ErroLogin::SerializacaoDaSessaoFalhou(papel, erro.classify()))?;
        parciais.push((
            preparar(&geracao_caminho, &format!("{json}\n"), papel)?,
            papel,
        ));
    }
    publicar(parciais)
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
    let json = serde_json::to_string(&marcador).map_err(|erro| {
        ErroLogin::SerializacaoDaSessaoFalhou(ArquivoPrivado::UrlControle, erro.classify())
    })?;
    let destinos = [
        (
            &arquivos.cookie,
            ArquivoPrivado::Cookie,
            format!("{cookie}\n"),
        ),
        (
            &arquivos.url_controle,
            ArquivoPrivado::UrlControle,
            format!("{url}\n"),
        ),
        (
            &geracao_caminho,
            ArquivoPrivado::UrlControle,
            format!("{json}\n"),
        ),
    ];
    // Todos preparados (destinos conferidos) antes do primeiro `rename`.
    let mut parciais = Vec::new();
    for (destino, papel, conteudo) in &destinos {
        parciais.push((preparar(destino, conteudo, *papel)?, *papel));
    }
    publicar(parciais)?;
    Ok(marcador.geracao)
}
