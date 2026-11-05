//! Gravação privada única (#211): o par de escrita de
//! `transporte::arquivo_privado`, que é a leitura. Cria (exclusivo) ou
//! substitui (atômico) um arquivo `0600` sem seguir link simbólico no
//! destino nem em ancestral, por um temporário `.NOME.parcial-PID-N` no
//! mesmo diretório, `sync_all` e `hard_link`/`rename`; o parcial nunca
//! fica para trás numa falha. Política e limites:
//! docs/decisoes/arquivos-privados/gravacao-privada.md.
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

/// Como o destino recebe o conteúdo, declarado pelo chamador.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Modo {
    /// Só arquivo novo: nada com esse nome pode existir.
    Exclusivo,
    /// Substituição atômica (temporário + `rename`) de arquivo regular.
    Substituir,
}

/// Política da gravação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Politica {
    modo: Modo,
    limite: Option<u64>,
    criar_diretorios: bool,
}

impl Politica {
    pub(crate) const fn exclusiva() -> Self {
        Self {
            modo: Modo::Exclusivo,
            limite: None,
            criar_diretorios: false,
        }
    }

    pub(crate) const fn substituicao() -> Self {
        Self {
            modo: Modo::Substituir,
            limite: None,
            criar_diretorios: false,
        }
    }

    pub(crate) const fn com_limite(self, bytes: u64) -> Self {
        Self {
            limite: Some(bytes),
            ..self
        }
    }

    pub(crate) const fn criando_diretorios(self) -> Self {
        Self {
            criar_diretorios: true,
            ..self
        }
    }
}

/// Falha tipada, sem caminho nem conteúdo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FalhaDaGravacao {
    SemNomeDeArquivo,
    DiretorioAusente,
    DiretorioLinkSimbolico,
    DestinoJaExiste,
    DestinoNaoEhArquivoRegular,
    AcimaDoLimite,
    CriacaoFalhou,
    EscritaFalhou,
    ConfirmacaoFalhou,
}

/// Cada ancestral do destino (da raiz ao pai) é diretório e não é link
/// simbólico. Com `criar`, os ausentes nascem `0700`, um nível por vez.
/// Conferência por caminho: não impede a troca concorrente de um
/// diretório por um link (a decisão documenta esse limite).
pub(crate) fn conferir_diretorios(destino: &Path, criar: bool) -> Result<(), FalhaDaGravacao> {
    let absoluto = absoluto(destino)?;
    let pai = absoluto.parent().ok_or(FalhaDaGravacao::DiretorioAusente)?;
    let mut niveis: Vec<&Path> = pai.ancestors().collect();
    niveis.reverse();
    for nivel in niveis {
        match fs::symlink_metadata(nivel) {
            Ok(info) if info.file_type().is_symlink() => {
                return Err(FalhaDaGravacao::DiretorioLinkSimbolico);
            }
            Ok(info) if info.is_dir() => {}
            Err(erro) if erro.kind() == ErrorKind::NotFound && criar => criar_nivel(nivel)?,
            Ok(_) | Err(_) => return Err(FalhaDaGravacao::DiretorioAusente),
        }
    }
    Ok(())
}

/// O destino como caminho absoluto, sem resolver links.
fn absoluto(destino: &Path) -> Result<PathBuf, FalhaDaGravacao> {
    if destino.is_absolute() {
        return Ok(destino.to_owned());
    }
    std::env::current_dir()
        .map(|corrente| corrente.join(destino))
        .map_err(|_| FalhaDaGravacao::DiretorioAusente)
}

fn criar_nivel(nivel: &Path) -> Result<(), FalhaDaGravacao> {
    #[cfg(unix)]
    let construtor = {
        use std::os::unix::fs::DirBuilderExt;
        let mut construtor = DirBuilder::new();
        construtor.mode(0o700);
        construtor
    };
    // Fora do Unix não há modo: vale a ACL do perfil.
    #[cfg(not(unix))]
    let construtor = DirBuilder::new();
    match construtor.create(nivel) {
        Ok(()) => Ok(()),
        // Outro processo criou ao mesmo tempo: vale se for diretório real.
        Err(erro) if erro.kind() == ErrorKind::AlreadyExists => match fs::symlink_metadata(nivel) {
            Ok(info) if info.file_type().is_symlink() => {
                Err(FalhaDaGravacao::DiretorioLinkSimbolico)
            }
            Ok(info) if info.is_dir() => Ok(()),
            _ => Err(FalhaDaGravacao::DiretorioAusente),
        },
        Err(_) => Err(FalhaDaGravacao::CriacaoFalhou),
    }
}

