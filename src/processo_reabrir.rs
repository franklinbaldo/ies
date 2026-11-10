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

/// Sucesso: prévia, nada a fazer, ou reaberto e conferido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoReaberto {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    /// A unidade da sessão (`lnkInfraUnidade` da tela do processo).
    pub(crate) unidade: String,
    pub(crate) confirmado: bool,
    /// A escrita saiu e a leitura lista a unidade.
    pub(crate) alterado: bool,
    /// `false`: já aberto na unidade, nada a fazer.
    pub(crate) oferecido: bool,
    /// Onde a tela do nó publicou a URL (só depois de pedi-la).
    pub(crate) forma: Option<FormaDaReabertura>,
    /// Siglas de `aberto_em` antes.
    pub(crate) aberto_em: Vec<String>,
    /// Siglas lidas na verificação (só depois da escrita).
    pub(crate) depois: Option<Vec<String>>,
    /// A resposta não foi a esperada e a leitura mostrou a reabertura.
    pub(crate) reconciliado_por: Option<FalhaDoEnvio>,
    pub(crate) requisicoes: u8,
}
