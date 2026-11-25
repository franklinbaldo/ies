//! `ies processo reabrir NUP [--confirmar]` (contracts/cli/processo-reabrir.md):
//! abre o processo pelo caminho de `ies processo NUP` (sem o link guardado
//! da árvore, que não publica a unidade), lê `aberto_em` e a ação de
//! reabrir da barra; prévia sem mais requisições; com `--confirmar`, o GET
//! da tela do nó, o GET da URL de reabrir lida nela (a escrita) e uma
//! leitura nova da árvore para verificar. Nunca repete a escrita. As peças
//! comuns com `processo concluir` estão em `processo_concluir::escrita`.
mod erro;
mod fluxo;

#[cfg(test)]
mod acervo;
#[cfg(test)]
mod tests_fluxo;

#[cfg(test)]
pub(crate) use erro::ErroReabrir;
pub(crate) use fluxo::reabrir_processo;

use crate::arvore_processo::FormaDaReabertura;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::FalhaDoEnvio;

/// O NUP, a `--unidade` e se a escrita foi confirmada.
pub(crate) struct PedidoDaReabertura<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) confirmar: bool,
}

/// O que `ies processo reabrir` encontrou ou fez na unidade da sessão.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DesfechoDaReabertura {
    /// Já aberto na unidade da sessão. `confirmado` é o pedido: sem escrita,
    /// nada muda.
    JaAbertoNaUnidade { confirmado: bool },
    /// A barra oferece reabrir; sem `--confirmar`, nada sai.
    Previa,
    /// A escrita saiu e a leitura de verificação lista a unidade.
    Reaberto {
        /// Onde a tela do nó publicou a URL (só depois de pedi-la).
        forma: FormaDaReabertura,
        /// Siglas lidas na verificação (depois da escrita).
        depois: Vec<String>,
        /// A resposta não foi a esperada e a leitura mostrou a reabertura.
        reconciliado_por: Option<FalhaDoEnvio>,
    },
}

/// Sucesso: prévia, nada a fazer, ou reaberto e conferido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoReaberto {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    /// A unidade da sessão (`lnkInfraUnidade` da tela do processo).
    pub(crate) unidade: String,
    /// Siglas de `aberto_em` antes.
    pub(crate) aberto_em: Vec<String>,
    pub(crate) requisicoes: u8,
    pub(crate) desfecho: DesfechoDaReabertura,
}

impl ProcessoReaberto {
    pub(crate) fn confirmado(&self) -> bool {
        match &self.desfecho {
            DesfechoDaReabertura::JaAbertoNaUnidade { confirmado } => *confirmado,
            DesfechoDaReabertura::Previa => false,
            DesfechoDaReabertura::Reaberto { .. } => true,
        }
    }

    /// A escrita saiu e a leitura lista a unidade.
    pub(crate) fn alterado(&self) -> bool {
        matches!(self.desfecho, DesfechoDaReabertura::Reaberto { .. })
    }

    /// `false`: já aberto na unidade, nada a fazer.
    pub(crate) fn oferecido(&self) -> bool {
        !matches!(
            self.desfecho,
            DesfechoDaReabertura::JaAbertoNaUnidade { .. }
        )
    }

    /// Só depois da escrita.
    pub(crate) fn forma(&self) -> Option<FormaDaReabertura> {
        match &self.desfecho {
            DesfechoDaReabertura::Reaberto { forma, .. } => Some(*forma),
            DesfechoDaReabertura::JaAbertoNaUnidade { .. } | DesfechoDaReabertura::Previa => None,
        }
    }

    /// Só depois da escrita.
    pub(crate) fn depois(&self) -> Option<&[String]> {
        match &self.desfecho {
            DesfechoDaReabertura::Reaberto { depois, .. } => Some(depois.as_slice()),
            DesfechoDaReabertura::JaAbertoNaUnidade { .. } | DesfechoDaReabertura::Previa => None,
        }
    }

    pub(crate) fn reconciliado_por(&self) -> Option<&FalhaDoEnvio> {
        match &self.desfecho {
            DesfechoDaReabertura::Reaberto {
                reconciliado_por, ..
            } => reconciliado_por.as_ref(),
            DesfechoDaReabertura::JaAbertoNaUnidade { .. } | DesfechoDaReabertura::Previa => None,
        }
    }
}
