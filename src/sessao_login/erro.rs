//! Recusas tipadas de `ies login`, uma por fato verificável. A mensagem é
//! apresentação; o subcódigo é a classificação. Nenhuma variante carrega
//! credencial, cookie, URL assinada ou HTML.
use crate::configuracao::{ArquivoPrivado, ErroConfiguracao};
use crate::erro_html::ErroHtml;
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
