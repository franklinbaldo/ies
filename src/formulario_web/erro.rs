//! Recusas tipadas da derivação e do envio de formulário (uma por fato).
use crate::transporte::{FalhaTransporte, FatoDaResposta, StatusHttpRecusado};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroSubmissaoFormulario {
    EfeitoDaOperacaoNaoAvaliado,
    UnidadeDaTelaDiferenteDaSolicitada,
    /// A tela exige conferir a unidade e nenhuma foi informada.
    UnidadeNaoInformada,
    FormulariosComMesmoIdentificadorNaTela,
    MetodoDoFormularioDiferenteDePost,
    EnctypeDoFormularioNaoUrlencoded,
    AcceptCharsetPublicadoNoFormulario,
    AcaoDoFormularioNaoEhUrlHttps,
    AcaoDoFormularioForaDaInstalacao,
    AcaoDoFormularioDiferenteDaOperacao,
    AcaoDoFormularioSemInfraHashUnico,
    CampoOcultoAusenteNoFormulario {
        campo: String,
    },
    CampoOcultoRepetidoNoFormulario {
        campo: String,
    },
    HdnAcaoPublicadoSemRegraDaOperacao,
    CampoForaDaListaDaOperacao {
        campo: String,
    },
    CampoNaoPublicadoPeloFormulario {
        campo: String,
    },
    CampoInformadoMaisDeUmaVez {
        campo: String,
    },
    /// O campo que o script da operação cria já é publicado pela tela.
    CampoCriadoPeloScriptJaPublicado {
        campo: String,
    },
    ValorNaoRepresentavelNaCodificacao {
        campo: String,
    },
    TelaDeLoginRecebidaAposEnvio,
    MarcadorDeResultadoAusenteNaResposta,
    /// HTTP 200 com `#txaInfraValidacao` não vazio: o SEI recusou o envio
    /// pela validação da Infra. Status 200 e a mensagem saneada.
    ValidacaoRecusadaPeloSei(String),
    Transporte(#[from] FalhaTransporte),
    /// A consulta da `action` não se lê em pares: a causa é a do leitor.
    ConsultaDaAcaoIlegivel(#[from] crate::consulta_do_controlador::ErroConsultaDoControlador),
    AcaoDoFormularioAusente,
    BotaoDeEnvioAusenteNoFormulario,
    BotoesDeEnvioRepetidosNoFormulario,
    FormularioAusenteNaTela,
    IdentificacaoDaUnidadeAusenteNaTela,
    /// A identificação da unidade publicada na tela repetiu: a causa é a do controle.
    IdentificacoesDaUnidadeRepetidasNaTela(#[source] crate::erro_web::ErroCaixa),
}

impl ErroSubmissaoFormulario {
    /// A sessão do SEI acabou (decide o relogin único): por `match` no tipo, sem
    /// percorrer a cadeia `source()`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Transporte(erro) => erro.sessao_encerrada(),
            Self::EfeitoDaOperacaoNaoAvaliado
            | Self::UnidadeDaTelaDiferenteDaSolicitada
            | Self::UnidadeNaoInformada
            | Self::FormulariosComMesmoIdentificadorNaTela
            | Self::MetodoDoFormularioDiferenteDePost
            | Self::EnctypeDoFormularioNaoUrlencoded
            | Self::AcceptCharsetPublicadoNoFormulario
            | Self::AcaoDoFormularioNaoEhUrlHttps
            | Self::AcaoDoFormularioForaDaInstalacao
            | Self::AcaoDoFormularioDiferenteDaOperacao
            | Self::AcaoDoFormularioSemInfraHashUnico
            | Self::CampoOcultoAusenteNoFormulario { .. }
            | Self::CampoOcultoRepetidoNoFormulario { .. }
            | Self::HdnAcaoPublicadoSemRegraDaOperacao
            | Self::CampoForaDaListaDaOperacao { .. }
            | Self::CampoNaoPublicadoPeloFormulario { .. }
            | Self::CampoInformadoMaisDeUmaVez { .. }
            | Self::CampoCriadoPeloScriptJaPublicado { .. }
            | Self::ValorNaoRepresentavelNaCodificacao { .. }
            | Self::TelaDeLoginRecebidaAposEnvio
            | Self::MarcadorDeResultadoAusenteNaResposta
            | Self::ValidacaoRecusadaPeloSei(_)
            | Self::AcaoDoFormularioAusente
            | Self::BotaoDeEnvioAusenteNoFormulario
            | Self::BotoesDeEnvioRepetidosNoFormulario
            | Self::FormularioAusenteNaTela
            | Self::IdentificacaoDaUnidadeAusenteNaTela
            | Self::IdentificacoesDaUnidadeRepetidasNaTela(_)
            | Self::ConsultaDaAcaoIlegivel(_) => false,
        }
    }
}

impl ErroSubmissaoFormulario {
    /// Status e mensagem do SEI quando a recusa veio do status HTTP.
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(falha) => falha.status_http(),
            Self::EfeitoDaOperacaoNaoAvaliado
            | Self::UnidadeDaTelaDiferenteDaSolicitada
            | Self::UnidadeNaoInformada
            | Self::FormulariosComMesmoIdentificadorNaTela
            | Self::MetodoDoFormularioDiferenteDePost
            | Self::EnctypeDoFormularioNaoUrlencoded
            | Self::AcceptCharsetPublicadoNoFormulario
            | Self::AcaoDoFormularioNaoEhUrlHttps
            | Self::AcaoDoFormularioForaDaInstalacao
            | Self::AcaoDoFormularioDiferenteDaOperacao
            | Self::AcaoDoFormularioSemInfraHashUnico
            | Self::CampoOcultoAusenteNoFormulario { .. }
            | Self::CampoOcultoRepetidoNoFormulario { .. }
            | Self::HdnAcaoPublicadoSemRegraDaOperacao
            | Self::CampoForaDaListaDaOperacao { .. }
            | Self::CampoNaoPublicadoPeloFormulario { .. }
            | Self::CampoInformadoMaisDeUmaVez { .. }
            | Self::CampoCriadoPeloScriptJaPublicado { .. }
            | Self::ValorNaoRepresentavelNaCodificacao { .. }
            | Self::TelaDeLoginRecebidaAposEnvio
            | Self::MarcadorDeResultadoAusenteNaResposta
            | Self::ValidacaoRecusadaPeloSei(_)
            | Self::AcaoDoFormularioAusente
            | Self::BotaoDeEnvioAusenteNoFormulario
            | Self::BotoesDeEnvioRepetidosNoFormulario
            | Self::FormularioAusenteNaTela
            | Self::IdentificacaoDaUnidadeAusenteNaTela
            | Self::IdentificacoesDaUnidadeRepetidasNaTela(_)
            | Self::ConsultaDaAcaoIlegivel(_) => None,
        }
    }
}

impl ErroSubmissaoFormulario {
    /// A resposta do envio não pôde ser lida: a tela de login no lugar da
    /// resposta é sessão perdida no envio; as demais seguem como transporte.
    pub(crate) fn da_resposta(falha: FalhaTransporte) -> Self {
        match falha.fato_da_resposta() {
            FatoDaResposta::TelaDeLoginNoLugarDaResposta => Self::TelaDeLoginRecebidaAposEnvio,
            FatoDaResposta::CorpoAcimaDoLimite | FatoDaResposta::NenhumAlemDoTransporte => {
                Self::Transporte(falha)
            }
        }
    }
}
