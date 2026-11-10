//! Recusas tipadas da derivação e do envio de formulário (uma por fato).
use crate::erro_html::ErroHtml;
use crate::html_atributos::ErroAtributoHtml;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroSubmissaoFormulario {
    EfeitoDaOperacaoNaoAvaliado,
    UnidadeDaTelaDiferenteDaSolicitada,
    FormulariosComMesmoIdentificadorNaTela,
    MetodoDoFormularioDiferenteDePost,
    EnctypeDoFormularioNaoUrlencoded,
    AcceptCharsetPublicadoNoFormulario,
    AcaoDoFormularioNaoEhUrlHttps,
    AcaoDoFormularioForaDaInstalacao,
    AcaoDoFormularioDiferenteDaOperacao,
    AcaoDoFormularioSemInfraHashUnico,
    EntidadeHtmlDesconhecidaNoAtributo {
        campo: String,
    },
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
    ValidacaoRecusadaPeloSei(StatusHttpRecusado),
    /// Tag de abertura servida que o tokenizador de atributos recusou.
    AtributoHtml(#[from] ErroAtributoHtml),
    Transporte(#[from] FalhaTransporte),
    Html(#[from] ErroHtml),
}

impl ErroSubmissaoFormulario {
    /// A sessão do SEI acabou (decide o relogin único): por variante, sem
    /// percorrer a cadeia `source()`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Transporte(erro) => erro.sessao_encerrada(),
            Self::EfeitoDaOperacaoNaoAvaliado
            | Self::UnidadeDaTelaDiferenteDaSolicitada
            | Self::FormulariosComMesmoIdentificadorNaTela
            | Self::MetodoDoFormularioDiferenteDePost
            | Self::EnctypeDoFormularioNaoUrlencoded
            | Self::AcceptCharsetPublicadoNoFormulario
            | Self::AcaoDoFormularioNaoEhUrlHttps
            | Self::AcaoDoFormularioForaDaInstalacao
            | Self::AcaoDoFormularioDiferenteDaOperacao
            | Self::AcaoDoFormularioSemInfraHashUnico
            | Self::EntidadeHtmlDesconhecidaNoAtributo { .. }
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
            | Self::AtributoHtml(_)
            | Self::Html(_) => false,
        }
    }
}

impl ErroSubmissaoFormulario {
    /// Status e mensagem do SEI quando a recusa veio do status HTTP.
    /// Na validação da Infra, status 200 e a mensagem do SEI.
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(falha) => falha.status_http(),
            Self::ValidacaoRecusadaPeloSei(recusa) => Some(recusa),
            _ => None,
        }
    }
}
