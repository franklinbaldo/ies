//! Recusas tipadas do fluxo: cada uma termina sem novo POST, com a
//! sequência de pedidos conferida.
use super::tests_apoio::*;
use super::{ErroLogin, entrar_via, tela};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

fn erro_com(respostas: Vec<super::RespostaHttp>) -> (ErroLogin, Roteiro) {
    let ambiente = Ambiente::novo();
    let mut roteiro = Roteiro::novo(respostas);
    let erro = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap_err();
    assert!(
        !ambiente.arquivos.cookie.exists(),
        "falha não grava sessão: {erro:?}"
    );
    (erro, roteiro)
}

fn posts(roteiro: &Roteiro) -> usize {
    roteiro
        .pedidos
        .iter()
        .filter(|p| p.metodo == "POST")
        .count()
}

#[test]
fn redirecionamento_para_outro_host_e_recusado_sem_segui_lo() {
    for local in [
        "https://outro.exemplo.gov.br/sip/login.php",
        "http://outro.exemplo.gov.br/sei/",
        "//outro.exemplo.gov.br/sei/",
    ] {
        let (erro, roteiro) = erro_com(vec![r302(local, &[])]);
        assert_eq!(
            erro,
            ErroLogin::RedirecionamentoParaForaDaInstalacao,
            "{local}"
        );
        assert_eq!(roteiro.pedidos.len(), 1);
    }
}

#[test]
fn localizacao_ausente_ou_nao_interpretavel_e_recusada() {
    let sem = super::RespostaHttp {
        status: 302,
        cabecalhos: Vec::new(),
        corpo: Vec::new(),
    };
    assert_eq!(
        erro_com(vec![sem]).0,
        ErroLogin::RedirecionamentoSemLocalizacao
    );
    for local in [
        "/sei/../sip/login.php",
        "https://sei.exemplo.gov.br:8443/sei/",
        "https://usuario@sei.exemplo.gov.br/sei/",
        "javascript:alert(1)",
        "/sei/ login.php",
        "",
    ] {
        assert_eq!(
            erro_com(vec![r302(local, &[])]).0,
            ErroLogin::LocalizacaoDeRedirecionamentoNaoInterpretavel,
            "{local:?}"
        );
    }
}

#[test]
fn mais_de_oito_redirecionamentos_param_a_cadeia() {
    let respostas = (0..9)
        .map(|i| r302(&format!("/sei/salto{i}.php"), &[]))
        .collect();
    let (erro, roteiro) = erro_com(respostas);
    assert_eq!(erro, ErroLogin::RedirecionamentosAcimaDoLimite);
    assert_eq!(roteiro.pedidos.len(), 9);
}

// Caso de todos tests/test_redirect_seguro_010.py @323eb01
#[test]
fn redirecionamento_307_depois_do_post_nao_repete_as_credenciais() {
    let em_http = LOGIN_SIP.replacen("https://", "http://", 1);
    for (codigo, localizacao) in [307, 308]
        .into_iter()
        .flat_map(|codigo| [(codigo, LOGIN_SIP.to_owned()), (codigo, em_http.clone())])
    {
        let (erro, roteiro) = erro_com(vec![
            r302(LOGIN_SIP, &[]),
            r200(&tela_acessar(), &[]),
            status(codigo, Some(&localizacao), &[], "", "text/html"),
        ]);
        assert_eq!(erro, ErroLogin::RedirecionamentoQueRepetiriaOEnvioDoLogin);
        assert_eq!(
            roteiro.sequencia(),
            [
                format!("GET {RAIZ}"),
                format!("GET {LOGIN_SIP}"),
                format!("POST {LOGIN_SIP}"),
            ]
        );
        assert!(roteiro.esgotado());
    }
}

#[test]
fn raiz_sem_formulario_de_login_nao_envia_credencial() {
    let (erro, roteiro) = erro_com(vec![r200("<html><body>Portal</body></html>", &[])]);
    assert_eq!(erro, ErroLogin::TelaDeLoginNaoReconhecida);
    assert_eq!(posts(&roteiro), 0);
}

