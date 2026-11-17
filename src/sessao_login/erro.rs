//! Recusas tipadas de `ies login`, uma por fato verificável. Nenhum erro
//! carrega credencial, cookie, URL assinada ou HTML.
use crate::arquivo_privado::ErroArquivoPrivado;
use crate::configuracao::{ArquivoPrivado, ErroConfiguracao};
use crate::erro_web::ErroCaixa;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::transporte::FalhaTransporte;
use std::sync::Arc;

/// Causa de E/S guardada inteira, com `source()`. `io::Error` não é
/// `Clone` nem `PartialEq`, então é compartilhada por `Arc` e comparada
/// pelo `kind` e pelo código do sistema operacional.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{0}")]
pub(crate) struct ErroDeIo(#[source] Arc<std::io::Error>);

impl ErroDeIo {
    pub(crate) fn new(erro: std::io::Error) -> Self {
        Self(Arc::new(erro))
    }
}

impl PartialEq for ErroDeIo {
    fn eq(&self, outro: &Self) -> bool {
        self.0.kind() == outro.0.kind() && self.0.raw_os_error() == outro.0.raw_os_error()
    }
}

impl Eq for ErroDeIo {}

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
    Configuracao(#[from] ErroConfiguracao),
    /// O arquivo da origem da instalação não foi lido.
    OrigemIlegivel(#[source] ErroArquivoPrivado),
    /// A origem lida não é uma instalação (`Instalacao`).
    OrigemNaoAutorizada,
    CaminhoDasCredenciaisNaoInformado,
    /// O arquivo de credenciais não foi lido.
    ArquivoDeCredenciais(#[source] ErroArquivoPrivado),
    CampoDasCredenciaisAusente(CampoDasCredenciais),
    CampoDasCredenciaisRepetido(CampoDasCredenciais),
    CampoDasCredenciaisVazio(CampoDasCredenciais),
    LinhaDasCredenciaisSemCampoConhecido,
    LoginEmAndamentoEmOutroProcesso,
    ArquivoDeLockDoLoginInacessivel(ErroDeIo),
    /// O arquivo da URL ou do cookie gravados não foi lido.
    ArquivoDaSessao(#[source] ErroArquivoPrivado),
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
    /// O `<form>` de login não tem o atributo `onsubmit`.
    OnsubmitDoLoginAusente,
    /// O `onsubmit` não é `return acaoLogin(N);` com `N` só de dígitos.
    OnsubmitDoLoginForaDoFormato,
    /// A página não grava o argumento do `acaoLogin` em `hdnAcao`.
    ScriptDoLoginSemHdnAcao,
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
    /// A serialização do marcador falhou; guarda a categoria do erro do serde.
    SerializacaoDaSessaoFalhou(ArquivoPrivado, serde_json::error::Category),
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
            | Self::OrigemIlegivel(_)
            | Self::OrigemNaoAutorizada
            | Self::CaminhoDasCredenciaisNaoInformado
            | Self::ArquivoDeCredenciais(_)
            | Self::CampoDasCredenciaisAusente(_)
            | Self::CampoDasCredenciaisRepetido(_)
            | Self::CampoDasCredenciaisVazio(_)
            | Self::LinhaDasCredenciaisSemCampoConhecido
            | Self::LoginEmAndamentoEmOutroProcesso
            | Self::ArquivoDeLockDoLoginInacessivel(_)
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
            | Self::OnsubmitDoLoginAusente
            | Self::OnsubmitDoLoginForaDoFormato
            | Self::ScriptDoLoginSemHdnAcao
            | Self::CredencialRecusada { .. }
            | Self::ControleAposOLoginNaoReconhecido(_)
            | Self::CookieDaSessaoNaoRecebido
            | Self::DestinoDaSessaoNaoEhArquivoRegular(_)
            | Self::GravacaoDaSessaoFalhou(_)
            | Self::SerializacaoDaSessaoFalhou(..)
            | Self::MarcadorDaSessaoIlegivel(_)
            | Self::UnidadeExibidaDiferenteDaSolicitada
            | Self::BotaoDeEnvioDoLoginNaoReconhecido
            | Self::LinkDoControleDeProcessosNaoReconhecido
            | Self::OrgaoComMaisDeUmaOpcaoNaTelaDeLogin
            | Self::OrgaoNaoEncontradoNaTelaDeLogin
            | Self::SelecaoDeOrgaoAusenteNaTelaDeLogin
            | Self::SelecaoDeOrgaoRepetidaNaTelaDeLogin
            | Self::TelaAposOLoginNaoReconhecida
            | Self::TelaDeLoginNaoReconhecida => false,
        }
    }
}
