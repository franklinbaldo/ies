//! Recusas de `ies processo reabrir` (contracts/cli/processo-reabrir.md).
//! Uma variante por fato; a resposta não esperada da escrita é a
//! `FalhaDoEnvio` de `processo concluir`, e a verificação, a mesma leitura.
use crate::arvore_processo::{ErroEnderecoDeReabrir, MotivoAbertoEm};
use crate::erro_html::ErroHtml;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura};
use crate::processo_obter::ErroProcessoObter;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroReabrir {
    /// A abertura do processo (a de `ies processo NUP`) falhou.
    #[causa(interna)]
    Abertura(ErroProcessoObter),
    /// A abertura não publicou a unidade da sessão (não ocorre sem o passo 1).
    UnidadeDaSessaoNaoPublicada,
    SituacaoDoProcessoNaoLidaAntesDeReabrir {
        motivo: MotivoAbertoEm,
    },
    /// A barra oferece reabrir, mas `aberto_em` já lista a unidade.
    UnidadeDaSessaoJaNaSituacao {
        aberto_em: Vec<String>,
    },
    /// Fora da unidade, e a barra não oferece reabrir.
    ReaberturaNaoPublicadaNaBarra {
        aberto_em: Vec<String>,
    },
    #[causa(interna)]
    Barra(ErroHtml),
    /// O cookie não pôde ser lido antes da tela do nó.
    #[causa(interna)]
    Cookie(CookieIlegivel),
    /// GET da tela do nó, antes da escrita.
    #[causa(interna)]
    TelaDoNo(FalhaTransporte),
    TelaDoNoRedirecionada,
    #[causa(sessao_encerrada)]
    SessaoEncerrada,
    #[causa(interna)]
    EnderecoDeReabrir(ErroEnderecoDeReabrir),
    /// A leitura depois da escrita não lista a unidade da sessão.
    ReaberturaNaoAplicada {
        aberto_em: Vec<String>,
        depois: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
    },
    /// Saída 3: a escrita saiu e a leitura não decide o efeito.
    EfeitoIncertoNaReabertura {
        aberto_em: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
        verificacao: Box<VerificacaoPorLeitura>,
    },
}

impl ErroReabrir {
    /// Categoria pública: o próximo passo de quem chama.
    pub(crate) fn codigo(&self) -> &'static str {
        match self {
            Self::ReaberturaNaoAplicada { .. } => "escrita_nao_aplicada",
            Self::EfeitoIncertoNaReabertura { .. } => "efeito_incerto",
            _ => "fonte_invalida",
        }
    }

    /// 3 só para efeito incerto; as demais recusas saem com 1.
    pub(crate) fn saida(&self) -> u8 {
        match self {
            Self::EfeitoIncertoNaReabertura { .. } => 3,
            _ => 1,
        }
    }

    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Abertura(erro) => erro.status_http().cloned(),
            Self::Cookie(falha) => falha.status_http().cloned(),
            Self::TelaDoNo(falha) => falha.status_http().cloned(),
            Self::TelaDoNoRedirecionada => Some(StatusHttpRecusado::sem_mensagem(302)),
            Self::ReaberturaNaoAplicada { falha_do_envio, .. }
            | Self::EfeitoIncertoNaReabertura { falha_do_envio, .. } => {
                falha_do_envio.as_ref().and_then(FalhaDoEnvio::status_http)
            }
            _ => None,
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Abertura(erro) => erro.mensagem(),
            Self::UnidadeDaSessaoNaoPublicada => {
                "a abertura não publicou a unidade da sessão; nada foi reaberto"
            }
            Self::SituacaoDoProcessoNaoLidaAntesDeReabrir { .. } => {
                "a árvore não publica onde o processo está aberto, e a reabertura não poderia ser conferida; nada foi reaberto"
            }
            Self::UnidadeDaSessaoJaNaSituacao { .. } => {
                "a barra oferece reabrir, mas a situação já lista a unidade da sessão; nada foi reaberto"
            }
            Self::ReaberturaNaoPublicadaNaBarra { .. } => {
                "o processo não está aberto na unidade, e a barra não oferece reabrir; nada foi reaberto"
            }
            Self::Barra(_) => {
                "a ação de reabrir da barra não está na forma observada; nada foi reaberto"
            }
            Self::Cookie(_) => "o cookie da sessão não pôde ser lido; nada foi reaberto",
            Self::TelaDoNo(_) | Self::TelaDoNoRedirecionada => {
                "a tela do nó não foi servida; nada foi reaberto"
            }
            Self::SessaoEncerrada => "a sessão encerrou; rode `ies login` (nada foi reaberto)",
            Self::EnderecoDeReabrir(_) => {
                "a tela do nó não publica a URL de reabrir na forma observada; nada foi enviado"
            }
            Self::ReaberturaNaoAplicada { .. } => {
                "a árvore lida depois de reabrir não lista a unidade da sessão: o processo não foi reaberto"
            }
            Self::EfeitoIncertoNaReabertura { .. } => {
                "não se sabe se o processo foi reaberto; confira com `ies processo NUP` antes de repetir"
            }
        }
    }
}
