//! Telas do login, lidas com `tl` e `html_atributos` (sem regex): o
//! formulário `frmLogin` servido, captcha e segundo fator exigidos, a
//! mensagem de recusa e o link do Controle no Painel. As `const` da
//! operação ficam aqui; o primitivo de formulário deriva o POST.
use super::{Credenciais, ErroLogin};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    ResultadoDoEnvio, SubmissaoDerivada, TelaServida,
};
use crate::instalacao::Instalacao;

const CAMPOS_DO_LOGIN: &[&str] = &["txtUsuario", "pwdSenha", "selOrgao"];

/// `hdnAcao` é gravado pelo `acaoLogin(N)` do `onsubmit` servido.
const OCULTOS_REESCRITOS: &[&str] = &["hdnAcao"];

/// Login com o botão `Acessar` (`UnB` 4.0.9): o `acaoLogin(2)` servido não o
/// desabilita, e ele participa do corpo com o valor servido.
pub(crate) const ENTRAR_COM_ACESSAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "login",
    formulario: "frmLogin",
    envio: EnvioDoFormulario::BotaoDeEnvio("Acessar"),
    campos_ocultos_exigidos: OCULTOS_REESCRITOS,
    campos_ocultos_reescritos: OCULTOS_REESCRITOS,
    campos_usuario: CAMPOS_DO_LOGIN,
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    // A tela de login não publica unidade; o Controle obtido é conferido.
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::SessaoAutenticada,
    },
};

/// Login com `sbmAcessar` (PEN1, UFRJ, GO): o `acaoLogin(2)` servido o
/// desabilita antes de o navegador montar o corpo; ele não participa.
pub(crate) const ENTRAR_COM_SBM_ACESSAR: OperacaoDeFormulario = OperacaoDeFormulario {
    envio: EnvioDoFormulario::BotaoDesabilitadoPeloOnsubmit("sbmAcessar"),
    ..ENTRAR_COM_ACESSAR
};

/// Trecho literal do `acaoLogin` servido nas quatro instalações.
pub(crate) const HDN_ACAO_PELO_ARGUMENTO: &str = "document.getElementById('hdnAcao').value = acao;";

/// Trecho literal que desabilita `sbmAcessar` (PEN1, UFRJ, GO).
pub(crate) const SBM_ACESSAR_DESABILITADO: &str =
    "document.getElementById('sbmAcessar').disabled = true;";

/// Controle de captcha servido (não basta CSS ou função JS de captcha).
pub(crate) fn captcha_exigido(html: &str) -> bool {
    let _ = html;
    todo!("CONTRACT: captcha exigido")
}

/// Campo do segundo fator servido (`txtCodigo2FA`, observado pelo Todos).
pub(crate) fn segundo_fator_exigido(html: &str) -> bool {
    let _ = html;
    todo!("CONTRACT: segundo fator")
}

/// Texto saneado da mensagem de erro que a tela declara, se houver.
pub(crate) fn mensagem_de_recusa(html: &str) -> Option<String> {
    let _ = html;
    todo!("CONTRACT: mensagem de recusa")
}

/// URL (relativa, como publicada) do `lnkControleProcessos` do menu.
pub(crate) fn link_do_controle(html: &str) -> Result<Option<String>, ErroLogin> {
    let _ = html;
    todo!("CONTRACT: link do Controle")
}

/// POST do login derivado do `frmLogin` servido, pela `const` do botão
/// observado, com `selOrgao` e `hdnAcao` pelas regras observadas.
pub(crate) fn derivar_login(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    credenciais: &Credenciais,
) -> Result<SubmissaoDerivada, ErroLogin> {
    let _ = (instalacao, tela, credenciais, ENTRAR_COM_SBM_ACESSAR);
    let _ = crate::formulario_web::derivar_submissao_de_login;
    todo!("CONTRACT: derivar login")
}
