//! Resultado e recusas de `ies documento assinar`
//! (contracts/cli/documento-assinar.md). Uma variante por fato.
use crate::arvore_processo::{Assinante, ErroLinkDeAssinar};
use crate::erro_html::ErroHtml;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_obter::ErroDocumentoObter;
use crate::sessao_login::ErroLogin;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

/// Recusas da tela "Assinatura de Documento", antes do POST.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTelaDeAssinatura {
    TelaDeAssinaturaDeOutroDocumento,
    AssinanteNaoPublicadoNaTela,
    OrgaoDoAssinanteNaoSelecionado,
    Html(#[from] ErroHtml),
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

impl FalhaDoEnvio {
    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Envio(erro) => erro.status_http().cloned(),
            Self::Redirecionamento => Some(StatusHttpRecusado::sem_mensagem(302)),
            Self::RespostaSemFinalizacao => None,
        }
    }
}

/// O que a leitura de verificação, feita uma vez, mostrou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VerificacaoDaAssinatura {
    Lida(Vec<Assinante>),
    ArvoreNaoLida(ErroHtml),
    LeituraFalhou(ErroDocumentoObter),
}

impl VerificacaoDaAssinatura {}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAssinar {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(#[from] CookieIlegivel),
    /// A abertura do documento (a de `ies documento DOC`) falhou.
    Abertura(#[from] ErroDocumentoObter),
    /// Credenciais (`IES_CREDENCIAIS_FILE`) ilegíveis: nada foi pedido.
    Credenciais(#[from] ErroLogin),
    Arvore(#[source] ErroHtml),
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
    Html(#[source] ErroHtml),
}

impl ErroAssinar {
    /// Categoria pública: o próximo passo de quem chama.
    pub(crate) fn codigo(&self) -> &'static str {
        match self {
            Self::AssinaturaNaoAplicada { .. } => "escrita_nao_aplicada",
            Self::NaoConfirmadoNaAssinatura { .. } => "nao_confirmado",
            _ => "fonte_invalida",
        }
    }

    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Abertura(erro) => erro.status_http().cloned(),
            Self::TelaDaEscrita(falha) => falha.status_http().cloned(),
            Self::TelaDaEscritaRedirecionada => Some(StatusHttpRecusado::sem_mensagem(302)),
            Self::Formulario(erro) => erro.status_http().cloned(),
            Self::AssinaturaNaoAplicada { falha_do_envio, .. }
            | Self::NaoConfirmadoNaAssinatura { falha_do_envio, .. } => {
                falha_do_envio.as_ref().and_then(FalhaDoEnvio::status_http)
            }
            _ => None,
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Cookie(e) => e.mensagem(),
            Self::Html(e) => e.mensagem,
            Self::Abertura(erro) => erro.mensagem(),
            Self::Credenciais(erro) => erro.mensagem(),
            Self::Arvore(_) => {
                "a árvore não mostra a assinatura do documento na forma observada; nada foi assinado"
            }
            Self::AssinaturaNaoPublicadaNaBarra => {
                "a barra do documento não oferece assinar nesta sessão; nada foi assinado"
            }
            Self::TelaDaEscrita(_) | Self::TelaDaEscritaRedirecionada => {
                "a tela da assinatura não foi servida; nada foi assinado"
            }
            Self::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
                "a sessão encerrou; rode `ies login` (nada foi assinado)"
            }
            Self::LinkDeAssinar(_) => {
                "a tela do documento não publica o link de assinar na forma observada; nada foi assinado"
            }
            Self::TelaDeAssinatura(_) => {
                "a tela Assinatura de Documento não está na forma observada; nada foi enviado"
            }
            Self::CharsetDaTelaNaoSuportado => {
                "a tela de assinatura declara um charset não suportado; nada foi enviado"
            }
            Self::CargoNaoOferecidoPelaTela { .. } => {
                "o cargo pedido não está entre as opções da tela (compare com `cargos`); nada foi enviado"
            }
            Self::DocumentoAssinadoPorOutroUsuario { .. } => {
                "o documento já tem assinatura de outro usuário, e a segunda não poderia ser conferida pela árvore; nada foi enviado"
            }
            Self::Formulario(_) => "o envio da assinatura não pôde ser montado; nada foi enviado",
            Self::AssinaturaNaoAplicada { .. } => {
                "a árvore lida depois de assinar mostra as mesmas assinaturas de antes: o documento não foi assinado"
            }
            Self::NaoConfirmadoNaAssinatura { .. } => {
                "não se sabe se o documento foi assinado; confira a árvore antes de repetir"
            }
        }
    }
}
