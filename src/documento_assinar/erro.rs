//! Resultado e recusas de `ies documento assinar`
//! (contracts/cli/documento-assinar.md). Uma variante por fato.
use crate::arvore_processo::{Assinante, ErroAssinaturaNaArvore, ErroLinkDeAssinar};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_obter::ErroDocumentoObter;
use crate::sessao_login::ErroLogin;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

/// Recusas da tela "Assinatura de Documento", antes do POST.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroTelaDeAssinatura {
    HtmlNaoInterpretado,
    FormularioDeAssinaturaAusente,
    CampoNaoPrevistoNaTelaDeAssinatura,
    TelaDeAssinaturaDeOutroDocumento,
    AssinanteNaoPublicadoNaTela,
    OrgaoDoAssinanteNaoSelecionado,
    CampoDeSenhaAusenteNaTela,
}

impl ErroTelaDeAssinatura {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlNaoInterpretado => "tela_de_assinatura_nao_interpretada",
            Self::FormularioDeAssinaturaAusente => "formulario_de_assinatura_ausente",
            Self::CampoNaoPrevistoNaTelaDeAssinatura => "campo_nao_previsto_na_tela_de_assinatura",
            Self::TelaDeAssinaturaDeOutroDocumento => "tela_de_assinatura_de_outro_documento",
            Self::AssinanteNaoPublicadoNaTela => "assinante_nao_publicado_na_tela",
            Self::OrgaoDoAssinanteNaoSelecionado => "orgao_do_assinante_nao_selecionado",
            Self::CampoDeSenhaAusenteNaTela => "campo_de_senha_ausente_na_tela",
        }
    }
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
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Envio(erro) => erro.subcodigo(),
            Self::Redirecionamento => "redirecionamento_apos_assinar",
            Self::RespostaSemFinalizacao => "resposta_da_assinatura_sem_finalizacao",
        }
    }

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
    ArvoreNaoLida(ErroAssinaturaNaArvore),
    LeituraFalhou(ErroDocumentoObter),
}

impl VerificacaoDaAssinatura {
    pub(crate) fn motivo(&self) -> Option<&'static str> {
        match self {
            Self::Lida(_) => None,
            Self::ArvoreNaoLida(erro) => Some(erro.subcodigo()),
            Self::LeituraFalhou(erro) => Some(erro.subcodigo()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroAssinar {
    /// A abertura do documento (a de `ies documento DOC`) falhou.
    Abertura(ErroDocumentoObter),
    /// Credenciais (`IES_CREDENCIAIS_FILE`) ilegíveis: nada foi pedido.
    Credenciais(ErroLogin),
    Arvore(ErroAssinaturaNaArvore),
    AssinaturaNaoPublicadaNaBarra,
    EnderecoDaTelaDoDocumentoNaoReconhecido,
    /// GET da tela do documento ou da de assinatura, antes da escrita.
    TelaDaEscrita(FalhaTransporte),
    TelaDaEscritaRedirecionada,
    SessaoEncerrada,
    LinkDeAssinar(ErroLinkDeAssinar),
    TelaDeAssinatura(ErroTelaDeAssinatura),
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
    Formulario(ErroSubmissaoFormulario),
    /// A leitura depois do POST mostra as mesmas assinaturas de antes.
    AssinaturaNaoAplicada {
        antes: Vec<Assinante>,
        falha_do_envio: Option<FalhaDoEnvio>,
    },
    /// Saída 3: o POST saiu e a leitura não decide o efeito.
    EfeitoIncertoNaAssinatura {
        antes: Vec<Assinante>,
        falha_do_envio: Option<FalhaDoEnvio>,
        verificacao: Box<VerificacaoDaAssinatura>,
    },
}

impl ErroAssinar {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Abertura(erro) => erro.subcodigo(),
            Self::Credenciais(erro) => erro.subcodigo(),
            Self::Arvore(erro) => erro.subcodigo(),
            Self::AssinaturaNaoPublicadaNaBarra => "assinatura_nao_publicada_na_barra",
            Self::EnderecoDaTelaDoDocumentoNaoReconhecido => {
                "endereco_da_tela_do_documento_nao_reconhecido"
            }
            Self::TelaDaEscrita(falha) => falha.subcodigo(),
            Self::TelaDaEscritaRedirecionada => "tela_da_assinatura_redirecionada",
            Self::SessaoEncerrada => "sessao_encerrada_rode_ies_login",
            Self::LinkDeAssinar(erro) => erro.subcodigo(),
            Self::TelaDeAssinatura(erro) => erro.subcodigo(),
            Self::CharsetDaTelaNaoSuportado => "charset_da_tela_de_assinatura_nao_suportado",
            Self::CargoNaoOferecidoPelaTela { .. } => "cargo_nao_oferecido_pela_tela",
            Self::DocumentoAssinadoPorOutroUsuario { .. } => "documento_assinado_por_outro_usuario",
            Self::Formulario(erro) => erro.subcodigo(),
            Self::AssinaturaNaoAplicada { .. } => "assinatura_nao_aplicada",
            Self::EfeitoIncertoNaAssinatura { .. } => "efeito_incerto_na_assinatura",
        }
    }

    /// Categoria pública: o próximo passo de quem chama.
    pub(crate) fn codigo(&self) -> &'static str {
        match self {
            Self::AssinaturaNaoAplicada { .. } => "escrita_nao_aplicada",
            Self::EfeitoIncertoNaAssinatura { .. } => "efeito_incerto",
            _ => "fonte_invalida",
        }
    }

    /// 3 só para efeito incerto; as demais recusas saem com 1.
    pub(crate) fn saida(&self) -> u8 {
        match self {
            Self::EfeitoIncertoNaAssinatura { .. } => 3,
            _ => 1,
        }
    }

    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Abertura(erro) => erro.status_http().cloned(),
            Self::TelaDaEscrita(falha) => falha.status_http().cloned(),
            Self::TelaDaEscritaRedirecionada => Some(StatusHttpRecusado::sem_mensagem(302)),
            Self::Formulario(erro) => erro.status_http().cloned(),
            Self::AssinaturaNaoAplicada { falha_do_envio, .. }
            | Self::EfeitoIncertoNaAssinatura { falha_do_envio, .. } => {
                falha_do_envio.as_ref().and_then(FalhaDoEnvio::status_http)
            }
            _ => None,
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Abertura(erro) => erro.mensagem(),
            Self::Credenciais(erro) => erro.mensagem(),
            Self::Arvore(_) => {
                "a árvore não mostra a assinatura do documento na forma observada; nada foi assinado"
            }
            Self::AssinaturaNaoPublicadaNaBarra => {
                "a barra do documento não oferece assinar nesta sessão; nada foi assinado"
            }
            Self::EnderecoDaTelaDoDocumentoNaoReconhecido => {
                "a árvore não publica a tela do documento na forma observada; nada foi assinado"
            }
            Self::TelaDaEscrita(_) | Self::TelaDaEscritaRedirecionada => {
                "a tela da assinatura não foi servida; nada foi assinado"
            }
            Self::SessaoEncerrada => "a sessão encerrou; rode `ies login` (nada foi assinado)",
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
            Self::EfeitoIncertoNaAssinatura { .. } => {
                "não se sabe se o documento foi assinado; confira a árvore antes de repetir"
            }
        }
    }
}
