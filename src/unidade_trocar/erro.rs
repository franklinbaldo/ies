//! Recusas tipadas de `ies unidade trocar` (#205), uma por fato. Antes do
//! POST nada mudou (saída 1); depois dele, todo desvio é efeito incerto
//! (saída 3), com a etapa e o fato.
use super::tela::UnidadeOferecida;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::transporte::FalhaTransporte;

/// Onde, depois do POST enviado, a sequência observada se desfez.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EtapaDaTroca {
    Envio,
    PrimeiroRedirecionamento,
    SegundoRedirecionamento,
    ControleDaUnidadeNova,
}

/// O fato observado depois do POST, sem URL nem corpo.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaDepoisDoEnvio {
    Envio(#[from] ErroSubmissaoFormulario),
    Transporte(#[from] FalhaTransporte),
    /// HTML 200 onde o observado é um 302.
    RespostaSemRedirecionamento,
    /// 302 sem `Location`.
    RedirecionamentoSemLocalizacao,
    /// `Location` fora do host, da pasta ou da forma observada.
    LocalizacaoForaDaFormaObservada,
    /// `Location` para outra unidade que a escolhida.
    LocalizacaoDeOutraUnidade,
    /// Status fora de 200 e 302 (recusa do transporte com o status).
    Status(i32),
    ControleNaoReconhecido(#[from] ErroCaixa),
    /// O Controle novo mostra outra sigla que a pedida.
    UnidadeDoControleDiferenteDaPedida,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTroca {
    /// GET do Controle ou da tela de troca, antes do POST.
    Transporte(#[from] FalhaTransporte),
    ControleNaoReconhecido(#[from] ErroCaixa),
    UnidadeExibidaDiferenteDaSolicitada,
    LinksDeTrocaDivergentes,
    ListaDeUnidadesIncompleta,
    UnidadeMarcadaDiferenteDaAtiva,
    SelecaoDeUnidadePorScriptNaoPublicada,
    UnidadeNaoOferecida {
        oferecidas: Vec<UnidadeOferecida>,
    },
    SiglaDeUnidadeAmbigua {
        candidatas: Vec<UnidadeOferecida>,
    },
    Submissao(#[from] ErroSubmissaoFormulario),
    NaoConfirmado {
        etapa: EtapaDaTroca,
        falha: FalhaDepoisDoEnvio,
    },
    LinhaDeUnidadeForaDaFormaObservada,
    AtributoDaLinhaDeUnidadeIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
    LinkDeTrocaDeUnidadeAusente,
    LinkDeTrocaDeUnidadeForaDaFormaObservada,
    AtributoDoLinkDeTrocaIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
    TelaDeTrocaDeUnidadeNaoReconhecida,
    AtributoDaTelaDeTrocaIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
}
