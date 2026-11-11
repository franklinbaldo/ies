//! Gravação privada única (#211): o par de escrita de
//! `transporte::arquivo_privado`, que é a leitura. Cria (exclusivo) ou
//! substitui (atômico) um arquivo `0600` sem nunca seguir link simbólico
//! no nome final, no diretório real (resolvido com `canonicalize`) que,
//! se é do IES, no Unix não é gravável por grupo ou outros sem sticky
//! bit (o do usuário não tem o modo conferido), por um
//! temporário `.NOME.parcial-PID-N` nesse diretório, `sync_all` e
//! `hard_link`/`rename`; o parcial nunca fica para trás numa falha.
//! Links em ancestrais são aceitos. Política, ameaça e limites:
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

/// De quem é o diretório do destino, declarado por todo chamador.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Diretorio {
    /// Gerenciado pelo IES (sessão, índice, configuração, cache): o
    /// diretório real não pode ser gravável por grupo ou outros sem
    /// sticky bit.
    DoIes,
    /// Escolhido pela pessoa (`--saida DIR`, o arquivo da caixa): o modo
    /// é responsabilidade dela; só valem a criação exclusiva e o nome
    /// final nunca seguido.
    DoUsuario,
}

/// O modo escolhido, ainda sem o dono do diretório: só vira [`Politica`]
/// com [`Self::diretorio_do_ies`] ou [`Self::diretorio_do_usuario`], para
/// que nenhum chamador fique num padrão implícito.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ModoDaGravacao(Modo);

impl ModoDaGravacao {
    pub(crate) const fn diretorio_do_ies(self) -> Politica {
        Politica::nova(self.0, Diretorio::DoIes)
    }

    pub(crate) const fn diretorio_do_usuario(self) -> Politica {
        Politica::nova(self.0, Diretorio::DoUsuario)
    }
}

/// Política da gravação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Politica {
    modo: Modo,
    diretorio: Diretorio,
    limite: Option<u64>,
    criar_diretorios: bool,
}

impl Politica {
    pub(crate) const fn exclusiva() -> ModoDaGravacao {
        ModoDaGravacao(Modo::Exclusivo)
    }

    pub(crate) const fn substituicao() -> ModoDaGravacao {
        ModoDaGravacao(Modo::Substituir)
    }

