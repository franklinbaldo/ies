//! Recusas tipadas da derivação e do envio de formulário (uma por fato).
use crate::html_atributos::ErroAtributoHtml;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroSubmissaoFormulario {
    EfeitoDaOperacaoNaoAvaliado,
    IdentificacaoDaUnidadeAusenteNaTela,
    IdentificacoesDaUnidadeRepetidasNaTela,
    UnidadeDaTelaDiferenteDaSolicitada,
    FormularioAusenteNaTela,
    FormulariosComMesmoIdentificadorNaTela,
    MetodoDoFormularioDiferenteDePost,
    EnctypeDoFormularioNaoUrlencoded,
    AcceptCharsetPublicadoNoFormulario,
    AcaoDoFormularioAusente,
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
    BotaoDeEnvioAusenteNoFormulario,
    BotoesDeEnvioRepetidosNoFormulario,
    TelaDeLoginRecebidaAposEnvio,
    MarcadorDeResultadoAusenteNaResposta,
    /// HTTP 200 com `#txaInfraValidacao` não vazio: o SEI recusou o envio
    /// pela validação da Infra. Status 200 e a mensagem saneada.
    ValidacaoRecusadaPeloSei(StatusHttpRecusado),
    /// Tag de abertura servida que o tokenizador de atributos recusou.
    AtributoHtml(ErroAtributoHtml),
    Transporte(FalhaTransporte),
}

impl ErroSubmissaoFormulario {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::EfeitoDaOperacaoNaoAvaliado => "efeito_da_operacao_nao_avaliado",
            Self::IdentificacaoDaUnidadeAusenteNaTela => "identificacao_da_unidade_ausente_na_tela",
            Self::IdentificacoesDaUnidadeRepetidasNaTela => {
                "identificacoes_da_unidade_repetidas_na_tela"
            }
            Self::UnidadeDaTelaDiferenteDaSolicitada => "unidade_da_tela_diferente_da_solicitada",
            Self::FormularioAusenteNaTela => "formulario_ausente_na_tela",
            Self::FormulariosComMesmoIdentificadorNaTela => {
                "formularios_com_mesmo_identificador_na_tela"
            }
            Self::MetodoDoFormularioDiferenteDePost => "metodo_do_formulario_diferente_de_post",
            Self::EnctypeDoFormularioNaoUrlencoded => "enctype_do_formulario_nao_urlencoded",
            Self::AcceptCharsetPublicadoNoFormulario => "accept_charset_publicado_no_formulario",
            Self::AcaoDoFormularioAusente => "acao_do_formulario_ausente",
            Self::AcaoDoFormularioNaoEhUrlHttps => "acao_do_formulario_nao_eh_url_https",
            Self::AcaoDoFormularioForaDaInstalacao => "acao_do_formulario_fora_da_instalacao",
            Self::AcaoDoFormularioDiferenteDaOperacao => "acao_do_formulario_diferente_da_operacao",
            Self::AcaoDoFormularioSemInfraHashUnico => "acao_do_formulario_sem_infra_hash_unico",
            Self::EntidadeHtmlDesconhecidaNoAtributo { .. } => {
                "entidade_html_desconhecida_no_atributo"
            }
            Self::CampoOcultoAusenteNoFormulario { .. } => "campo_oculto_ausente_no_formulario",
            Self::CampoOcultoRepetidoNoFormulario { .. } => "campo_oculto_repetido_no_formulario",
            Self::HdnAcaoPublicadoSemRegraDaOperacao => "hdn_acao_publicado_sem_regra_da_operacao",
            Self::CampoForaDaListaDaOperacao { .. } => "campo_fora_da_lista_da_operacao",
            Self::CampoNaoPublicadoPeloFormulario { .. } => "campo_nao_publicado_pelo_formulario",
            Self::CampoInformadoMaisDeUmaVez { .. } => "campo_informado_mais_de_uma_vez",
            Self::CampoCriadoPeloScriptJaPublicado { .. } => {
                "campo_criado_pelo_script_ja_publicado"
            }
            Self::ValorNaoRepresentavelNaCodificacao { .. } => {
                "valor_nao_representavel_na_codificacao"
            }
            Self::BotaoDeEnvioAusenteNoFormulario => "botao_de_envio_ausente_no_formulario",
            Self::BotoesDeEnvioRepetidosNoFormulario => "botoes_de_envio_repetidos_no_formulario",
            Self::TelaDeLoginRecebidaAposEnvio => "tela_de_login_recebida_apos_envio",
            Self::MarcadorDeResultadoAusenteNaResposta => {
                "marcador_de_resultado_ausente_na_resposta"
            }
            Self::ValidacaoRecusadaPeloSei(_) => "validacao_recusada_pelo_sei",
            Self::AtributoHtml(falha) => falha.subcodigo(),
            Self::Transporte(falha) => falha.subcodigo(),
        }
    }

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
