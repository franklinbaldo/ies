//! Gravação privada única (#211): o par de escrita de
//! `transporte::arquivo_privado`, que é a leitura. Shim do contrato (RED).
use std::path::{Path, PathBuf};

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

    #[cfg_attr(not(test), allow(dead_code))]
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

pub(crate) fn conferir_diretorios(_destino: &Path, _criar: bool) -> Result<(), FalhaDaGravacao> {
    Err(FalhaDaGravacao::EscritaFalhou)
}

pub(crate) fn gravar(
    _destino: &Path,
    _bytes: &[u8],
    _politica: Politica,
) -> Result<(), FalhaDaGravacao> {
    Err(FalhaDaGravacao::EscritaFalhou)
}

/// Temporário ainda não confirmado; removido no `Drop`.
#[derive(Debug)]
pub(crate) struct Parcial {
    temporario: PathBuf,
}

impl Parcial {
    pub(crate) fn abrir(_destino: &Path, _politica: Politica) -> Result<Self, FalhaDaGravacao> {
        Err(FalhaDaGravacao::EscritaFalhou)
    }

    pub(crate) fn escrever(&mut self, _bytes: &[u8]) -> Result<(), FalhaDaGravacao> {
        Err(FalhaDaGravacao::EscritaFalhou)
    }

    pub(crate) fn sincronizar(&mut self) -> Result<(), FalhaDaGravacao> {
        Err(FalhaDaGravacao::EscritaFalhou)
    }

    pub(crate) fn temporario(&self) -> &Path {
        &self.temporario
    }

    pub(crate) fn confirmar(self) -> Result<(), FalhaDaGravacao> {
        Err(FalhaDaGravacao::EscritaFalhou)
    }
}

#[cfg(test)]
#[path = "arquivo_privado/tests.rs"]
mod tests;
