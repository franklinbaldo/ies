//! Resultado e recusas de `ies processo concluir`
//! (contracts/cli/processo-concluir.md). Uma variante por fato.
use crate::arvore_processo::{ErroConcluirNaBarra, ErroFuncaoDeConcluir, MotivoAbertoEm};
use crate::erro_html::ErroHtml;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_obter::ErroProcessoObter;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

/// Recusas da tela "Conclusão de Processo", antes do POST.
#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroTelaDeConclusao {
    TelaDeConclusaoDeOutroProcesso,
    OpcaoSomenteConcluirNaoMarcada,
    TelaDeConclusaoComReaberturaPreenchida,
    #[causa(transparente)]
    Html(ErroHtml),
}

impl ErroTelaDeConclusao {}

/// Por que a resposta da escrita não é a esperada. A escrita saiu (ou pode
/// ter saído): o efeito é decidido pela leitura.
#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum FalhaDoEnvio {
    /// POST da tela: rede, tempo, status, MIME, charset, login, validação.
    #[causa(interna)]
    Envio(ErroSubmissaoFormulario),
    /// GET da função: rede, tempo, status, MIME, charset, tamanho.
    #[causa(interna)]
    Transporte(FalhaTransporte),
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

impl FalhaDoEnvio {
    pub(crate) fn subcodigo(&self) -> &'static str {
        crate::erro_publico::folha(self)
    }

    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Envio(erro) => erro.status_http().cloned(),
            Self::Transporte(falha) => falha.status_http().cloned(),
            Self::RedirecionamentoNaoReconhecido
            | Self::RedirecionamentoNaFuncao
            | Self::RedirecionamentoNaReabertura
            | Self::RedirecionamentoNaoReconhecidoAposEnviar => {
                Some(StatusHttpRecusado::sem_mensagem(302))
            }
            Self::LoginAposEscrita
            | Self::RespostaSemRedirecionamento
            | Self::RespostaDoEnvioSemRedirecionamento => None,
        }
    }
}

/// O que a leitura de verificação, feita uma vez, mostrou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VerificacaoPorLeitura {
    /// Siglas de `aberto_em`.
    Lida(Vec<String>),
    SituacaoNaoLida(MotivoAbertoEm),
    LeituraFalhou(ErroProcessoObter),
}

impl VerificacaoPorLeitura {
    /// Subcódigo (leitura) ou motivo (situação não lida).
    pub(crate) fn motivo(&self) -> Option<&'static str> {
        match self {
            Self::Lida(_) => None,
            Self::SituacaoNaoLida(motivo) => Some(motivo.como_texto()),
            Self::LeituraFalhou(erro) => Some(erro.subcodigo()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroConcluir {
    /// A abertura do processo (a de `ies processo NUP`) falhou.
    #[causa(interna)]
    Abertura(ErroProcessoObter),
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
    #[causa(interna)]
    Barra(ErroConcluirNaBarra),
    /// GET da tela de conclusão ou da tela do nó, antes da escrita.
    #[causa(interna)]
    TelaDaEscrita(FalhaTransporte),
    TelaDaEscritaRedirecionada,
    #[causa(sessao_encerrada)]
    SessaoEncerrada,
    CharsetDaTelaNaoSuportado,
    #[causa(interna)]
    TelaDeConclusao(ErroTelaDeConclusao),
    #[causa(interna)]
    FuncaoDeConcluir(ErroFuncaoDeConcluir),
    /// Derivação do POST ou falha antes de ele sair (cookie).
    #[causa(interna)]
    Formulario(ErroSubmissaoFormulario),
    /// A leitura depois da escrita ainda lista a unidade da sessão.
    ConclusaoNaoAplicada {
        aberto_em: Vec<String>,
        depois: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
    },
    /// Saída 3: a escrita saiu e a leitura não decide o efeito.
    EfeitoIncertoNaConclusao {
        aberto_em: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
        verificacao: Box<VerificacaoPorLeitura>,
    },
}

impl ErroConcluir {
    pub(crate) fn subcodigo(&self) -> &'static str {
        crate::erro_publico::folha(self)
    }

    /// Categoria pública: o próximo passo de quem chama.
    pub(crate) fn codigo(&self) -> &'static str {
        match self {
            Self::ConclusaoNaoAplicada { .. } => "escrita_nao_aplicada",
            Self::EfeitoIncertoNaConclusao { .. } => "efeito_incerto",
            _ => "fonte_invalida",
        }
    }

    /// 3 só para efeito incerto; as demais recusas saem com 1.
    pub(crate) fn saida(&self) -> u8 {
        match self {
            Self::EfeitoIncertoNaConclusao { .. } => 3,
            _ => 1,
        }
    }

    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Abertura(erro) => erro.status_http().cloned(),
            Self::TelaDaEscrita(falha) => falha.status_http().cloned(),
            Self::TelaDaEscritaRedirecionada => Some(StatusHttpRecusado::sem_mensagem(302)),
            Self::Formulario(erro) => erro.status_http().cloned(),
            Self::ConclusaoNaoAplicada { falha_do_envio, .. }
            | Self::EfeitoIncertoNaConclusao { falha_do_envio, .. } => {
                falha_do_envio.as_ref().and_then(FalhaDoEnvio::status_http)
            }
            _ => None,
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Abertura(erro) => erro.mensagem(),
            Self::UnidadeDaSessaoNaoPublicada => {
                "a abertura não publicou a unidade da sessão; nada foi concluído"
            }
            Self::SituacaoDoProcessoNaoLidaAntesDeConcluir { .. } => {
                "a árvore não publica onde o processo está aberto, e a conclusão não poderia ser conferida; nada foi concluído"
            }
            Self::UnidadeDaSessaoAusenteDaSituacao { .. } => {
                "a barra oferece concluir, mas a situação não lista a unidade da sessão; nada foi concluído"
            }
            Self::ConclusaoNaoPublicadaNaBarra { .. } => {
                "o processo está aberto na unidade, mas a barra não oferece concluir; nada foi concluído"
            }
            Self::Barra(_) => {
                "a ação de concluir da barra não está na forma observada; nada foi concluído"
            }
            Self::TelaDaEscrita(_) | Self::TelaDaEscritaRedirecionada => {
                "a tela da conclusão não foi servida; nada foi concluído"
            }
            Self::SessaoEncerrada => "a sessão encerrou; rode `ies login` (nada foi concluído)",
            Self::CharsetDaTelaNaoSuportado => {
                "a tela de conclusão declara um charset não suportado; nada foi concluído"
            }
            Self::TelaDeConclusao(_) => {
                "a tela Conclusão de Processo não está na forma observada; nada foi enviado"
            }
            Self::FuncaoDeConcluir(_) => {
                "a função concluirProcesso() da tela do nó não está na forma observada; nada foi enviado"
            }
            Self::Formulario(_) => "o envio da conclusão não pôde ser montado; nada foi enviado",
            Self::ConclusaoNaoAplicada { .. } => {
                "a árvore lida depois de concluir ainda lista a unidade da sessão: o processo não foi concluído"
            }
            Self::EfeitoIncertoNaConclusao { .. } => {
                "não se sabe se o processo foi concluído; confira com `ies processo NUP` antes de repetir"
            }
        }
    }
}
