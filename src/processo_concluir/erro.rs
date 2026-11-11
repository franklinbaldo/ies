//! Resultado e recusas de `ies processo concluir`
//! (contracts/cli/processo-concluir.md). Um erro por fato.
use crate::arvore_processo::{ErroConcluirNaBarra, ErroFuncaoDeConcluir, MotivoAbertoEm};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_obter::ErroProcessoObter;
use crate::transporte::FalhaTransporte;

/// Recusas da tela "Conclusão de Processo", antes do POST.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTelaDeConclusao {
    TelaDeConclusaoDeOutroProcesso,
    OpcaoSomenteConcluirNaoMarcada,
    TelaDeConclusaoComReaberturaPreenchida,
    CampoNaoPrevistoNaTelaDeConclusao,
    FormularioDeConclusaoAusente(#[source] crate::formulario_web::ErroSubmissaoFormulario),
    TelaDeConclusaoNaoInterpretada(#[source] tl::ParseError),
    TelaDeConclusaoComAtributoIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
}

/// Por que a resposta da escrita não é a esperada. A escrita saiu (ou pode
/// ter saído): o efeito é decidido pela leitura.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaDoEnvio {
    /// POST da tela: rede, tempo, status, MIME, charset, login, validação.
    Envio(#[from] ErroSubmissaoFormulario),
    /// GET da função: rede, tempo, status, MIME, charset, tamanho.
    Transporte(#[from] FalhaTransporte),
    /// GET da função: a tela de login.
    LoginAposEscrita,
    /// 302 para destino fora dos observados (`arvore_visualizar`, `principal`).
    RedirecionamentoNaoReconhecido,
    /// HTML 200 em vez do 302 (forma `tela`).
    RespostaSemRedirecionamento,
    /// 302 em vez do HTML 200 (forma `funcao`).
    RedirecionamentoNaFuncao,
    /// 302 em vez do HTML 200 (GET de reabrir).
    RedirecionamentoNaReabertura,
    /// POST de enviar: 302 para destino fora do observado (`arvore_visualizar`).
    RedirecionamentoNaoReconhecidoAposEnviar,
    /// POST de enviar: HTML 200 em vez do 302.
    RespostaDoEnvioSemRedirecionamento,
}

/// O que a leitura de verificação, feita uma vez, mostrou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VerificacaoPorLeitura {
    /// Siglas de `aberto_em`.
    Lida(Vec<String>),
    SituacaoNaoLida(MotivoAbertoEm),
    LeituraFalhou(ErroProcessoObter),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroConcluir {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(#[from] CookieIlegivel),
    /// A abertura do processo (a de `ies processo NUP`) falhou.
    Abertura(#[from] ErroProcessoObter),
    /// A abertura não publicou a unidade da sessão (não ocorre sem o passo 1).
    UnidadeDaSessaoNaoPublicada,
    SituacaoDoProcessoNaoLidaAntesDeConcluir {
        motivo: MotivoAbertoEm,
    },
    /// A barra oferece concluir, mas `aberto_em` não lista a unidade.
    UnidadeDaSessaoAusenteDaSituacao {
        aberto_em: Vec<String>,
    },
    ConclusaoNaoPublicadaNaBarra {
        aberto_em: Vec<String>,
    },
    Barra(#[from] ErroConcluirNaBarra),
    /// GET da tela de conclusão ou da tela do nó, antes da escrita.
    TelaDaEscrita(#[from] FalhaTransporte),
    TelaDaEscritaRedirecionada,
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
    CharsetDaTelaNaoSuportado,
    TelaDeConclusao(#[from] ErroTelaDeConclusao),
    FuncaoDeConcluir(#[from] ErroFuncaoDeConcluir),
    /// Derivação do POST ou falha antes de ele sair (cookie).
    Formulario(#[from] ErroSubmissaoFormulario),
    /// A leitura depois da escrita ainda lista a unidade da sessão.
    ConclusaoNaoAplicada {
        aberto_em: Vec<String>,
        depois: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
    },
    /// Saída 3: a escrita saiu e a leitura não decide o efeito.
    NaoConfirmadoNaConclusao {
        aberto_em: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
        verificacao: Box<VerificacaoPorLeitura>,
    },
}
