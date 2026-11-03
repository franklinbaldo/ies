//! Abertura do banco: permissões, links simbólicos, pragmas e esquema.
//!
//! O diretório nasce `0700` e o banco `0600` com `O_CREAT|O_EXCL`
//! (`create_new`, que falha diante de um link simbólico); o `-wal` e o
//! `-shm` herdam o modo do banco. Diretório existente aberto a grupo/outros,
//! ou banco/lateral que seja link simbólico, não regular ou aberto a
//! grupo/outros, é falha: o índice se desliga. O IES não muda o modo do que
//! não criou.
use rusqlite::{Connection, OpenFlags};
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::ErrorKind;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Falha sem dados: o erro do SQLite ou do sistema nunca sai do módulo.
#[derive(Debug)]
pub(super) struct Falha;

impl From<rusqlite::Error> for Falha {
    fn from(_: rusqlite::Error) -> Self {
        Self
    }
}

impl From<std::io::Error> for Falha {
    fn from(_: std::io::Error) -> Self {
        Self
    }
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
];

pub(super) fn versao_do_esquema() -> i64 {
    i64::try_from(MIGRACOES.len()).unwrap_or(i64::MAX)
}

/// Sem bits de grupo nem de outros (a mesma regra dos arquivos de sessão).
fn privado(modo: u32) -> bool {
    let aberto_a_outros = modo & 0o077 != 0;
    !aberto_a_outros
}

fn conferir_diretorio(diretorio: &Path) -> Result<(), Falha> {
    match fs::metadata(diretorio) {
        Ok(info) if info.is_dir() && privado(info.permissions().mode()) => Ok(()),
        Ok(_) => Err(Falha),
        Err(erro) if erro.kind() == ErrorKind::NotFound => {
            DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(diretorio)?;
            let info = fs::metadata(diretorio)?;
            if info.is_dir() && privado(info.permissions().mode()) {
                Ok(())
            } else {
                Err(Falha)
            }
        }
        Err(erro) => Err(erro.into()),
    }
}

/// Arquivo regular, não link simbólico, sem bits de grupo/outros.
/// `Ok(false)`: não existe.
fn arquivo_privado_existente(caminho: &Path) -> Result<bool, Falha> {
    match fs::symlink_metadata(caminho) {
        Ok(info) => {
            if info.file_type().is_symlink()
                || !info.is_file()
                || !privado(info.permissions().mode())
            {
                return Err(Falha);
            }
            Ok(true)
        }
        Err(erro) if erro.kind() == ErrorKind::NotFound => Ok(false),
        Err(erro) => Err(erro.into()),
    }
}

fn lateral(caminho: &Path, sufixo: &str) -> PathBuf {
    let mut nome = caminho.as_os_str().to_owned();
    nome.push(sufixo);
    PathBuf::from(nome)
}

fn conferir_arquivos(caminho: &Path) -> Result<(), Falha> {
    if !arquivo_privado_existente(caminho)? {
        let criado = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(caminho);
        match criado {
            Ok(_) => {}
            // Outro `ies` criou ao mesmo tempo: confere o dele.
            Err(erro) if erro.kind() == ErrorKind::AlreadyExists => {}
            Err(erro) => return Err(erro.into()),
        }
        if !arquivo_privado_existente(caminho)? {
            return Err(Falha);
        }
    }
    for sufixo in ["-wal", "-shm", "-journal"] {
        arquivo_privado_existente(&lateral(caminho, sufixo))?;
    }
    Ok(())
}

/// Abre o banco já conferido; `preparar` aplica WAL e o esquema (uma vez
/// por `IndiceLocal`).
pub(super) fn conectar(caminho: &Path, preparar: bool) -> Result<Connection, Falha> {
    let diretorio = match caminho.parent() {
        Some(pai) if !pai.as_os_str().is_empty() => pai,
        _ => Path::new("."),
    };
    conferir_diretorio(diretorio)?;
    conferir_arquivos(caminho)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_NOFOLLOW
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut conexao = Connection::open_with_flags(caminho, flags)?;
    conexao.busy_timeout(ESPERA_DO_LOCK)?;
    if preparar {
        let modo: String =
            conexao.pragma_update_and_check(None, "journal_mode", "WAL", |l| l.get(0))?;
        if !modo.eq_ignore_ascii_case("wal") {
            return Err(Falha);
        }
        migrar(&mut conexao)?;
    }
    Ok(conexao)
}

/// O esquema e as migrações numa transação só: o que falhar no meio desfaz
/// tudo, e a próxima abertura recomeça do banco como estava.
fn migrar(conexao: &mut Connection) -> Result<(), Falha> {
    let transacao = conexao.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let versao: i64 = transacao.pragma_query_value(None, "user_version", |l| l.get(0))?;
    let inicio = usize::try_from(versao).map_err(|_| Falha)?;
    if inicio > MIGRACOES.len() {
        return Err(Falha);
    }
    for migracao in &MIGRACOES[inicio..] {
        transacao.execute_batch(migracao)?;
    }
    if inicio < MIGRACOES.len() {
        transacao.pragma_update(None, "user_version", versao_do_esquema())?;
    }
    transacao.commit()?;
    Ok(())
}
