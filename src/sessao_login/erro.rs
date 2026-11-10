//! Recusas tipadas de `ies login`, uma por fato verificável. A mensagem é
//! apresentação; o subcódigo é a classificação. Nenhuma variante carrega
//! credencial, cookie, URL assinada ou HTML.
use crate::configuracao::{ArquivoPrivado, ErroConfiguracao};
use crate::erro_html::ErroHtml;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

/// Campo do arquivo de credenciais.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CampoDasCredenciais {
    Usuario,
    Senha,
    Orgao,
}

impl CampoDasCredenciais {
    pub(crate) fn chave(self) -> &'static str {
        match self {
            Self::Usuario => "usuario",
            Self::Senha => "senha",
            Self::Orgao => "orgao",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLogin {
    Configuracao(#[from] ErroConfiguracao),
    /// Origem da instalação ilegível ou fora do formato `Instalacao`.
    Origem(#[from] super::fluxo::OrigemIlegivel),
    CaminhoDasCredenciaisNaoInformado,
    ArquivoDeCredenciais(#[from] super::credenciais::CredenciaisIlegiveis),
    CampoDasCredenciaisAusente(CampoDasCredenciais),
    CampoDasCredenciaisRepetido(CampoDasCredenciais),
    CampoDasCredenciaisVazio(CampoDasCredenciais),
    LinhaDasCredenciaisSemCampoConhecido,
    LoginEmAndamentoEmOutroProcesso,
    ArquivoDeLockDoLoginInacessivel,
    /// URL ou cookie gravados que o transporte recusa antes da rede.
    ArquivoDaSessao(#[from] super::fluxo::SessaoGravadaInvalida),
    /// O GET da sessão gravada não mostrou um Controle reconhecível.
    ControleDaSessaoExistenteNaoReconhecido(ErroCaixa),
    RedirecionamentoSemLocalizacao,
    LocalizacaoDeRedirecionamentoNaoInterpretavel,
    RedirecionamentoParaForaDaInstalacao,
    RedirecionamentosAcimaDoLimite,
    /// 307/308 depois do POST: seguir repetiria as credenciais.
    RedirecionamentoQueRepetiriaOEnvioDoLogin,
    CookieRecebidoForaDoFormatoAceito,
    CharsetDaTelaDeLoginNaoSuportado,
    CaptchaExigido,
    SegundoFatorExigido,
    RegraDoOnsubmitDoLoginNaoObservada,
    Formulario(#[from] ErroSubmissaoFormulario),
    /// A tela de login voltou depois do POST: o SEI recusou a credencial.
    CredencialRecusada {
        mensagem_do_servidor: Option<String>,
    },
    ControleAposOLoginNaoReconhecido(ErroCaixa),
    CookieDaSessaoNaoRecebido,
    DestinoDaSessaoNaoEhArquivoRegular(ArquivoPrivado),
    DiretorioDaSessaoGravavelPorOutros(ArquivoPrivado),
    GravacaoDaSessaoFalhou(ArquivoPrivado),
    UnidadeExibidaDiferenteDaSolicitada,
    Transporte(#[from] FalhaTransporte),
    Html(#[from] ErroHtml),
}

impl ErroLogin {
    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Html(e) => e.mensagem,
            Self::Configuracao(erro) => erro.mensagem(),
            Self::Origem(_) => "arquivo de origem da instalação ausente, inseguro ou inválido",
            Self::ArquivoDeCredenciais(_) => "arquivo de credenciais ausente, inseguro ou ilegível",
            Self::ArquivoDaSessao(_) => {
                "arquivo de sessão gravado inválido; corrija-o ou use --forcar"
            }
            Self::Transporte(falha) => falha.mensagem(),
            Self::CaminhoDasCredenciaisNaoInformado => "defina IES_CREDENCIAIS_FILE",
            Self::CampoDasCredenciaisAusente(_) => {
                "o arquivo de credenciais não traz usuario, senha e orgao"
            }
            Self::CampoDasCredenciaisRepetido(_) => "campo repetido no arquivo de credenciais",
            Self::CampoDasCredenciaisVazio(_) => "campo vazio no arquivo de credenciais",
            Self::LinhaDasCredenciaisSemCampoConhecido => {
                "linha do arquivo de credenciais sem usuario=, senha= ou orgao="
            }
            Self::LoginEmAndamentoEmOutroProcesso => {
                "outro ies login segura o lock da sessão; nada foi enviado"
            }
            Self::ArquivoDeLockDoLoginInacessivel => {
                "não foi possível abrir o arquivo de lock do login"
            }
            Self::ControleDaSessaoExistenteNaoReconhecido(_) => {
                "a sessão gravada não mostrou Controle de Processos verificável"
            }
            Self::RedirecionamentoSemLocalizacao => "redirecionamento sem Location",
            Self::LocalizacaoDeRedirecionamentoNaoInterpretavel => {
                "Location do redirecionamento não interpretável"
            }
            Self::RedirecionamentoParaForaDaInstalacao => {
                "redirecionamento para fora do host da instalação recusado"
            }
            Self::RedirecionamentosAcimaDoLimite => "mais de 8 redirecionamentos seguidos",
            Self::RedirecionamentoQueRepetiriaOEnvioDoLogin => {
                "redirecionamento 307/308 repetiria o envio das credenciais"
            }
            Self::CookieRecebidoForaDoFormatoAceito => {
                "o SEI enviou cookie fora do formato aceito pelo arquivo de cookie"
            }
            Self::CharsetDaTelaDeLoginNaoSuportado => {
                "a tela de login não declara charset ISO-8859-1 ou UTF-8"
            }
            Self::CaptchaExigido => "a tela de login exige captcha; nada foi enviado de novo",
            Self::SegundoFatorExigido => "o SEI pediu segundo fator de autenticação",
            Self::RegraDoOnsubmitDoLoginNaoObservada => {
                "o onsubmit do login não é o observado (acaoLogin com hdnAcao)"
            }
            Self::Formulario(_) => "formulário de login fora do observado",
            Self::CredencialRecusada { .. } => "o SEI recusou a credencial; não repetir",
            Self::ControleAposOLoginNaoReconhecido(_) => {
                "o login não chegou a um Controle de Processos verificável"
            }
            Self::CookieDaSessaoNaoRecebido => "o login não deixou cookie para o Controle",
            Self::DestinoDaSessaoNaoEhArquivoRegular(_) => {
                "destino da sessão é link simbólico ou não é arquivo regular"
            }
            Self::DiretorioDaSessaoGravavelPorOutros(_) => {
                "o diretório da sessão é gravável por grupo ou outros (sem sticky bit)"
            }
            Self::GravacaoDaSessaoFalhou(_) => "não foi possível gravar a sessão",
            Self::UnidadeExibidaDiferenteDaSolicitada => {
                "a unidade mostrada pelo SEI não corresponde à solicitada"
            }
        }
    }

    /// Status e mensagem do SEI quando a recusa veio do status HTTP.
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(falha) => falha.status_http(),
            Self::Formulario(erro) => erro.status_http(),
            _ => None,
        }
    }

    /// Status com que o proxy do ambiente recusou o túnel.
    pub(crate) fn status_do_proxy(&self) -> Option<u16> {
        match self {
            Self::Transporte(falha) => falha.status_do_proxy(),
            _ => None,
        }
    }

    /// Chave do arquivo de credenciais envolvida (nunca o valor).
    pub(crate) fn campo_das_credenciais(&self) -> Option<&'static str> {
        match self {
            Self::CampoDasCredenciaisAusente(campo)
            | Self::CampoDasCredenciaisRepetido(campo)
            | Self::CampoDasCredenciaisVazio(campo) => Some(campo.chave()),
            _ => None,
        }
    }

    /// Texto saneado que a tela de login devolvida declarou como erro.
    pub(crate) fn mensagem_do_servidor(&self) -> Option<&str> {
        match self {
            Self::CredencialRecusada {
                mensagem_do_servidor,
            } => mensagem_do_servidor.as_deref(),
            _ => self
                .status_http()
                .and_then(|http| http.mensagem_do_servidor.as_deref()),
        }
    }
}
