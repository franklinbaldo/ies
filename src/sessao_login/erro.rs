//! Recusas tipadas de `ies login`, uma por fato verificável. A mensagem é
//! apresentação; o subcódigo é a classificação. Nenhuma variante carrega
//! credencial, cookie, URL assinada ou HTML.
use crate::configuracao::{ArquivoPrivado, ErroConfiguracao};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroLogin {
    Configuracao(ErroConfiguracao),
    /// Origem da instalação ilegível ou fora do formato `Instalacao`.
    Origem(FalhaTransporte),
    CaminhoDasCredenciaisNaoInformado,
    ArquivoDeCredenciais(FalhaTransporte),
    CampoDasCredenciaisAusente(CampoDasCredenciais),
    CampoDasCredenciaisRepetido(CampoDasCredenciais),
    CampoDasCredenciaisVazio(CampoDasCredenciais),
    LinhaDasCredenciaisSemCampoConhecido,
    LoginEmAndamentoEmOutroProcesso,
    ArquivoDeLockDoLoginInacessivel,
    /// URL ou cookie gravados que o transporte recusa antes da rede.
    ArquivoDaSessao(FalhaTransporte),
    /// O GET da sessão gravada não mostrou um Controle reconhecível.
    ControleDaSessaoExistenteNaoReconhecido(ErroCaixa),
    RedirecionamentoSemLocalizacao,
    LocalizacaoDeRedirecionamentoNaoInterpretavel,
    RedirecionamentoParaForaDaInstalacao,
    RedirecionamentosAcimaDoLimite,
    /// 307/308 depois do POST: seguir repetiria as credenciais.
    RedirecionamentoQueRepetiriaOEnvioDoLogin,
    CookieRecebidoForaDoFormatoAceito,
    TelaDeLoginNaoReconhecida,
    CharsetDaTelaDeLoginNaoSuportado,
    CaptchaExigido,
    SegundoFatorExigido,
    SelecaoDeOrgaoAusenteNaTelaDeLogin,
    SelecaoDeOrgaoRepetidaNaTelaDeLogin,
    OrgaoNaoEncontradoNaTelaDeLogin,
    OrgaoComMaisDeUmaOpcaoNaTelaDeLogin,
    RegraDoOnsubmitDoLoginNaoObservada,
    BotaoDeEnvioDoLoginNaoReconhecido,
    Formulario(ErroSubmissaoFormulario),
    /// A tela de login voltou depois do POST: o SEI recusou a credencial.
    CredencialRecusada {
        mensagem_do_servidor: Option<String>,
    },
    TelaAposOLoginNaoReconhecida,
    LinkDoControleDeProcessosNaoReconhecido,
    ControleAposOLoginNaoReconhecido(ErroCaixa),
    CookieDaSessaoNaoRecebido,
    DestinoDaSessaoNaoEhArquivoRegular(ArquivoPrivado),
    GravacaoDaSessaoFalhou(ArquivoPrivado),
    UnidadeExibidaDiferenteDaSolicitada,
    Transporte(FalhaTransporte),
}

