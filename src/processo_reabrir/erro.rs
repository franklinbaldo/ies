//! Recusas de `ies processo reabrir` (contracts/cli/processo-reabrir.md).
//! Um erro por fato; a resposta não esperada da escrita é a
//! `FalhaDoEnvio` de `processo concluir`, e a verificação, a mesma leitura.
use crate::arvore_processo::{ErroEnderecoDeReabrir, MotivoAbertoEm};
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura};
use crate::processo_obter::ErroProcessoObter;
use crate::transporte::FalhaTransporte;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroReabrir {
    /// A abertura do processo (a de `ies processo NUP`) falhou.
    Abertura(#[from] ErroProcessoObter),
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
    Barra(#[from] crate::arvore_processo::ErroReabrirNaBarra),
    /// GET da tela do nó, antes da escrita.
    TelaDoNo(#[from] FalhaTransporte),
    TelaDoNoRedirecionada,
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
    EnderecoDeReabrir(#[from] ErroEnderecoDeReabrir),
    /// A leitura depois da escrita não lista a unidade da sessão.
    ReaberturaNaoAplicada {
        aberto_em: Vec<String>,
        depois: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
    },
    /// Saída 3: a escrita saiu e a leitura não decide o efeito.
    NaoConfirmadoNaReabertura {
        aberto_em: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
        verificacao: Box<VerificacaoPorLeitura>,
    },
}
