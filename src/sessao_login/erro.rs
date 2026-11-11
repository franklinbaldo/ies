//! Recusas tipadas de `ies login`, uma por fato verificável. Nenhum erro
//! carrega credencial, cookie, URL assinada ou HTML.
use crate::configuracao::{ArquivoPrivado, ErroConfiguracao};
use crate::erro_web::ErroCaixa;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::transporte::FalhaTransporte;

/// Campo do arquivo de credenciais.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CampoDasCredenciais {
    Usuario,
    Senha,
    Orgao,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLogin {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
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
    #[cfg(unix)]
    DiretorioDaSessaoGravavelPorOutros(ArquivoPrivado),
    GravacaoDaSessaoFalhou(ArquivoPrivado),
    /// O marcador `.geracao` existe e não pôde ser lido.
    MarcadorDaSessaoIlegivel(#[source] super::gravacao::ErroMarcadorDaSessao),
    UnidadeExibidaDiferenteDaSolicitada,
    Transporte(#[from] FalhaTransporte),
    BotaoDeEnvioDoLoginNaoReconhecido,
    LinkDoControleDeProcessosNaoReconhecido,
    OrgaoComMaisDeUmaOpcaoNaTelaDeLogin,
    OrgaoNaoEncontradoNaTelaDeLogin,
    SelecaoDeOrgaoAusenteNaTelaDeLogin,
    SelecaoDeOrgaoRepetidaNaTelaDeLogin,
    TelaAposOLoginNaoReconhecida,
    TelaDeLoginNaoReconhecida,
}

impl ErroLogin {
    /// A sessão do SEI acabou no meio do caminho (validação de acesso
    /// durante uma leitura): decide o relogin único por `match`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Transporte(falha) => falha.sessao_encerrada(),
            Self::Formulario(erro) => erro.sessao_encerrada(),
            #[cfg(unix)]
            Self::DiretorioDaSessaoGravavelPorOutros(_) => false,
            Self::Configuracao(_)
            | Self::Origem(_)
            | Self::CaminhoDasCredenciaisNaoInformado
            | Self::ArquivoDeCredenciais(_)
            | Self::CampoDasCredenciaisAusente(_)
            | Self::CampoDasCredenciaisRepetido(_)
            | Self::CampoDasCredenciaisVazio(_)
            | Self::LinhaDasCredenciaisSemCampoConhecido
            | Self::LoginEmAndamentoEmOutroProcesso
            | Self::ArquivoDeLockDoLoginInacessivel
            | Self::ArquivoDaSessao(_)
            | Self::ControleDaSessaoExistenteNaoReconhecido(_)
            | Self::RedirecionamentoSemLocalizacao
            | Self::LocalizacaoDeRedirecionamentoNaoInterpretavel
            | Self::RedirecionamentoParaForaDaInstalacao
            | Self::RedirecionamentosAcimaDoLimite
            | Self::RedirecionamentoQueRepetiriaOEnvioDoLogin
            | Self::CookieRecebidoForaDoFormatoAceito
            | Self::CharsetDaTelaDeLoginNaoSuportado
            | Self::CaptchaExigido
            | Self::SegundoFatorExigido
            | Self::RegraDoOnsubmitDoLoginNaoObservada
            | Self::CredencialRecusada { .. }
            | Self::ControleAposOLoginNaoReconhecido(_)
            | Self::CookieDaSessaoNaoRecebido
            | Self::DestinoDaSessaoNaoEhArquivoRegular(_)
            | Self::GravacaoDaSessaoFalhou(_)
            | Self::MarcadorDaSessaoIlegivel(_)
            | Self::UnidadeExibidaDiferenteDaSolicitada
            | Self::BotaoDeEnvioDoLoginNaoReconhecido
            | Self::LinkDoControleDeProcessosNaoReconhecido
            | Self::OrgaoComMaisDeUmaOpcaoNaTelaDeLogin
            | Self::OrgaoNaoEncontradoNaTelaDeLogin
            | Self::SelecaoDeOrgaoAusenteNaTelaDeLogin
            | Self::SelecaoDeOrgaoRepetidaNaTelaDeLogin
            | Self::TelaAposOLoginNaoReconhecida
            | Self::Atributo(_)
            | Self::TelaDeLoginNaoReconhecida => false,
        }
    }
}