    const fn nova(modo: Modo, diretorio: Diretorio) -> Self {
        Self {
            modo,
            diretorio,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaDaGravacao {
    SemNomeDeArquivo,
    DiretorioAusente,
    /// Um nível do caminho existe e não é diretório.
    NivelDoCaminhoNaoEhDiretorio,
    /// Um nível do caminho não pôde ser consultado (o motivo do sistema).
    NivelDoCaminhoIlegivel(std::io::ErrorKind),
    /// O sistema recusou criar um nível `0700`.
    CriacaoDoDiretorioFalhou(std::io::ErrorKind),
    /// O sistema recusou criar o temporário `0600`.
    CriacaoDoTemporarioFalhou(std::io::ErrorKind),
    /// Oito nomes de temporário seguidos já existiam.
    NomesDeTemporarioEsgotados,
    /// O diretório real do destino é gravável por grupo ou outros
    /// (`mode & 0o022`) sem o sticky bit (`0o1000`). Só no Unix.
    #[cfg(unix)]
    DiretorioGravavelPorOutros,
    DestinoJaExiste,
    DestinoNaoEhArquivoRegular,
    AcimaDoLimite,
    /// Falha do sistema ao consultar o nome final antes da publicação.
    CriacaoFalhou(ErrorKind),
    EscritaFalhou,
    ConfirmacaoFalhou,
}

/// O pai do destino existe e é diretório (links no caminho são aceitos)
/// e, resolvido e se é do IES, não é gravável por grupo ou outros sem
/// sticky bit. Com
/// `criar`, os níveis ausentes nascem `0700`, um por vez, cada um num
/// diretório que passa pela mesma conferência. Conferência por caminho:
/// não impede a troca concorrente por quem pode escrever num ancestral
/// (a decisão documenta esse limite).
pub(crate) fn conferir_diretorios(
    destino: &Path,
    politica: Politica,
) -> Result<(), FalhaDaGravacao> {
    diretorio_real(destino, politica).map(|_| ())
}

/// O diretório real (sem links) do destino, já conferido.
fn diretorio_real(destino: &Path, politica: Politica) -> Result<PathBuf, FalhaDaGravacao> {
    let absoluto = absoluto(destino)?;
    let pai = absoluto.parent().ok_or(FalhaDaGravacao::DiretorioAusente)?;
    if politica.criar_diretorios {
        let mut niveis: Vec<&Path> = pai.ancestors().collect();
        niveis.reverse();
        for nivel in niveis {
            match fs::metadata(nivel) {
                Ok(info) if info.is_dir() => {}
                Err(erro) if erro.kind() == ErrorKind::NotFound => {
                    criar_nivel(nivel, politica.diretorio)?;
                }
                Ok(_) => return Err(FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio),
                Err(erro) => return Err(FalhaDaGravacao::NivelDoCaminhoIlegivel(erro.kind())),
            }
        }
    }
    real_e_conferido(pai, politica.diretorio)
}

/// `canonicalize` do diretório e, se é do IES, a conferência do modo do
/// diretório real.
fn real_e_conferido(pasta: &Path, dono: Diretorio) -> Result<PathBuf, FalhaDaGravacao> {
    let real = fs::canonicalize(pasta).map_err(|_| FalhaDaGravacao::DiretorioAusente)?;
    let info = fs::metadata(&real).map_err(|_| FalhaDaGravacao::DiretorioAusente)?;
    if !info.is_dir() {
        return Err(FalhaDaGravacao::DiretorioAusente);
    }
    // Fora do Unix não há modo: o dono não muda a conferência.
    #[cfg(not(unix))]
    let _ = dono;
    #[cfg(unix)]
    if dono == Diretorio::DoIes {
        use std::os::unix::fs::PermissionsExt;
        let modo = info.permissions().mode();
        if modo & 0o022 != 0 && modo & 0o1000 == 0 {
            return Err(FalhaDaGravacao::DiretorioGravavelPorOutros);
        }
    }
    Ok(real)
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

/// Cria um nível `0700` dentro de um diretório real conferido como o
/// destino (fechado a outros, se é do IES).
fn criar_nivel(nivel: &Path, dono: Diretorio) -> Result<(), FalhaDaGravacao> {
    let pai = nivel.parent().ok_or(FalhaDaGravacao::DiretorioAusente)?;
    let nivel = real_e_conferido(pai, dono)?
        .join(nivel.file_name().ok_or(FalhaDaGravacao::DiretorioAusente)?);
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
    match construtor.create(&nivel) {
        Ok(()) => Ok(()),
        // Outro processo criou ao mesmo tempo: vale se for diretório.
        Err(erro) if erro.kind() == ErrorKind::AlreadyExists => match fs::metadata(&nivel) {
            Ok(info) if info.is_dir() => Ok(()),
            Ok(_) => Err(FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio),
            Err(erro) => Err(FalhaDaGravacao::NivelDoCaminhoIlegivel(erro.kind())),
        },
        Err(erro) => Err(FalhaDaGravacao::CriacaoDoDiretorioFalhou(erro.kind())),
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
    /// Confere nome, diretório real e destino; varre órfãos antigos do
    /// mesmo nome e cria o temporário no diretório real.
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
        let pasta = diretorio_real(destino, politica)?;
        let destino = pasta.join(&nome);
        conferir_destino(&destino, politica.modo)?;
        let prefixo = prefixo_do_parcial(&nome);
        varrer_orfaos(&pasta, &prefixo);
        let (temporario, arquivo) = criar_temporario(&pasta, &prefixo)?;
        Ok(Self {
            temporario,
            destino,
            arquivo: Some(arquivo),
            politica,
            escritos: 0,
            confirmado: false,
        })
    }

    pub(crate) fn escrever(&mut self, bytes: &[u8]) -> Result<(), FalhaDaGravacao> {
        let total = u64::try_from(bytes.len())
            .map_or(u64::MAX, |novos| self.escritos.saturating_add(novos));
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
        (Err(erro), _) => Err(FalhaDaGravacao::CriacaoFalhou(erro.kind())),
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
            Err(erro) => return Err(FalhaDaGravacao::CriacaoDoTemporarioFalhou(erro.kind())),
        }
    }
    Err(FalhaDaGravacao::NomesDeTemporarioEsgotados)
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
