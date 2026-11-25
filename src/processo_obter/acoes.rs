//! As ações de estado que `abrir_para_escrever` lê na barra da árvore.
use crate::arvore_processo::{
    ConcluirNaBarra, EnderecoDeEscrita, EnderecoDoIncluirDocumento, ErroAcaoSobrestar,
    ErroAcompanharNaBarra, ErroAndamentoNaBarra, ErroAnexarNaBarra, ErroAtribuirNaBarra,
    ErroConcluirNaBarra, ErroEnviarNaBarra, ErroLinkIncluirDocumento, ErroRelacionarNaBarra,
    ReabrirNaBarra,
};
use crate::processo_comentarios::{EnderecoDosComentarios, ErroLinkDosComentarios};

/// As ações de estado lidas na barra da árvore aberta; a falha de cada uma
/// só pesa no comando que a usa.
pub(crate) struct AcoesDeEstado {
    /// Link da inclusão de documentos internos; falha só pesa no comando #246.
    pub(crate) incluir: Result<Option<EnderecoDoIncluirDocumento>, ErroLinkIncluirDocumento>,
    /// contracts/cli/processo-prazo.md
    pub(crate) prazo: Result<Option<String>, crate::arvore_processo::ErroLinkPrazo>,
    /// contracts/cli/processo-concluir.md
    pub(crate) concluir: Result<Option<ConcluirNaBarra>, ErroConcluirNaBarra>,
    /// contracts/cli/processo-reabrir.md
    pub(crate) reabrir: Result<Option<ReabrirNaBarra>, crate::arvore_processo::ErroReabrirNaBarra>,
    pub(crate) sobrestar: Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar>,
    pub(crate) retomar: Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar>,
    /// contracts/cli/processo-enviar.md
    pub(crate) enviar: Result<Option<EnderecoDeEscrita>, ErroEnviarNaBarra>,
    /// Gerenciar marcadores, URL literal publicada na barra da árvore.
    pub(crate) marcadores: Result<
        Option<crate::marcadores_processo::EnderecoDosMarcadores>,
        crate::marcadores_processo::ErroMarcadores,
    >,
    /// Link das anotações da origem (só pesa no envio com remoção).
    pub(crate) anotacoes: Result<Option<String>, crate::arvore_processo::ErroLinkDasAnotacoes>,
    pub(crate) comentarios: Result<Option<EnderecoDosComentarios>, ErroLinkDosComentarios>,
    /// Ciência no processo e sua lista de conferência, ambas da árvore servida.
    pub(crate) ciencia:
        Result<Option<EnderecoDeEscrita>, crate::arvore_processo::ErroCienciaNaArvore>,
    pub(crate) lista_ciencias:
        Result<Option<EnderecoDeEscrita>, crate::arvore_processo::ErroCienciaNaArvore>,
    pub(crate) anexar: Result<Option<EnderecoDeEscrita>, ErroAnexarNaBarra>,
    /// Atribuição do processo na unidade atual.
    pub(crate) atribuir: Result<Option<EnderecoDeEscrita>, ErroAtribuirNaBarra>,
    /// Ação de registrar andamento, lida da barra do processo.
    pub(crate) andamento: Result<Option<EnderecoDeEscrita>, ErroAndamentoNaBarra>,
    pub(crate) relacionar: Result<Option<EnderecoDeEscrita>, ErroRelacionarNaBarra>,
    pub(crate) acompanhar: Result<Option<EnderecoDeEscrita>, ErroAcompanharNaBarra>,
    /// O cookie que a abertura leu e conferiu: a escrita não o relê.
    pub(crate) cookie: crate::transporte::sessao::CookieLido,
}
