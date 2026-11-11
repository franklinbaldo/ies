//! Derivação do formulário de login no host da instalação: `hdnAcao` só
//! com regra declarada, botão desabilitado pelo `onsubmit` servido e não
//! enviado, tela e `action` fora do host recusados.
use super::*;
use crate::sessao_login::tela::{ENTRAR_COM_ACESSAR, ENTRAR_COM_SBM_ACESSAR};
use crate::tela_html::HtmlServido;

fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
}

const FORM: &str = "<form id='frmLogin' method='post' action='login.php?s=1' onsubmit='return acaoLogin(2);'>\
    <input name='txtUsuario'><input type='password' name='pwdSenha'>\
    <select name='selOrgao'><option value='0'>A</option></select>\
    <button type='submit' name='BOTAO'>ACESSAR</button>\
    <input type='hidden' name='hdnAcao' value='1'></form>";

fn tela<'a>(url: &'a str, html: &'a HtmlServido) -> TelaServida<'a> {
    TelaServida {
        url,
        html,
        codificacao: CodificacaoDaTela::Iso88591,
    }
}

const CAMPOS: &[(&str, &str)] = &[
    ("txtUsuario", "u"),
    ("pwdSenha", "s"),
    ("selOrgao", "0"),
    ("hdnAcao", "2"),
];

#[test]
fn login_no_sip_fora_do_caminho_base_deriva_com_hdn_acao_reescrito() {
    let html = HtmlServido::de_teste(&FORM.replace("BOTAO", "Acessar"));
    let url = "https://sei.exemplo.gov.br/sip/login.php?s=1";
    let s = derivar_submissao_de_login(
        &ENTRAR_COM_ACESSAR,
        &instalacao(),
        &tela(url, &html),
        CAMPOS,
    )
    .unwrap();
    assert_eq!(s.url(), "https://sei.exemplo.gov.br/sip/login.php?s=1");
    assert_eq!(
        s.corpo(),
        b"txtUsuario=u&pwdSenha=s&selOrgao=0&Acessar=&hdnAcao=2"
    );
    assert_eq!(s.referer(), url);
    assert_eq!(s.origem(), "https://sei.exemplo.gov.br");
}

#[test]
fn botao_desabilitado_pelo_onsubmit_e_exigido_mas_nao_enviado() {
    let url = "https://sei.exemplo.gov.br/sip/login.php?s=1";
    let html = HtmlServido::de_teste(&FORM.replace("BOTAO", "sbmAcessar"));
    let s = derivar_submissao_de_login(
        &ENTRAR_COM_SBM_ACESSAR,
        &instalacao(),
        &tela(url, &html),
        CAMPOS,
    )
    .unwrap();
    assert_eq!(
        s.nomes_dos_campos(),
        ["txtUsuario", "pwdSenha", "selOrgao", "hdnAcao"]
    );
    let sem = HtmlServido::de_teste(&FORM.replace("BOTAO", "outro"));
    assert_eq!(
        derivar_submissao_de_login(
            &ENTRAR_COM_SBM_ACESSAR,
            &instalacao(),
            &tela(url, &sem),
            CAMPOS
        )
        .unwrap_err(),
        ErroSubmissaoFormulario::BotaoDeEnvioAusenteNoFormulario
    );
}

#[test]
fn tela_ou_action_fora_do_host_sao_recusados() {
    let html = FORM.replace("BOTAO", "Acessar");
    assert_eq!(
        derivar_submissao_de_login(
            &ENTRAR_COM_ACESSAR,
            &instalacao(),
            &tela(
                "https://outro.gov.br/sip/login.php",
                &HtmlServido::de_teste(&html)
            ),
            CAMPOS
        )
        .unwrap_err(),
        ErroSubmissaoFormulario::Transporte(FalhaTransporte::OrigemNaoAutorizada)
    );
    let fora = HtmlServido::de_teste(&html.replace(
        "action='login.php?s=1'",
        "action='//outro.gov.br/login.php'",
    ));
    assert_eq!(
        derivar_submissao_de_login(
            &ENTRAR_COM_ACESSAR,
            &instalacao(),
            &tela("https://sei.exemplo.gov.br/sip/login.php", &fora),
            CAMPOS
        )
        .unwrap_err(),
        ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao
    );
    let ponto =
        HtmlServido::de_teste(&html.replace("action='login.php?s=1'", "action='../sei/login.php'"));
    assert_eq!(
        derivar_submissao_de_login(
            &ENTRAR_COM_ACESSAR,
            &instalacao(),
            &tela("https://sei.exemplo.gov.br/sip/login.php", &ponto),
            CAMPOS
        )
        .unwrap_err(),
        ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao
    );
}
