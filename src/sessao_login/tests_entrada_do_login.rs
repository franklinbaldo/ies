//! O 302 do GET do Controle para a entrada do login da instalação: SIP,
//! módulo de Goiás e raiz, `http://` no mesmo host; e os casos que
//! continuam `redirecionamento_recusado` (outro host, outro caminho,
//! fragmento, recusa de link sem assinatura, sem `Location`). Fixtures
//! sintéticas com a forma das telas observadas (docs/como-o-sei-funciona/
//! sessao-login.md).
use super::entrada_do_login::aponta_para_o_login;
use super::tests_apoio::{LOGIN_SIP, LOGIN_SIP_INFRA_URL, MODULO_GO, ORIGEM, url_do_controle};
use crate::instalacao::Instalacao;
use crate::transporte::envio_post::ler_tela_via;
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, RespostaWeb, StatusHttpRecusado};
use std::io::Cursor;

fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}

fn aponta(localizacao: &str) -> bool {
    aponta_para_o_login(&instalacao(), &url_do_controle(), localizacao)
}

#[test]
fn entradas_do_login_da_instalacao() {
    let modulo_https = format!("https://{MODULO_GO}");
    let modulo_http = format!("http://{MODULO_GO}");
    for local in [
        LOGIN_SIP,
        LOGIN_SIP_INFRA_URL,
        "/sip/login.php?sigla_orgao_sistema=ORGAO-A&sigla_sistema=SEI",
        "//sei.exemplo.gov.br/sip/login.php?sigla_sistema=SEI",
        "https://SEI.Exemplo.GOV.br/sip/login.php",
        "/sip/login.php",
        modulo_https.as_str(),
        "modulos/goias/login-sei/MdGoiasLoginSEI.php?acao=md_goias_login_SEI",
        "/sei/modulos/goias/login-sei/login.php?sigla_sistema=SEI",
        "https://sei.exemplo.gov.br/sei/",
        "/sei/",
        "/sei/?infra_url=L3NlaS8=",
        // `http://` no mesmo host conta como a mesma origem (UFRJ e GO).
        "http://sei.exemplo.gov.br/sip/login.php?sigla_orgao_sistema=ORGAO-A&sigla_sistema=SEI",
        "http://sei.exemplo.gov.br:80/sei/",
        modulo_http.as_str(),
    ] {
        assert!(aponta(local), "{local}");
    }
}

#[test]
fn outro_host_outro_caminho_ou_fragmento_nao_e_o_login() {
    for local in [
        "https://outro.gov.br/sip/login.php",
        "https://sei.exemplo.gov.br.mal.example/sip/login.php",
        "https://sei.exemplo.gov.br@mal.example/sip/login.php",
        "https://sei.exemplo.gov.br:443/sip/login.php",
        "http://sei.exemplo.gov.br:8080/sip/login.php",
        "ftp://sei.exemplo.gov.br/sip/login.php",
        // O 302 do Controle para o próprio Controle (troca de unidade).
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar",
        "http://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar",
        "/sei/inicializar.php",
        "/sei/login.php",
        "login.php",
        "/sip/login.php.bak",
        "/sip/Login.php",
        "/sip/",
        "/sei",
        "/",
        "/outra/sip/login.php",
        "/sei/modulos/goias/login-sei/",
        "/sei/modulos/outro/login-sei/login.php",
        "../sip/login.php",
        "/sei/../sip/login.php",
        "/sip/login.php#topo",
        "/sei/#",
        "/sip/login.php?msg=Link+sem+assinatura",
        "/sip/login.php?sigla_sistema=SEI&msg=Link%20sem%20assinatura",
        "",
        "   ",
        "/sip/login.php?a=\u{e9}",
    ] {
        assert!(!aponta(local), "{local}");
    }
}

#[test]
fn instalacao_na_raiz_do_host_tem_a_raiz_e_o_modulo_na_raiz() {
    let raiz = Instalacao::interpretar("https://sei.exemplo.gov.br").unwrap();
    let atual = "https://sei.exemplo.gov.br/controlador.php?acao=procedimento_controlar";
    for (local, esperado) in [
        ("/", true),
        ("/sip/login.php", true),
        ("/modulos/goias/login-sei/MdGoiasLoginSEI.php", true),
        ("/sei/", false),
        ("/sei/modulos/goias/login-sei/MdGoiasLoginSEI.php", false),
    ] {
        assert_eq!(
            aponta_para_o_login(&raiz, atual, local),
            esperado,
            "{local}"
        );
    }
}

/// O GET do Controle (`ler_tela_via`, o caminho de `unidade *`,
/// `processo buscar` e `documento buscar`) com a resposta dada.
fn ler_controle(status: i32, localizacao: Option<&str>) -> FalhaTransporte {
    let ambiente = super::tests_apoio::Ambiente::novo();
    let cookie = &ambiente.arquivos.cookie;
    super::tests_apoio::privado(cookie, "PHPSESSID=desconhecida\n");
    let mut pedidos = 0;
    let falha = ler_tela_via(
        &url_do_controle(),
        &CookieDaSessao::ler(cookie),
        ORIGEM,
        |_, _| {
            pedidos += 1;
            Ok(RespostaWeb {
                status_code: status,
                mime: "text/html".to_owned(),
                content_length: None,
                localizacao: localizacao.map(str::to_owned),
                corpo: Cursor::new(Vec::new()),
            })
        },
    )
    .unwrap_err();
    assert_eq!(pedidos, 1, "um único GET, o 302 nunca é seguido");
    falha
}

#[test]
fn controle_302_para_o_login_e_sessao_encerrada_com_status() {
    for local in [
        LOGIN_SIP,
        "/sei/",
        "http://sei.exemplo.gov.br/sip/login.php",
    ] {
        let falha = ler_controle(302, Some(local));
        assert_eq!(
            falha,
            FalhaTransporte::ControleRedirecionadoParaOLogin(StatusHttpRecusado::sem_mensagem(302)),
            "{local}"
        );
        assert_eq!(falha.subcodigo(), "controle_redirecionado_para_o_login");
        assert_eq!(falha.status_http().map(|s| s.status), Some(302));
    }
}

#[test]
fn outro_302_outro_3xx_ou_sem_location_continua_recusado() {
    for (status, local) in [
        (302, None),
        (302, Some("https://outro.gov.br/sip/login.php")),
        (
            302,
            Some("/sei/controlador.php?acao=procedimento_controlar"),
        ),
        (302, Some("/sip/login.php#x")),
        (301, Some(LOGIN_SIP)),
        (303, Some(LOGIN_SIP)),
        (307, Some(LOGIN_SIP)),
    ] {
        let falha = ler_controle(status, local);
        assert_eq!(falha.subcodigo(), "redirecionamento", "{status} {local:?}");
        assert_eq!(
            falha.status_http().map(|s| s.status),
            Some(u16::try_from(status).unwrap())
        );
    }
}
