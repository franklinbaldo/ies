//! Resultado e recusas de `ies documento assinar`
//! (contracts/cli/documento-assinar.md). Um erro por fato.
use crate::arvore_processo::{Assinante, ErroAssinaturaNaArvore, ErroLinkDeAssinar};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_obter::ErroDocumentoObter;
use crate::sessao_login::ErroLogin;
use crate::transporte::FalhaTransporte;

/// Recusas da tela "Assinatura de Documento", antes do POST.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTelaDeAssinatura {
    TelaDeAssinaturaDeOutroDocumento,
    AssinanteNaoPublicadoNaTela,
    OrgaoDoAssinanteNaoSelecionado,
    CampoDeSenhaAusenteNaTela,
    CampoNaoPrevistoNaTelaDeAssinatura,
    FormularioDeAssinaturaAusente,
    TelaDeAssinaturaNaoInterpretada,
    TelaDeAssinaturaComAtributoIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
}

/// Por que a resposta do POST não é a esperada. O POST saiu (ou pode ter
/// saído): o efeito é decidido pela leitura.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FalhaDoEnvio {
    /// Rede, tempo, status, MIME, charset, login, validação da Infra.
    Envio(ErroSubmissaoFormulario),
    /// 302 (não seguido) em vez da tela.
    Redirecionamento,
    /// HTML 200 sem `finalizar()` no `inicializar()`.
    RespostaSemFinalizacao,
}

/// O que a leitura de verificação, feita uma vez, mostrou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VerificacaoDaAssinatura {
    Lida(Vec<Assinante>),
    ArvoreNaoLida(ErroAssinaturaNaArvore),
    LeituraFalhou(ErroDocumentoObter),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAssinar {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(#[from] CookieIlegivel),
    /// A abertura do documento (a de `ies documento DOC`) falhou.
    Abertura(#[from] ErroDocumentoObter),
    /// Credenciais (`IES_CREDENCIAIS_FILE`) ilegíveis: nada foi pedido.
    Credenciais(#[from] ErroLogin),
    Arvore(#[source] ErroAssinaturaNaArvore),
    AssinaturaNaoPublicadaNaBarra,
    /// GET da tela do documento ou da de assinatura, antes da escrita.
    TelaDaEscrita(#[from] FalhaTransporte),
    TelaDaEscritaRedirecionada,
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
    LinkDeAssinar(#[from] ErroLinkDeAssinar),
    TelaDeAssinatura(#[from] ErroTelaDeAssinatura),
    CharsetDaTelaNaoSuportado,
    CargoNaoOferecidoPelaTela {
        cargos: Vec<String>,
    },
    /// A caneta já mostra outro assinante: a dica com dois não foi
    /// observada, e a segunda assinatura não poderia ser conferida.
    DocumentoAssinadoPorOutroUsuario {
        antes: Vec<Assinante>,
    },
    /// Derivação do POST ou falha antes de ele sair (cookie).
    Formulario(#[from] ErroSubmissaoFormulario),
    /// A leitura depois do POST mostra as mesmas assinaturas de antes.
    AssinaturaNaoAplicada {
        antes: Vec<Assinante>,
        falha_do_envio: Option<FalhaDoEnvio>,
    },
    /// Saída 3: o POST saiu e a leitura não decide o efeito.
    NaoConfirmadoNaAssinatura {
        antes: Vec<Assinante>,
        falha_do_envio: Option<FalhaDoEnvio>,
        verificacao: Box<VerificacaoDaAssinatura>,
    },
    EnderecoDaTelaDoDocumentoNaoReconhecido,
}
