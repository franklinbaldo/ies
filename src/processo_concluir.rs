//! `ies processo concluir NUP [--confirmar]` (contracts/cli/processo-concluir.md):
//! abre o processo pelo caminho de `ies processo NUP` (sem o link guardado
//! da árvore, que não publica a unidade), lê `aberto_em` e a ação de
//! concluir da barra; prévia sem mais requisições; com `--confirmar`, uma
//! escrita (POST da tela de conclusão, ou o GET de `concluirProcesso()`) e
//! uma leitura nova da árvore para verificar. Nunca repete a escrita.
mod erro;
pub(crate) mod escrita;
mod fluxo;
mod tela;

#[cfg(test)]
mod acervo;
#[cfg(test)]
pub(crate) mod tests_apoio;
#[cfg(test)]
mod tests_fluxo;
#[cfg(test)]
mod tests_tela;

#[cfg(test)]
pub(crate) use erro::ErroConcluir;
pub(crate) use erro::{FalhaDoEnvio, VerificacaoPorLeitura};
pub(crate) use fluxo::concluir_processo;

use crate::processo::NumeroProcesso;

/// O NUP, a `--unidade` e se a escrita foi confirmada.
pub(crate) struct PedidoDaConclusao<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) confirmar: bool,
}

/// Como a barra oferece concluir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FormaDaConclusao {
    /// Link da tela "Conclusão de Processo" e POST de Salvar (4.1, 5.0).
    Tela,
    /// `concluirProcesso()`: o GET da URL da função (4.0).
    Funcao,
}

impl FormaDaConclusao {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::Tela => "tela",
            Self::Funcao => "funcao",
        }
    }
}

/// O que `ies processo concluir` encontrou ou fez na unidade da sessão.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DesfechoDaConclusao {
    /// Não está aberto na unidade e a barra não oferece concluir. `confirmado`
    /// é o pedido: sem escrita, nada muda.
    NaoAbertoNaUnidade { confirmado: bool },
    /// A barra oferece concluir; sem `--confirmar`, nada sai.
    Previa { forma: FormaDaConclusao },
    /// A escrita saiu e a leitura de verificação não lista mais a unidade.
    Concluido {
        forma: FormaDaConclusao,
        /// Siglas lidas na verificação (depois da escrita).
        depois: Vec<String>,
        /// A resposta não foi a esperada e a leitura mostrou a conclusão.
        reconciliado_por: Option<FalhaDoEnvio>,
    },
}

/// Sucesso: prévia, nada a fazer, ou concluído e conferido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoConcluido {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    /// A unidade da sessão (`lnkInfraUnidade` da tela do processo).
    pub(crate) unidade: String,
    /// Siglas de `aberto_em` antes.
    pub(crate) aberto_em: Vec<String>,
    pub(crate) requisicoes: u8,
    pub(crate) desfecho: DesfechoDaConclusao,
}

impl ProcessoConcluido {
    /// `aberto_em` sem a unidade da sessão: o que a verificação espera.
    pub(crate) fn depois_esperado(&self) -> Vec<String> {
        self.aberto_em
            .iter()
            .filter(|sigla| **sigla != self.unidade)
            .cloned()
            .collect()
    }

    pub(crate) fn confirmado(&self) -> bool {
        match &self.desfecho {
            DesfechoDaConclusao::NaoAbertoNaUnidade { confirmado } => *confirmado,
            DesfechoDaConclusao::Previa { .. } => false,
            DesfechoDaConclusao::Concluido { .. } => true,
        }
    }

    /// A escrita saiu e a leitura não lista mais a unidade.
    pub(crate) fn alterado(&self) -> bool {
        matches!(self.desfecho, DesfechoDaConclusao::Concluido { .. })
    }

    /// `None`: não está aberto na unidade e a barra não oferece concluir.
    pub(crate) fn forma(&self) -> Option<FormaDaConclusao> {
        match &self.desfecho {
            DesfechoDaConclusao::NaoAbertoNaUnidade { .. } => None,
            DesfechoDaConclusao::Previa { forma }
            | DesfechoDaConclusao::Concluido { forma, .. } => Some(*forma),
        }
    }

    /// Só depois da escrita.
    pub(crate) fn depois(&self) -> Option<&[String]> {
        match &self.desfecho {
            DesfechoDaConclusao::Concluido { depois, .. } => Some(depois.as_slice()),
            DesfechoDaConclusao::NaoAbertoNaUnidade { .. } | DesfechoDaConclusao::Previa { .. } => {
                None
            }
        }
    }

    pub(crate) fn reconciliado_por(&self) -> Option<&FalhaDoEnvio> {
        match &self.desfecho {
            DesfechoDaConclusao::Concluido {
                reconciliado_por, ..
            } => reconciliado_por.as_ref(),
            DesfechoDaConclusao::NaoAbertoNaUnidade { .. } | DesfechoDaConclusao::Previa { .. } => {
                None
            }
        }
    }
}