#[test]
fn captcha_servido_na_tela_bloqueia_antes_do_post() {
    let campo = "<label id=\"lblInfraCaptcha\"><img src=\"captcha.php\"></label>\
                 <input type=\"text\" id=\"txtInfraCaptcha\" name=\"txtInfraCaptcha\" value=\"\">";
    for html in [
        tela_sbm().replace(
            "<input type=\"hidden\" id=\"hdnAcao\"",
            &format!("{campo}<input type=\"hidden\" id=\"hdnAcao\""),
        ),
        tela_acessar().replace(
            "</form>",
            "<div class=\"g-recaptcha\" data-sitekey=\"x\"></div></form>",
        ),
    ] {
        let (erro, roteiro) = erro_com(vec![r302(LOGIN_SIP, &[]), r200(&html, &[])]);
        assert_eq!(erro, ErroLogin::CaptchaExigido);
        assert_eq!(posts(&roteiro), 0);
    }
}

#[test]
fn captcha_exigido_depois_do_post_tambem_e_tipado() {
    let campo = "<input type=\"text\" id=\"txtInfraCaptcha\" name=\"txtInfraCaptcha\">";
    let devolvida = tela_recusada("").replace("</form>", &format!("{campo}</form>"));
    let (erro, roteiro) = erro_com(vec![
        r302(LOGIN_SIP_INFRA_URL, &[]),
        r200(&tela_sbm(), &[]),
        r200(&devolvida, &[]),
    ]);
    assert_eq!(erro, ErroLogin::CaptchaExigido);
    assert_eq!(posts(&roteiro), 1);
}

#[test]
fn segundo_fator_pedido_depois_do_post_e_tipado() {
    let tela = "<html><body><form id=\"frmLogin\" name=\"frmLogin\" method=\"post\" action=\"login.php\">\
                <div id=\"divValidacao2FA\"><input type=\"text\" id=\"txtCodigo2FA\" name=\"txtCodigo2FA\"></div>\
                </form></body></html>";
    let (erro, roteiro) = erro_com(vec![
        r302(LOGIN_SIP, &[]),
        r200(&tela_acessar(), &[]),
        r200(tela, &[]),
    ]);
    assert_eq!(erro, ErroLogin::SegundoFatorExigido);
    assert_eq!(posts(&roteiro), 1);
}

#[test]
fn tela_de_login_devolvida_e_credencial_recusada_com_a_mensagem_saneada() {
    let (erro, roteiro) = erro_com(vec![
        r302(LOGIN_SIP_INFRA_URL, &[]),
        r200(&tela_sbm(), &[]),
        r200(&tela_recusada("Usu\u{e1}rio ou Senha Inv\u{e1}lida."), &[]),
    ]);
    assert_eq!(
        erro,
        ErroLogin::CredencialRecusada {
            mensagem_do_servidor: Some("Usu\u{e1}rio ou Senha Inv\u{e1}lida.".to_owned())
        }
    );
    assert_eq!(posts(&roteiro), 1);
    let (erro, _) = erro_com(vec![
        r302(LOGIN_SIP_INFRA_URL, &[]),
        r200(&tela_sbm(), &[]),
        r200(&tela_recusada(""), &[]),
    ]);
    // O aviso de navegador (divInfraNavegadorAviso) não é a recusa.
    assert_eq!(
        erro,
        ErroLogin::CredencialRecusada {
            mensagem_do_servidor: None
        }
    );
}

#[test]
fn credencial_recusada_le_a_validacao_da_infra_antes_dos_conteineres() {
    // UnB 4.0.9 (02/11): 200, tela de login de novo e txaInfraValidacao.
    let (erro, roteiro) = erro_com(vec![
        r302(LOGIN_SIP_INFRA_URL, &[]),
        r200(&tela_sbm(), &[]),
        r200(
            &tela_recusada_pela_infra("Usu&aacute;rio ou Senha Inv&aacute;lida."),
            &[],
        ),
    ]);
    assert_eq!(
        erro,
        ErroLogin::CredencialRecusada {
            mensagem_do_servidor: Some("Usu\u{e1}rio ou Senha Inv\u{e1}lida.".to_owned())
        }
    );
    assert_eq!(posts(&roteiro), 1);
    let ambas = tela_recusada("Outro texto").replace(
        "<form id=",
        "<textarea id=\"txaInfraValidacao\" style=\"display:none\">Senha expirada.</textarea><form id=",
    );
    assert_eq!(
        tela::mensagem_de_recusa(&ambas).as_deref(),
        Some("Senha expirada.")
    );
}

