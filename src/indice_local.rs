//! Índice local SQLite ([ADR 0003](../docs/arquitetura/0003-indice-local.md)):
//! um palpite barato que o servidor sempre vence.
//!
//! - `instalacao`: a instalação aceita a entrada sem assinatura
//!   `procedimento_trabalhar&id_procedimento=N`? (recusa expira em 30 dias);
//! - `processo`: identidade durável NUP ↔ `id_procedimento`;
//! - `link_assinado`: URLs assinadas **literais**, válidas só na geração de
//!   sessão (e unidade) em que foram vistas.
//!
//! Falha do banco nunca derruba um comando: o índice se desliga e tudo segue
//! como se ele não existisse (`None` nas consultas, nada nas gravações).
//! Nenhuma requisição é feita para alimentá-lo e nenhuma transação atravessa
//! HTTP: cada chamada abre, roda uma transação minúscula e fecha.
mod abertura;
mod entrada_por_id;
mod identidade;
mod links;
mod validacao;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_entrada;
#[cfg(test)]
mod tests_links;

use crate::configuracao::ArquivosDeAcesso;
use std::cell::Cell;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};

// A integração (pesquisa, `processo`, login) vem em PRs seguintes.
#[allow(unused_imports)]
pub(crate) use entrada_por_id::url_da_entrada_por_id;
pub(crate) use links::ChaveDoLink;
#[allow(unused_imports)]
pub(crate) use validacao::{RegistroRecusado, UrlAssinada};

/// Nome do banco ao lado do `IES_URL_CONTROLE_FILE`.
pub(crate) const NOME_DO_ARQUIVO: &str = "ies.sqlite3";

type Relogio = Box<dyn Fn() -> f64 + Send>;

/// Índice de um processo `ies`. Não é `Sync`; cada processo (ou thread)
/// abre o seu e o SQLite coordena o arquivo (WAL e `busy_timeout`).
pub(crate) struct IndiceLocal {
    caminho: Option<PathBuf>,
    ligado: Cell<bool>,
    pronto: Cell<bool>,
    relogio: Relogio,
}

/// Nem o caminho (diz onde a sessão mora) nem dados.
impl fmt::Debug for IndiceLocal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IndiceLocal")
            .field("ligado", &self.ligado())
            .finish_non_exhaustive()
    }
}

/// Caminho do banco: `None` se `IES_SEM_INDICE` tem valor não vazio
/// diferente de `0`; `IES_INDICE_FILE` não vazio; senão `ies.sqlite3` no
/// diretório do arquivo da URL do Controle.
pub(crate) fn caminho_do_indice(
    sem_indice: Option<&OsStr>,
    arquivo: Option<&OsStr>,
    url_controle: &Path,
) -> Option<PathBuf> {
    if sem_indice.is_some_and(|valor| !valor.is_empty() && valor != "0") {
        return None;
    }
    if let Some(arquivo) = arquivo.filter(|valor| !valor.is_empty()) {
        return Some(PathBuf::from(arquivo));
    }
    let pasta = url_controle.parent().unwrap_or(Path::new(""));
    Some(pasta.join(NOME_DO_ARQUIVO))
}

impl IndiceLocal {
    /// Lê `IES_SEM_INDICE` e `IES_INDICE_FILE` do ambiente.
    pub(crate) fn da_configuracao(arquivos: &ArquivosDeAcesso) -> Self {
        match caminho_do_indice(
            std::env::var_os("IES_SEM_INDICE").as_deref(),
            std::env::var_os("IES_INDICE_FILE").as_deref(),
            &arquivos.url_controle,
        ) {
            Some(caminho) => Self::em(caminho),
            None => Self::desligado(),
        }
    }

    /// Índice no caminho dado; nada é aberto até a primeira chamada.
    pub(crate) fn em(caminho: PathBuf) -> Self {
        Self {
            caminho: Some(caminho),
            ligado: Cell::new(true),
            pronto: Cell::new(false),
            relogio: Box::new(agora_do_sistema),
        }
    }

    pub(crate) fn desligado() -> Self {
        Self {
            caminho: None,
            ligado: Cell::new(false),
            pronto: Cell::new(false),
            relogio: Box::new(agora_do_sistema),
        }
    }

    /// Relógio injetado (segundos desde a época), para testes.
    #[cfg(test)]
    pub(crate) fn com_relogio(mut self, relogio: impl Fn() -> f64 + Send + 'static) -> Self {
        self.relogio = Box::new(relogio);
        self
    }

    /// `false` depois da primeira falha do banco (ou se nasceu desligado).
    pub(crate) fn ligado(&self) -> bool {
        self.ligado.get() && self.caminho.is_some()
    }

    /// Roda `operacao` numa transação minúscula; qualquer falha do banco
    /// desliga o índice e devolve `padrao`. Nunca atravessa HTTP: o chamador
    /// consulta, faz a requisição e só depois registra.
    fn executar<T>(
        &self,
        padrao: T,
        operacao: impl FnOnce(&rusqlite::Transaction<'_>, f64) -> rusqlite::Result<T>,
    ) -> T {
        let Some(caminho) = self.caminho.as_deref().filter(|_| self.ligado.get()) else {
            return padrao;
        };
        let resultado = abertura::conectar(caminho, !self.pronto.get()).and_then(|mut conexao| {
            self.pronto.set(true);
            let transacao =
                conexao.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let valor = operacao(&transacao, (self.relogio)())?;
            transacao.commit()?;
            Ok(valor)
        });
        resultado.unwrap_or_else(|abertura::Falha| {
            self.ligado.set(false);
            padrao
        })
    }
}

fn agora_do_sistema() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}
