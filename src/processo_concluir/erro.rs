//! Resultado e recusas de `ies processo concluir`
//! (contracts/cli/processo-concluir.md). Uma variante por fato.
use crate::arvore_processo::{ErroConcluirNaBarra, ErroFuncaoDeConcluir, MotivoAbertoEm};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_obter::ErroProcessoObter;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

/// Recusas da tela "Conclusão de Processo", antes do POST.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroTelaDeConclusao {
    HtmlNaoInterpretado,
    FormularioDeConclusaoAusente,
    CampoNaoPrevistoNaTelaDeConclusao,
    TelaDeConclusaoDeOutroProcesso,
    OpcaoSomenteConcluirNaoMarcada,
    TelaDeConclusaoComReaberturaPreenchida,
}

impl ErroTelaDeConclusao {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlNaoInterpretado => "tela_de_conclusao_nao_interpretada",
            Self::FormularioDeConclusaoAusente => "formulario_de_conclusao_ausente",
            Self::CampoNaoPrevistoNaTelaDeConclusao => "campo_nao_previsto_na_tela_de_conclusao",
            Self::TelaDeConclusaoDeOutroProcesso => "tela_de_conclusao_de_outro_processo",
            Self::OpcaoSomenteConcluirNaoMarcada => "opcao_somente_concluir_nao_marcada",
            Self::TelaDeConclusaoComReaberturaPreenchida => {
                "tela_de_conclusao_com_reabertura_preenchida"
            }
        }
    }
}

/// Por que a resposta da escrita não é a esperada. A escrita saiu (ou pode
/// ter saído): o efeito é decidido pela leitura.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FalhaDoEnvio {
    /// POST da tela: rede, tempo, status, MIME, charset, login, validação.
    Envio(ErroSubmissaoFormulario),
    /// GET da função: rede, tempo, status, MIME, charset, tamanho.
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
}

impl FalhaDoEnvio {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Envio(erro) => erro.subcodigo(),
            Self::Transporte(falha) => falha.subcodigo(),
            Self::LoginAposEscrita => "tela_de_login_recebida_apos_envio",
            Self::RedirecionamentoNaoReconhecido => {
                "redirecionamento_nao_reconhecido_apos_concluir"
            }
            Self::RespostaSemRedirecionamento => "resposta_do_concluir_sem_redirecionamento",
            Self::RedirecionamentoNaFuncao => "redirecionamento_na_conclusao_pela_funcao",
            Self::RedirecionamentoNaReabertura => "redirecionamento_na_reabertura",
        }
    }

    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Envio(erro) => erro.status_http().cloned(),
            Self::Transporte(falha) => falha.status_http().cloned(),
            Self::RedirecionamentoNaoReconhecido
            | Self::RedirecionamentoNaFuncao
            | Self::RedirecionamentoNaReabertura => Some(StatusHttpRecusado::sem_mensagem(302)),
            Self::LoginAposEscrita | Self::RespostaSemRedirecionamento => None,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroConcluir {
    /// A abertura do processo (a de `ies processo NUP`) falhou.
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
    Barra(ErroConcluirNaBarra),
    /// GET da tela de conclusão ou da tela do nó, antes da escrita.
    TelaDaEscrita(FalhaTransporte),
    TelaDaEscritaRedirecionada,
    SessaoEncerrada,
    CharsetDaTelaNaoSuportado,
    TelaDeConclusao(ErroTelaDeConclusao),
    FuncaoDeConcluir(ErroFuncaoDeConcluir),
    /// Derivação do POST ou falha antes de ele sair (cookie).
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
        match self {
            Self::Abertura(erro) => erro.subcodigo(),
            Self::UnidadeDaSessaoNaoPublicada => "unidade_da_sessao_nao_publicada_na_abertura",
            Self::SituacaoDoProcessoNaoLidaAntesDeConcluir { .. } => {
                "situacao_do_processo_nao_lida_antes_de_concluir"
            }
            Self::UnidadeDaSessaoAusenteDaSituacao { .. } => {
                "unidade_da_sessao_ausente_da_situacao"
            }
            Self::ConclusaoNaoPublicadaNaBarra { .. } => "conclusao_nao_publicada_na_barra",
            Self::Barra(erro) => erro.subcodigo(),
            Self::TelaDaEscrita(falha) => falha.subcodigo(),
            Self::TelaDaEscritaRedirecionada => "tela_da_conclusao_redirecionada",
            Self::SessaoEncerrada => "sessao_encerrada_rode_ies_login",
            Self::CharsetDaTelaNaoSuportado => "charset_da_tela_de_conclusao_nao_suportado",
            Self::TelaDeConclusao(erro) => erro.subcodigo(),
            Self::FuncaoDeConcluir(erro) => erro.subcodigo(),
            Self::Formulario(erro) => erro.subcodigo(),
            Self::ConclusaoNaoAplicada { .. } => "conclusao_nao_aplicada",
            Self::EfeitoIncertoNaConclusao { .. } => "efeito_incerto_na_conclusao",
        }
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