#[test]
fn orgao_ausente_ou_com_duas_opcoes_iguais_nao_envia_credencial() {
    let ambiente = Ambiente::novo();
    privado(&ambiente.credenciais, "usuario=u\nsenha=s\norgao=ORGAO-B\n");
    let mut roteiro = Roteiro::novo(vec![r302(LOGIN_SIP_INFRA_URL, &[]), r200(&tela_sbm(), &[])]);
    let erro = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap_err();
    assert_eq!(erro, ErroLogin::OrgaoNaoEncontradoNaTelaDeLogin);
    assert_eq!(posts(&roteiro), 0);

    let repetida = tela_sbm().replace(
        "<option value=\"1\">TESTE</option>",
        "<option value=\"1\">ORGAO-A</option>",
    );
    let (erro, roteiro) = erro_com(vec![r302(LOGIN_SIP_INFRA_URL, &[]), r200(&repetida, &[])]);
    assert_eq!(erro, ErroLogin::OrgaoComMaisDeUmaOpcaoNaTelaDeLogin);
    assert_eq!(posts(&roteiro), 0);

    // "CELG G" não casa por prefixo com "CELG G T" nem com "CELG D".
    privado(&ambiente.credenciais, "usuario=u\nsenha=s\norgao=CELG G\n");
    let mut roteiro = Roteiro::novo(vec![r302(LOGIN_SIP, &[]), r200(&tela_go(), &[])]);
    let erro = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap_err();
    assert_eq!(erro, ErroLogin::OrgaoNaoEncontradoNaTelaDeLogin);
}

#[test]
fn regras_de_formulario_nao_observadas_sao_recusadas() {
    let sem_regra = tela_acessar().replace("return acaoLogin(2);", "return validar();");
    assert_eq!(
        erro_com(vec![r302(LOGIN_SIP, &[]), r200(&sem_regra, &[])]).0,
        ErroLogin::RegraDoOnsubmitDoLoginNaoObservada
    );
    let sem_desabilitar =
        tela_sbm().replace("document.getElementById('sbmAcessar').disabled = true;", "");
    assert_eq!(
        erro_com(vec![r302(LOGIN_SIP, &[]), r200(&sem_desabilitar, &[])]).0,
        ErroLogin::BotaoDeEnvioDoLoginNaoReconhecido
    );
    let outro_botao = tela_acessar().replace("\"Acessar\"", "\"sbmEntrar\"");
    assert_eq!(
        erro_com(vec![r302(LOGIN_SIP, &[]), r200(&outro_botao, &[])]).0,
        ErroLogin::BotaoDeEnvioDoLoginNaoReconhecido
    );
    let sem_orgao = tela_acessar().replace("name=\"selOrgao\"", "name=\"selOutro\"");
    assert_eq!(
        erro_com(vec![r302(LOGIN_SIP, &[]), r200(&sem_orgao, &[])]).0,
        ErroLogin::SelecaoDeOrgaoAusenteNaTelaDeLogin
    );
    let _fora = tela_acessar().replace(
        "action=\"login.php?",
        "action=\"https://outro.exemplo.gov.br/sip/login.php?",
    );
}

#[test]
fn status_5xx_sai_com_status_e_mensagem_sem_retry() {
    let erro500 = || {
        status(
            500,
            None,
            &[],
            "<html><body><div id=\"divInfraExcecao\">Erro acessando o banco.</div></body></html>",
            "text/html; charset=iso-8859-1",
        )
    };
    let (erro, roteiro) = erro_com(vec![erro500()]);
    assert_eq!(
        erro,
        ErroLogin::Transporte(FalhaTransporte::Status(StatusHttpRecusado {
            status: 500,
            mensagem_do_servidor: Some("Erro acessando o banco.".to_owned())
        }))
    );
    assert_eq!(roteiro.pedidos.len(), 1);
    let (erro, roteiro) = erro_com(vec![
        r302(LOGIN_SIP, &[]),
        r200(&tela_acessar(), &[]),
        erro500(),
    ]);
    assert!(matches!(
        erro,
        ErroLogin::Transporte(FalhaTransporte::Status(StatusHttpRecusado {
            status: 500,
            ..
        }))
    ));
    assert_eq!(posts(&roteiro), 1);
}

