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

/// Sucesso: prévia, nada a fazer, ou concluído e conferido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoConcluido {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    /// A unidade da sessão (`lnkInfraUnidade` da tela do processo).
    pub(crate) unidade: String,
    pub(crate) confirmado: bool,
    /// A escrita saiu e a leitura não lista mais a unidade.
    pub(crate) alterado: bool,
    /// `None`: não está aberto na unidade e a barra não oferece concluir.
    pub(crate) forma: Option<FormaDaConclusao>,
    /// Siglas de `aberto_em` antes.
    pub(crate) aberto_em: Vec<String>,
    /// Siglas lidas na verificação (só depois da escrita).
    pub(crate) depois: Option<Vec<String>>,
    /// A resposta não foi a esperada e a leitura mostrou a conclusão.
    pub(crate) reconciliado_por: Option<FalhaDoEnvio>,
    pub(crate) requisicoes: u8,
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
}
