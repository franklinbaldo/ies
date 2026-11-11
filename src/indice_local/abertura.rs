//! Abertura do banco: permissões, links simbólicos, pragmas e esquema.
//!
//! O diretório nasce `0700` e o banco `0600`, novo, pela gravação privada
//! única (`crate::arquivo_privado`: nenhum link no banco; o caminho é
//! resolvido e o banco é aberto pelo diretório real);
//! o `-wal` e o `-shm` herdam o modo do banco. Diretório existente aberto a grupo/outros,
//! ou banco/lateral que seja link simbólico, não regular ou aberto a
//! grupo/outros, é falha: o índice se desliga. O IES não muda o modo do que
//! não criou. Fora do Unix não há modo de arquivo: vale a ACL do perfil do
//! usuário, como nos arquivos de sessão (`configuracao`, `sessao_login`).
use rusqlite::{Connection, OpenFlags};
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Por que o banco não abriu. O índice se desliga em qualquer caso
/// (`IndiceLocal::executar`); o erro diz qual conferência recusou.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(super) enum ErroAberturaDoIndice {
    DiretoriosRecusados(#[source] crate::arquivo_privado::FalhaDaGravacao),
    DiretorioIlegivel(#[source] std::io::Error),
    DiretorioNaoPrivado,
    CaminhoDoBancoSemNomeDeArquivo,
    ArquivoIlegivel(#[source] std::io::Error),
    /// Link simbólico, não regular, ou aberto a grupo/outros.
    ArquivoNaoPrivado,
    CriacaoDoBancoRecusada(#[source] crate::arquivo_privado::FalhaDaGravacao),
    BancoAusenteAposCriacao,
    TravaDaPreparacaoIlegivel(#[source] std::io::Error),
    TravaDaPreparacaoOcupada,
    ModoWalRecusado,
    VersaoDoEsquemaNegativa(i64),
    EsquemaDeBinarioMaisNovo,
    Sqlite(#[from] rusqlite::Error),
}

/// Vários `ies` ao mesmo tempo: as transações são minúsculas.
const ESPERA_DO_LOCK: Duration = Duration::from_secs(2);

/// `MIGRACOES[v]` leva o banco da versão `v` à `v + 1`. Só para frente:
/// versão maior que `MIGRACOES.len()` é de um binário mais novo e desliga.
const MIGRACOES: &[&str] = &[
    // 0 → 1: as três tabelas de vidas distintas (ADR 0003).
    "CREATE TABLE instalacao (
        origem TEXT PRIMARY KEY NOT NULL,
        entrada_por_id INTEGER CHECK (entrada_por_id IN (0, 1)),
        entrada_por_id_testado_em REAL,
        recusas INTEGER NOT NULL DEFAULT 0
    ) STRICT;
    CREATE TABLE processo (
        origem TEXT NOT NULL,
        nup TEXT NOT NULL,
        id_procedimento TEXT NOT NULL,
        observado_em REAL NOT NULL,
        PRIMARY KEY (origem, nup),
        UNIQUE (origem, id_procedimento)
    ) STRICT;
    CREATE TABLE link_assinado (
        origem TEXT NOT NULL,
        geracao_sessao TEXT NOT NULL,
        unidade TEXT NOT NULL,
        acao TEXT NOT NULL,
        objeto TEXT NOT NULL,
        url TEXT NOT NULL,
        observado_em REAL NOT NULL,
        PRIMARY KEY (origem, geracao_sessao, unidade, acao, objeto)
    ) STRICT;",
    // 1 → 2 (Refs #60, fatia 2): número SEI ↔ (processo, documento) e o
    // aceite da entrada por id com `id_documento`, medido à parte.
    "CREATE TABLE documento (
        origem TEXT NOT NULL,
        numero_sei TEXT NOT NULL,
        id_procedimento TEXT NOT NULL,
        id_documento TEXT NOT NULL,
        observado_em REAL NOT NULL,
        PRIMARY KEY (origem, numero_sei),
        UNIQUE (origem, id_documento)
    ) STRICT;
    ALTER TABLE instalacao ADD COLUMN entrada_com_documento INTEGER
        CHECK (entrada_com_documento IN (0, 1));
    ALTER TABLE instalacao ADD COLUMN entrada_com_documento_testado_em REAL;
    ALTER TABLE instalacao ADD COLUMN recusas_com_documento INTEGER NOT NULL DEFAULT 0;",
    // 2 → 3 (#244): snapshot completo por instalação e unidade ativa.
    // A tabela 'unidade' é contrato público exclusivamente de leitura.
    "CREATE TABLE lista_unidade (
        instalacao TEXT NOT NULL,
        unidade_ativa TEXT NOT NULL,
        lida_em REAL NOT NULL,
        total INTEGER NOT NULL CHECK (total >= 0),
        PRIMARY KEY (instalacao, unidade_ativa)
    ) STRICT;
    CREATE TABLE unidade (
        instalacao TEXT NOT NULL,
        unidade_ativa TEXT NOT NULL,
        id TEXT NOT NULL,
        sigla TEXT NOT NULL,
        descricao TEXT NOT NULL,
        orgao TEXT NOT NULL,
        lida_em REAL NOT NULL,
        PRIMARY KEY (instalacao, unidade_ativa, id),
        FOREIGN KEY (instalacao, unidade_ativa)
            REFERENCES lista_unidade(instalacao, unidade_ativa)
    ) STRICT;
    CREATE INDEX unidade_sigla
        ON unidade(instalacao, unidade_ativa, sigla);",
    // 3 → 4: projeção tipada do Controle; sem dados de URL/cookie.
    iesei::sqlite::MIGRACAO_CONTROLE,
];

pub(super) fn versao_do_esquema() -> i64 {
    i64::try_from(MIGRACOES.len()).unwrap_or(i64::MAX)
}

/// Sem bits de grupo nem de outros (a mesma regra dos arquivos de sessão).
#[cfg(unix)]
fn privado(info: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let aberto_a_outros = info.permissions().mode() & 0o077 != 0;
    !aberto_a_outros
}

/// Sem modo de arquivo fora do Unix: a ACL do perfil decide.
#[cfg(not(unix))]
fn privado(_info: &fs::Metadata) -> bool {
    true
}

/// `0600` no Unix e, no Linux, `O_NOFOLLOW` (o último componente não pode
/// ser link simbólico; o mesmo valor da gravação da sessão).
fn opcoes_privadas(opcoes: &mut OpenOptions) -> &mut OpenOptions {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opcoes.mode(0o600);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opcoes.custom_flags(0o400_000);
    }
    opcoes
}

/// O banco é do IES: diretório real fechado a outros, níveis `0700`.
fn politica_do_banco() -> crate::arquivo_privado::Politica {
    crate::arquivo_privado::Politica::exclusiva()
        .diretorio_do_ies()
        .criando_diretorios()
}

/// Os níveis ausentes nascem `0700` e o diretório real do banco (links
/// no caminho são aceitos) é privado; o IES não muda o modo do que não
/// criou.
fn conferir_diretorio(banco: &Path, diretorio: &Path) -> Result<(), ErroAberturaDoIndice> {
    crate::arquivo_privado::conferir_diretorios(banco, politica_do_banco())
        .map_err(ErroAberturaDoIndice::DiretoriosRecusados)?;
    let info = fs::metadata(diretorio).map_err(ErroAberturaDoIndice::DiretorioIlegivel)?;
    if info.is_dir() && privado(&info) {
        Ok(())
    } else {
        Err(ErroAberturaDoIndice::DiretorioNaoPrivado)
    }
}

/// Arquivo regular, não link simbólico, sem bits de grupo/outros.
/// `Ok(false)`: não existe.
fn arquivo_privado_existente(caminho: &Path) -> Result<bool, ErroAberturaDoIndice> {
    match fs::symlink_metadata(caminho) {
        Ok(info) => {
            if info.file_type().is_symlink() || !info.is_file() || !privado(&info) {
                return Err(ErroAberturaDoIndice::ArquivoNaoPrivado);
            }
            Ok(true)
        }
        Err(erro) if erro.kind() == ErrorKind::NotFound => Ok(false),
        Err(erro) => Err(ErroAberturaDoIndice::ArquivoIlegivel(erro)),
    }
}

fn lateral(caminho: &Path, sufixo: &str) -> PathBuf {
    let mut nome = caminho.as_os_str().to_owned();
    nome.push(sufixo);
    PathBuf::from(nome)
}

fn conferir_arquivos(caminho: &Path) -> Result<(), ErroAberturaDoIndice> {
    if !arquivo_privado_existente(caminho)? {
        use crate::arquivo_privado::{FalhaDaGravacao, gravar};
        match gravar(caminho, b"", politica_do_banco()) {
            // Outro `ies` criou ao mesmo tempo: confere o dele.
            Ok(()) | Err(FalhaDaGravacao::DestinoJaExiste) => {}
            Err(falha) => return Err(ErroAberturaDoIndice::CriacaoDoBancoRecusada(falha)),
        }
        if !arquivo_privado_existente(caminho)? {
            return Err(ErroAberturaDoIndice::BancoAusenteAposCriacao);
        }
    }
    for sufixo in ["-wal", "-shm", "-journal", ".lock"] {
        arquivo_privado_existente(&lateral(caminho, sufixo))?;
    }
    Ok(())
}

/// Abre o banco já conferido; `preparar` aplica WAL e o esquema (uma vez
/// por `IndiceLocal`).
pub(super) fn conectar(caminho: &Path, preparar: bool) -> Result<Connection, ErroAberturaDoIndice> {
    let diretorio = match caminho.parent() {
        Some(pai) if !pai.as_os_str().is_empty() => pai,
        _ => Path::new("."),
    };
    conferir_diretorio(caminho, diretorio)?;
    // O SQLite com `NOFOLLOW` recusa link em qualquer componente: o banco
    // é aberto pelo diretório real, já conferido.
    let nome = caminho
        .file_name()
        .ok_or(ErroAberturaDoIndice::CaminhoDoBancoSemNomeDeArquivo)?;
    let caminho = &fs::canonicalize(diretorio)
        .map_err(ErroAberturaDoIndice::DiretorioIlegivel)?
        .join(nome);
    conferir_arquivos(caminho)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_NOFOLLOW
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut conexao = Connection::open_with_flags(caminho, flags)?;
    conexao.busy_timeout(ESPERA_DO_LOCK)?;
    if preparar {
        // Trocar para WAL exige lock exclusivo: com outra conexão aberta no
        // modo antigo, o SQLite devolve o modo anterior sem erro. A preparação
        // é serializada entre processos por um lock lateral.
        let _trava = TravaDaPreparacao::obter(caminho)?;
        let modo: String =
            conexao.pragma_update_and_check(None, "journal_mode", "WAL", |l| l.get(0))?;
        if !modo.eq_ignore_ascii_case("wal") {
            return Err(ErroAberturaDoIndice::ModoWalRecusado);
        }
        migrar(&mut conexao)?;
    }
    Ok(conexao)
}

/// Lock exclusivo de `<banco>.lock` (`0600`, sem seguir link simbólico),
/// liberado ao sair de escopo; espera no máximo `ESPERA_DO_LOCK`.
struct TravaDaPreparacao {
    _arquivo: fs::File,
}

impl TravaDaPreparacao {
    fn obter(banco: &Path) -> Result<Self, ErroAberturaDoIndice> {
        let arquivo = opcoes_privadas(OpenOptions::new().write(true).create(true).truncate(false))
            .open(lateral(banco, ".lock"))
            .map_err(ErroAberturaDoIndice::TravaDaPreparacaoIlegivel)?;
        let limite = std::time::Instant::now() + ESPERA_DO_LOCK;
        loop {
            match arquivo.try_lock() {
                Ok(()) => return Ok(Self { _arquivo: arquivo }),
                Err(fs::TryLockError::WouldBlock) if std::time::Instant::now() < limite => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(fs::TryLockError::WouldBlock) => {
                    return Err(ErroAberturaDoIndice::TravaDaPreparacaoOcupada);
                }
                Err(fs::TryLockError::Error(erro)) => {
                    return Err(ErroAberturaDoIndice::TravaDaPreparacaoIlegivel(erro));
                }
            }
        }
    }
}

/// O esquema e as migrações numa transação só: o que falhar no meio desfaz
/// tudo, e a próxima abertura recomeça do banco como estava.
fn migrar(conexao: &mut Connection) -> Result<(), ErroAberturaDoIndice> {
    let transacao = conexao.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let versao: i64 = transacao.pragma_query_value(None, "user_version", |l| l.get(0))?;
    let inicio = usize::try_from(versao)
        .map_err(|_| ErroAberturaDoIndice::VersaoDoEsquemaNegativa(versao))?;
    let pendentes = MIGRACOES
        .get(inicio..)
        .ok_or(ErroAberturaDoIndice::EsquemaDeBinarioMaisNovo)?;
    for migracao in pendentes {
        transacao.execute_batch(migracao)?;
    }
    if inicio < MIGRACOES.len() {
        transacao.pragma_update(None, "user_version", versao_do_esquema())?;
    }
    transacao.commit()?;
    Ok(())
}