/// Grava `bytes` inteiros em `destino` conforme a política.
pub(crate) fn gravar(
    destino: &Path,
    bytes: &[u8],
    politica: Politica,
) -> Result<(), FalhaDaGravacao> {
    let mut parcial = Parcial::abrir(destino, politica)?;
    parcial.escrever(bytes)?;
    parcial.confirmar()
}

/// Parciais órfãos do mesmo nome com mais que isto sem modificação são
/// de um processo interrompido: um parcial vivo publica em segundos.
const IDADE_DO_ORFAO: Duration = Duration::from_secs(3_600);

/// Temporário `0600` no diretório do destino, ainda não publicado.
/// Removido no `Drop` se não for confirmado.
pub(crate) struct Parcial {
    temporario: PathBuf,
    destino: PathBuf,
    arquivo: Option<File>,
    politica: Politica,
    escritos: u64,
    confirmado: bool,
}

impl Parcial {
    /// Confere nome, ancestrais e destino; varre órfãos antigos do mesmo
    /// nome e cria o temporário.
    pub(crate) fn abrir(destino: &Path, politica: Politica) -> Result<Self, FalhaDaGravacao> {
        let nome = destino
            .file_name()
            .filter(|_| {
                !matches!(
                    destino.components().next_back(),
                    Some(Component::ParentDir | Component::CurDir | Component::RootDir)
                )
            })
            .ok_or(FalhaDaGravacao::SemNomeDeArquivo)?
            .to_owned();
        conferir_diretorios(destino, politica.criar_diretorios)?;
        conferir_destino(destino, politica.modo)?;
        let prefixo = prefixo_do_parcial(&nome);
        let pasta = destino.parent().unwrap_or(Path::new(""));
        varrer_orfaos(pasta, &prefixo);
        let (temporario, arquivo) = criar_temporario(pasta, &prefixo)?;
        Ok(Self {
            temporario,
            destino: destino.to_owned(),
            arquivo: Some(arquivo),
            politica,
            escritos: 0,
            confirmado: false,
        })
    }

    pub(crate) fn escrever(&mut self, bytes: &[u8]) -> Result<(), FalhaDaGravacao> {
        let total = self.escritos.saturating_add(bytes.len() as u64);
        if self.politica.limite.is_some_and(|limite| total > limite) {
            return Err(FalhaDaGravacao::AcimaDoLimite);
        }
        let arquivo = self
            .arquivo
            .as_mut()
            .ok_or(FalhaDaGravacao::EscritaFalhou)?;
        arquivo
            .write_all(bytes)
            .map_err(|_| FalhaDaGravacao::EscritaFalhou)?;
        self.escritos = total;
        Ok(())
    }

    /// `sync_all` e fecha o temporário; depois disso não há escrita.
    pub(crate) fn sincronizar(&mut self) -> Result<(), FalhaDaGravacao> {
        if let Some(arquivo) = self.arquivo.take() {
            arquivo
                .sync_all()
                .map_err(|_| FalhaDaGravacao::EscritaFalhou)?;
        }
        Ok(())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn temporario(&self) -> &Path {
        &self.temporario
    }

    /// Publica no nome final: `hard_link` (exclusivo, nunca sobrescreve)
    /// ou `rename` (substituição). O temporário some em qualquer caso.
    pub(crate) fn confirmar(mut self) -> Result<(), FalhaDaGravacao> {
        self.sincronizar()?;
        match self.politica.modo {
            Modo::Exclusivo => {
                fs::hard_link(&self.temporario, &self.destino).map_err(|erro| {
                    if erro.kind() == ErrorKind::AlreadyExists {
                        FalhaDaGravacao::DestinoJaExiste
                    } else {
                        FalhaDaGravacao::ConfirmacaoFalhou
                    }
                })?;
            }
            Modo::Substituir => {
                conferir_destino(&self.destino, Modo::Substituir)?;
                fs::rename(&self.temporario, &self.destino)
                    .map_err(|_| FalhaDaGravacao::ConfirmacaoFalhou)?;
                self.confirmado = true;
            }
        }
        sincronizar_pasta(&self.destino);
        Ok(())
    }
}

impl Drop for Parcial {
    fn drop(&mut self) {
        drop(self.arquivo.take());
        if !self.confirmado {
            let _ = fs::remove_file(&self.temporario);
        }
    }
}

impl fmt::Debug for Parcial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parcial")
            .field("temporario", &self.temporario)
            .field("destino", &self.destino)
            .finish_non_exhaustive()
    }
}

