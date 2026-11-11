//! Recusas e resultado de `ies processo enviar`
//! (contracts/cli/processo-enviar.md). Um erro por fato.
use crate::arvore_processo::{ErroEnviarNaBarra, MotivoAbertoEm};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura};
use crate::processo_obter::ErroProcessoObter;
use crate::transporte::FalhaTransporte;

/// Recusas da tela "Enviar Processo", antes de qualquer POST.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTelaDeEnvio {
    TelaDeEnvioDeOutroProcesso,
    TelaDeEnvioComOpcoesPreenchidas,
    OrgaoDaTelaDeEnvioNaoSelecionado,
    AutocompletarDeUnidadeNaoDeclarado,
    ConsultaDoAutocompletarForaDaFormaObservada,
    CampoNaoPrevistoNaTelaDeEnvio,
    FormularioDeEnvioAusente(#[source] crate::formulario_web::ErroSubmissaoFormulario),
    TelaDeEnvioNaoInterpretada,
    ParseDaTelaDeEnvioFalhou(#[source] tl::ParseError),
    TelaDeEnvioComAtributoIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
}

/// Por que a consulta do autocomplete não deu a lista de unidades.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaDaConsulta {
    /// Derivação ou cookie: nada saiu.
    Derivacao(#[from] ErroSubmissaoFormulario),
    /// Rede, tempo, status, tamanho, charset.
    Transporte(#[from] FalhaTransporte),
    /// 200 sem `application/xml`/`text/xml` (tela de login, outra página).
    RespostaDoAutocompletarNaoXml,
    RespostaDoAutocompletarForaDaFormaObservada,
    ParseDaRespostaDoAutocompletarFalhou(#[source] tl::ParseError),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroEnviar {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(#[from] CookieIlegivel),
    /// A abertura do processo (a de `ies processo NUP`) falhou.
    Abertura(#[from] ErroProcessoObter),
    UnidadeDaSessaoNaoPublicada,
    SituacaoDoProcessoNaoLidaAntesDeEnviar {
        motivo: MotivoAbertoEm,
    },
    ProcessoNaoAbertoNaUnidadeDaSessao {
        aberto_em: Vec<String>,
    },
    /// A barra oferece enviar, mas `aberto_em` não lista a unidade.
    UnidadeDaSessaoAusenteDaSituacao {
        aberto_em: Vec<String>,
    },
    EnvioNaoPublicadoNaBarra {
        aberto_em: Vec<String>,
    },
    Barra(#[from] ErroEnviarNaBarra),
    TelaDaEscrita(#[from] FalhaTransporte),
    TelaDaEscritaRedirecionada,
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
    CharsetDaTelaNaoSuportado,
    TelaDeEnvio(#[from] ErroTelaDeEnvio),
    Consulta(#[from] FalhaDaConsulta),
    /// O checkbox não foi servido ou está indisponível na tela atual.
    RemocaoDeAnotacoesNaoOferecida,
    /// Confirmação pública fechada enquanto a nova escrita não foi validada em treinamento.
    RemocaoDeAnotacoesSemVerificacao,
    /// O envio foi confirmado pela árvore, mas a anotação ainda existe.
    AnotacoesNaoRemovidasAposEnvio {
        /// A árvore comprovou esta situação mesmo com a anotação persistente.
        depois: Option<Vec<String>>,
    },
    /// A tentativa de limpar foi relida, mas a anotação segue preenchida: não tramitar.
    AnotacaoNaoLimpaAntesDeEnviar,
    /// A limpeza pode ter sido aplicada; não prosseguir nem tentar novamente.
    /// Carrega o que impediu a releitura.
    NaoConfirmadoNaLimpezaDaAnotacao(#[source] Box<ErroEnviar>),
    /// O envio foi confirmado; a releitura da anotação falhou.
    NaoConfirmadoNaRemocaoDeAnotacoes {
        /// Unidades comprovadas pela árvore após envio; indisponível na limpeza pré-envio.
        depois: Option<Vec<String>>,
        /// A razão concreta que impediu a leitura independente.
        #[source]
        causa: Box<ErroEnviar>,
    },
    /// A anotação da origem não foi lida antes de enviar.
    Anotacoes(#[from] super::anotacoes::ErroDasAnotacoes),
    DestinoNaoOferecidoPeloSei {
        oferecidos: Vec<String>,
    },
    VariosDestinosComASigla {
        candidatos: Vec<String>,
    },
    ProcessoJaAbertoNoDestino {
        aberto_em: Vec<String>,
    },
    /// Derivação do POST ou falha antes de ele sair (cookie).
    Formulario(#[from] ErroSubmissaoFormulario),
    /// A anotação foi removida e relida; o formulário de envio falhou
    /// antes de qualquer POST de tramitação. A limpeza não é desfeita.
    FormularioAposLimpezaComprovada {
        aberto_em: Vec<String>,
        #[source]
        causa: ErroSubmissaoFormulario,
    },
    /// A leitura depois do envio não lista o destino.
    EnvioNaoAplicado {
        aberto_em: Vec<String>,
        depois: Vec<String>,
        anotacao_depois: Option<bool>,
        falha_do_envio: Option<FalhaDoEnvio>,
    },
    /// O destino está aberto, mas a unidade da sessão não ficou como pedido.
    SituacaoDaOrigemDiferenteDaPedidaAposEnvio {
        aberto_em: Vec<String>,
        depois: Vec<String>,
        anotacao_depois: Option<bool>,
        falha_do_envio: Option<FalhaDoEnvio>,
    },
    /// Saída 3: o POST saiu e a leitura não decide o efeito.
    NaoConfirmadoNoEnvio {
        aberto_em: Vec<String>,
        anotacao_depois: Option<bool>,
        falha_do_envio: Option<FalhaDoEnvio>,
        verificacao: Box<VerificacaoPorLeitura>,
    },
}
