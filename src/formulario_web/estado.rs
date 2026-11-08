//! Estados de sessão e de processo alterados por formulários Web.
//! Evidência e política permanecem declaradas em cada operação.

/// Estado de sessão que uma operação pode alterar, mesmo quando consulta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDeSessao {
    PaginaOuFiltroLembrado,
    UnidadeAtual,
    PreferenciaDeVisualizacao,
    /// O login cria a sessão autenticada (cookies) no servidor.
    SessaoAutenticada,
}

/// Preferência persistente do usuário, que sobrevive à sessão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreferenciaDoUsuario {
    /// Colunas da visão detalhada (Nível de detalhe).
    ColunasDaVisaoDetalhada,
}

/// Estado do processo no SEI que uma escrita altera.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDoProcesso {
    /// O andamento aberto na unidade da sessão (concluir fecha).
    AndamentoAbertoNaUnidade,
    /// A assinatura do usuário no documento (não se desfaz).
    AssinaturaDoDocumento,
    /// Inclusão de documento interno, provada por releitura da árvore.
    DocumentoCriadoNoProcesso,
    /// O conteúdo de um documento foi editado e publicado em nova versão.
    DocumentoEditado,
    /// Cancelar documento altera o estado permanentemente.
    CancelamentoDoDocumento,
    /// As unidades com o processo aberto (enviar abre no destino e, sem
    /// "manter aberto", fecha na da sessão).
    UnidadesComOProcessoAberto,
    /// Registro de uma linha nova no histórico do processo.
    AndamentoNoHistorico,
    /// Inclusão ou remoção de Acompanhamento Especial na unidade.
    AcompanhamentoEspecial,
    /// Cadastro de novo processo na unidade ativa, com identidade conferida.
    ProcessoCriadoNaUnidade,
    /// Sobrestar ou remover o sobrestamento de um processo.
    Sobrestamento,
    /// Atribuição de um processo a usuário da unidade, ou sua remoção.
    AtribuicaoNaUnidade,
    /// Texto e prioridade da anotação registrada na unidade do processo.
    AnotacaoNaUnidade,
    /// Vínculo entre processos, incluindo sua remoção.
    RelacionamentosDoProcesso,
}