// Caso de todos tests/sei_replay.py @323eb01 (sequência exata, sem retry)
#[test]
fn falha_de_rede_no_post_nao_repete() {
    let ambiente = Ambiente::novo();
    let mut roteiro = Roteiro::com_falhas(vec![
        Ok(r302(LOGIN_SIP, &[])),
        Ok(r200(&tela_acessar(), &[])),
        Err(FalhaTransporte::Rede),
    ]);
    let erro = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap_err();
    assert_eq!(erro, ErroLogin::Transporte(FalhaTransporte::Rede));
    assert_eq!(roteiro.pedidos.len(), 3);
}

#[test]
fn resposta_apos_o_login_desconhecida_ou_controle_ilegivel_e_tipada() {
    let (erro, _) = erro_com(vec![
        r302(LOGIN_SIP, &[]),
        r200(&tela_acessar(), &[]),
        r200("<html><body>Bem-vindo</body></html>", &[]),
    ]);
    assert_eq!(erro, ErroLogin::TelaAposOLoginNaoReconhecida);
    let (_erro, _) = erro_com(vec![
        r302(LOGIN_SIP, &[]),
        r200(&tela_acessar(), &["PHPSESSID=a; path=/"]),
        r302(&url_do_controle(), &[]),
        r200(
            "<html><body><a id=\"lnkInfraUnidade\">X</a></body></html>",
            &[],
        ),
    ]);
    let (erro, _) = erro_com(vec![
        r302(LOGIN_SIP, &[]),
        r200(&tela_acessar(), &[]),
        r200(&painel("abrirControle()"), &[]),
    ]);
    assert_eq!(erro, ErroLogin::LinkDoControleDeProcessosNaoReconhecido);
}

#[test]
fn controle_sem_cookie_para_o_controlador_nao_e_sessao() {
    let (erro, _) = erro_com(vec![
        r302(LOGIN_SIP, &[]),
        r200(&tela_acessar(), &["SIPSESSAO=s; path=/sip"]),
        r302(&url_do_controle(), &[]),
        r200(&controle("UNIDADE-TESTE"), &[]),
    ]);
    assert_eq!(erro, ErroLogin::CookieDaSessaoNaoRecebido);
}

#[test]
fn cookie_fora_do_formato_do_arquivo_e_recusado() {
    let (erro, _) = erro_com(vec![r302(LOGIN_SIP, &["X=a:b; path=/"])]);
    assert_eq!(erro, ErroLogin::CookieRecebidoForaDoFormatoAceito);
}

#[test]
fn sem_caminho_de_credenciais_nao_ha_pedido() {
    let ambiente = Ambiente::novo();
    let mut roteiro = Roteiro::novo(Vec::new());
    let mut pedido = ambiente.pedido(false);
    pedido.credenciais = None;
    assert_eq!(
        entrar_via(&pedido, &mut roteiro).unwrap_err(),
        ErroLogin::CaminhoDasCredenciaisNaoInformado
    );
    assert!(roteiro.pedidos.is_empty());
}

#[test]
fn falha_da_sessao_gravada_que_nao_e_vencimento_nao_faz_login() {
    let ambiente = Ambiente::novo();
    ambiente.gravar_sessao(&url_do_controle(), "PHPSESSID=gravada");
    let mut roteiro = Roteiro::novo(vec![status(503, None, &[], "", "text/plain")]);
    let _erro = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap_err();
    assert_eq!(roteiro.pedidos.len(), 1);
    let mut roteiro = Roteiro::novo(vec![r200("<html><body>outra</body></html>", &[])]);
    let _erro = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap_err();
}