/// Dois parciais são o mesmo pelo caminho (o conteúdo não é comparado).
impl PartialEq for Parcial {
    fn eq(&self, outro: &Self) -> bool {
        self.temporario == outro.temporario && self.destino == outro.destino
    }
}

impl Eq for Parcial {}

/// Exclusivo: nada com o nome. Substituição: ausente ou arquivo regular.
fn conferir_destino(destino: &Path, modo: Modo) -> Result<(), FalhaDaGravacao> {
    match (fs::symlink_metadata(destino), modo) {
        (Err(erro), _) if erro.kind() == ErrorKind::NotFound => Ok(()),
        (Err(_), _) => Err(FalhaDaGravacao::CriacaoFalhou),
        (Ok(_), Modo::Exclusivo) => Err(FalhaDaGravacao::DestinoJaExiste),
        (Ok(info), Modo::Substituir) if info.file_type().is_file() => Ok(()),
        (Ok(_), Modo::Substituir) => Err(FalhaDaGravacao::DestinoNaoEhArquivoRegular),
    }
}

/// `.NOME.parcial-`: o mesmo prefixo dos parciais da #207.
fn prefixo_do_parcial(nome: &OsStr) -> String {
    format!(".{}.parcial-", nome.to_string_lossy())
}

fn criar_temporario(pasta: &Path, prefixo: &str) -> Result<(PathBuf, File), FalhaDaGravacao> {
    static SEQUENCIA: AtomicU64 = AtomicU64::new(0);
    for _ in 0..8 {
        let n = SEQUENCIA.fetch_add(1, Ordering::Relaxed);
        let temporario = pasta.join(format!("{prefixo}{}-{n}", std::process::id()));
        let mut opcoes = OpenOptions::new();
        opcoes.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opcoes.mode(0o600);
        }
        match opcoes.open(&temporario) {
            Ok(arquivo) => return Ok((temporario, arquivo)),
            // Órfão recente de um PID reaproveitado: o próximo número.
            Err(erro) if erro.kind() == ErrorKind::AlreadyExists => {}
            Err(_) => return Err(FalhaDaGravacao::CriacaoFalhou),
        }
    }
    Err(FalhaDaGravacao::CriacaoFalhou)
}

/// Remove `PREFIXO*` regulares sem modificação há mais de
/// [`IDADE_DO_ORFAO`]. Melhor esforço: nunca falha a gravação.
fn varrer_orfaos(pasta: &Path, prefixo: &str) {
    let pasta = if pasta.as_os_str().is_empty() {
        Path::new(".")
    } else {
        pasta
    };
    let Ok(entradas) = fs::read_dir(pasta) else {
        return;
    };
    let agora = SystemTime::now();
    for entrada in entradas.flatten() {
        if !entrada.file_name().to_string_lossy().starts_with(prefixo) {
            continue;
        }
        let caminho = entrada.path();
        let Ok(info) = fs::symlink_metadata(&caminho) else {
            continue;
        };
        let antigo = info
            .modified()
            .ok()
            .and_then(|quando| agora.duration_since(quando).ok())
            .is_some_and(|idade| idade > IDADE_DO_ORFAO);
        if info.file_type().is_file() && antigo {
            let _ = fs::remove_file(caminho);
        }
    }
}

/// Durabilidade da publicação; falha aqui não desfaz a gravação.
fn sincronizar_pasta(destino: &Path) {
    if let Some(pasta) = destino.parent().filter(|p| !p.as_os_str().is_empty()) {
        let _ = File::open(pasta).and_then(|d| d.sync_all());
    }
}

#[cfg(test)]
#[path = "arquivo_privado/tests.rs"]
mod tests;