impl ErroLogin {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Configuracao(erro) => erro.subcodigo(),
            Self::Origem(falha)
            | Self::ArquivoDeCredenciais(falha)
            | Self::ArquivoDaSessao(falha)
            | Self::Transporte(falha) => falha.subcodigo(),
            Self::CaminhoDasCredenciaisNaoInformado => "caminho_credenciais_nao_informado",
            Self::CampoDasCredenciaisAusente(_) => "campo_das_credenciais_ausente",
            Self::CampoDasCredenciaisRepetido(_) => "campo_das_credenciais_repetido",
            Self::CampoDasCredenciaisVazio(_) => "campo_das_credenciais_vazio",
            Self::LinhaDasCredenciaisSemCampoConhecido => {
                "linha_das_credenciais_sem_campo_conhecido"
            }
            Self::LoginEmAndamentoEmOutroProcesso => "login_em_andamento_em_outro_processo",
            Self::ArquivoDeLockDoLoginInacessivel => "arquivo_de_lock_do_login_inacessivel",
            Self::ControleDaSessaoExistenteNaoReconhecido(_) => "controle_web_nao_reconhecido",
            Self::RedirecionamentoSemLocalizacao => "redirecionamento_sem_localizacao",
            Self::LocalizacaoDeRedirecionamentoNaoInterpretavel => {
                "localizacao_de_redirecionamento_nao_interpretavel"
            }
            Self::RedirecionamentoParaForaDaInstalacao => {
                "redirecionamento_para_fora_da_instalacao"
            }
            Self::RedirecionamentosAcimaDoLimite => "redirecionamentos_acima_do_limite",
            Self::RedirecionamentoQueRepetiriaOEnvioDoLogin => {
                "redirecionamento_que_repetiria_o_envio_do_login"
            }
            Self::CookieRecebidoForaDoFormatoAceito => "cookie_recebido_fora_do_formato_aceito",
            Self::TelaDeLoginNaoReconhecida => "tela_de_login_nao_reconhecida",
            Self::CharsetDaTelaDeLoginNaoSuportado => "charset_da_tela_de_login_nao_suportado",
            Self::CaptchaExigido => "captcha_exigido",
            Self::SegundoFatorExigido => "segundo_fator_exigido",
            Self::SelecaoDeOrgaoAusenteNaTelaDeLogin => "selecao_de_orgao_ausente_na_tela_de_login",
            Self::SelecaoDeOrgaoRepetidaNaTelaDeLogin => {
                "selecao_de_orgao_repetida_na_tela_de_login"
            }
            Self::OrgaoNaoEncontradoNaTelaDeLogin => "orgao_nao_encontrado_na_tela_de_login",
            Self::OrgaoComMaisDeUmaOpcaoNaTelaDeLogin => {
                "orgao_com_mais_de_uma_opcao_na_tela_de_login"
            }
            Self::RegraDoOnsubmitDoLoginNaoObservada => "regra_do_onsubmit_do_login_nao_observada",
            Self::BotaoDeEnvioDoLoginNaoReconhecido => "botao_de_envio_do_login_nao_reconhecido",
            Self::Formulario(erro) => erro.subcodigo(),
            Self::CredencialRecusada { .. } => "credencial_recusada",
            Self::TelaAposOLoginNaoReconhecida => "tela_apos_o_login_nao_reconhecida",
            Self::LinkDoControleDeProcessosNaoReconhecido => {
                "link_do_controle_de_processos_nao_reconhecido"
            }
            Self::ControleAposOLoginNaoReconhecido(_) => "controle_apos_o_login_nao_reconhecido",
            Self::CookieDaSessaoNaoRecebido => "cookie_da_sessao_nao_recebido",
            Self::DestinoDaSessaoNaoEhArquivoRegular(_) => {
                "destino_da_sessao_nao_eh_arquivo_regular"
            }
            Self::GravacaoDaSessaoFalhou(_) => "gravacao_da_sessao_falhou",
            Self::UnidadeExibidaDiferenteDaSolicitada => "unidade_exibida_diferente_da_solicitada",
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
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
            Self::TelaDeLoginNaoReconhecida => "a raiz não levou a um formulário de login",
            Self::CharsetDaTelaDeLoginNaoSuportado => {
                "a tela de login não declara charset ISO-8859-1 ou UTF-8"
            }
            Self::CaptchaExigido => "a tela de login exige captcha; nada foi enviado de novo",
            Self::SegundoFatorExigido => "o SEI pediu segundo fator de autenticação",
            Self::SelecaoDeOrgaoAusenteNaTelaDeLogin => "a tela de login não publica selOrgao",
            Self::SelecaoDeOrgaoRepetidaNaTelaDeLogin => {
                "a tela de login publica mais de um selOrgao"
            }
            Self::OrgaoNaoEncontradoNaTelaDeLogin => {
                "nenhuma opção de órgão da tela é igual ao órgão configurado"
            }
            Self::OrgaoComMaisDeUmaOpcaoNaTelaDeLogin => {
                "mais de uma opção de órgão da tela é igual ao órgão configurado"
            }
            Self::RegraDoOnsubmitDoLoginNaoObservada => {
                "o onsubmit do login não é o observado (acaoLogin com hdnAcao)"
            }
            Self::BotaoDeEnvioDoLoginNaoReconhecido => {
                "botão de envio do login diferente dos observados"
            }
            Self::Formulario(_) => "formulário de login fora do observado",
            Self::CredencialRecusada { .. } => "o SEI recusou a credencial; não repetir",
            Self::TelaAposOLoginNaoReconhecida => {
                "a resposta do login não é Controle, Painel nem tela de login"
            }
            Self::LinkDoControleDeProcessosNaoReconhecido => {
                "o link do Controle de Processos do menu não é o observado"
            }
            Self::ControleAposOLoginNaoReconhecido(_) => {
                "o login não chegou a um Controle de Processos verificável"
            }
            Self::CookieDaSessaoNaoRecebido => "o login não deixou cookie para o Controle",
            Self::DestinoDaSessaoNaoEhArquivoRegular(_) => {
                "destino da sessão é link simbólico ou não é arquivo regular"
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
