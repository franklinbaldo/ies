//! Peças do login isoladas: credenciais, `Location`, cookies, leitura
//! HTTP/1.1, telas e gravação.
use super::cookies::PoteDeCookies;
use super::redirecionamento::resolver_no_host;
use super::tests_apoio::*;
use super::{CampoDasCredenciais as C, Credenciais, ErroLogin, RespostaHttp, http, tela};
use crate::erro_html::ErroHtml;
use std::io::Cursor;

#[test]
fn credenciais_aceitam_senha_com_igual_e_espacos_e_redigem_o_debug() {
    let c = Credenciais::interpretar("\nusuario= aluno1 \r\nsenha= a=b c \norgao=UnB\n").unwrap();
    assert_eq!(c.usuario(), "aluno1");
    assert_eq!(c.senha(), " a=b c ");
    assert_eq!(c.orgao(), "UnB");
    let debug = format!("{c:?}");
    assert!(!debug.contains("aluno1") && !debug.contains("a=b"));
}

#[test]
fn credenciais_recusam_campo_ausente_repetido_vazio_ou_desconhecido() {
    for (texto, erro) in [
        (
            "usuario=u\nsenha=s\n",
            ErroLogin::CampoDasCredenciaisAusente(C::Orgao),
        ),
        (
            "usuario=u\nusuario=v\nsenha=s\norgao=o",
            ErroLogin::CampoDasCredenciaisRepetido(C::Usuario),
        ),
        (
            "usuario=u\nsenha=\norgao=o",
            ErroLogin::CampoDasCredenciaisVazio(C::Senha),
        ),
        (
            "usuario=u\nsenha=s\norgao=o\nsistema=SEI",
            ErroLogin::LinhaDasCredenciaisSemCampoConhecido,
        ),
        (
            "usuario u\nsenha=s\norgao=o",
            ErroLogin::LinhaDasCredenciaisSemCampoConhecido,
        ),
    ] {
        assert_eq!(
            Credenciais::interpretar(texto).unwrap_err(),
            erro,
            "{texto:?}"
        );
    }
}

#[cfg(unix)]
#[test]
fn credenciais_inseguras_sao_recusadas_pelo_arquivo_privado() {
    use std::os::unix::fs::PermissionsExt;
    let ambiente = Ambiente::novo();
    std::fs::set_permissions(
        &ambiente.credenciais,
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
}

#[test]
fn location_e_resolvido_no_host_com_https() {
    let atual = "https://sei.exemplo.gov.br/sei/inicializar.php?a=1";
    for (local, esperado) in [
        (
            "controlador.php?acao=x",
            "https://sei.exemplo.gov.br/sei/controlador.php?acao=x",
        ),
        (
            "/sip/login.php?s=1",
            "https://sei.exemplo.gov.br/sip/login.php?s=1",
        ),
        ("?b=2", "https://sei.exemplo.gov.br/sei/inicializar.php?b=2"),
        (
            "https://SEI.exemplo.gov.br/sei/",
            "https://sei.exemplo.gov.br/sei/",
        ),
        (
            "http://sei.exemplo.gov.br/sei/",
            "https://sei.exemplo.gov.br/sei/",
        ),
        ("HTTP://sei.exemplo.gov.br", "https://sei.exemplo.gov.br/"),
        (
            "//sei.exemplo.gov.br/sei/x.php#frag",
            "https://sei.exemplo.gov.br/sei/x.php",
        ),
        (
            "https://sei.exemplo.gov.br/sei/m.php?infra_url=L3NlaS8=",
            "https://sei.exemplo.gov.br/sei/m.php?infra_url=L3NlaS8=",
        ),
    ] {
        assert_eq!(
            resolver_no_host(atual, local, HOST).unwrap(),
            esperado,
            "{local}"
        );
    }
    assert_eq!(
        resolver_no_host(atual, "https://sei.exemplo.gov.br.outro.br/", HOST),
        Err(ErroLogin::RedirecionamentoParaForaDaInstalacao)
    );
    assert_eq!(
        resolver_no_host(atual, "./controlador.php", HOST),
        Err(ErroLogin::LocalizacaoDeRedirecionamentoNaoInterpretavel)
    );
}

fn resposta_com(cookies: &[&str]) -> RespostaHttp {
    r302("/", cookies)
}

#[test]
fn pote_respeita_path_remocao_e_dominio_e_redige_valores() {
    let mut pote = PoteDeCookies::novo(HOST);
    pote.absorver(
        "https://sei.exemplo.gov.br/sip/login.php?x=1",
        &resposta_com(&[
            "SIP=s1; path=/sip; secure",
            "SEMPATH=p",
            "GERAL=g; Path=/; HttpOnly",
            "ALHEIO=a; Domain=outro.gov.br; Path=/",
            "PAI=b; Domain=exemplo.gov.br; Path=/",
        ]),
    )
    .unwrap();
    // `SEMPATH` herda o diretório do pedido (/sip); caminhos longos primeiro.
    assert_eq!(
        pote.cabecalho("https://sei.exemplo.gov.br/sip/login.php"),
        "SIP=s1; SEMPATH=p; GERAL=g; PAI=b"
    );
    assert_eq!(
        pote.cabecalho("https://sei.exemplo.gov.br/sei/controlador.php?acao=a"),
        "GERAL=g; PAI=b"
    );
    assert_eq!(
        pote.cabecalho("https://sei.exemplo.gov.br/sipx/a"),
        "GERAL=g; PAI=b"
    );
    pote.absorver(
        "https://sei.exemplo.gov.br/sei/",
        &resposta_com(&[
            "GERAL=deleted; expires=Thu, 01 Jan 1970 00:00:01 GMT; Max-Age=0; path=/",
            "PAI=c; Expires=Thu, 19 Nov 1981 08:52:00 GMT; Domain=exemplo.gov.br; Path=/",
        ]),
    )
    .unwrap();
    assert_eq!(pote.cabecalho("https://sei.exemplo.gov.br/sei/x"), "");
    assert!(!format!("{pote:?}").contains("s1"));
    for invalido in ["=semnome", "SEMIGUAL", "X=\"aspas\"", "X=a b", "X=a,b"] {
        assert_eq!(
            PoteDeCookies::novo(HOST).absorver(RAIZ, &resposta_com(&[invalido])),
            Err(ErroLogin::CookieRecebidoForaDoFormatoAceito),
            "{invalido}"
        );
    }
}

#[test]
fn leitura_http_guarda_todos_os_set_cookie_e_decodifica_o_corpo() {
    let bruto = "HTTP/1.1 302 Found\r\nLocation: /sip/login.php\r\nSet-Cookie: A=1; path=/\r\nset-cookie: B=2; path=/\r\nContent-Length: 999\r\n\r\nnao lido";
    let r = http::ler_resposta(&mut Cursor::new(bruto.as_bytes())).unwrap();
    assert_eq!(r.status, 302);
    assert_eq!(
        r.valores("set-cookie").collect::<Vec<_>>(),
        ["A=1; path=/", "B=2; path=/"]
    );
    assert_eq!(r.cabecalho("location"), Some("/sip/login.php"));
    assert_eq!(r.corpo, b"");

    let chunked = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/html\r\n\r\n4;ext=1\r\n<htm\r\n3\r\nl/>\r\n0\r\nX-Trailer: 1\r\n\r\n";
    let r = http::ler_resposta(&mut Cursor::new(chunked.as_bytes())).unwrap();
    assert_eq!(r.corpo, b"<html/>");

    let fixo = "HTTP/1.0 500 Internal\r\nContent-Length: 3\r\n\r\nabcdef";
    assert_eq!(
        http::ler_resposta(&mut Cursor::new(fixo.as_bytes()))
            .unwrap()
            .corpo,
        b"abc"
    );

    let ate_o_fim = "HTTP/1.1 200 OK\r\n\r\nfim";
    assert_eq!(
        http::ler_resposta(&mut Cursor::new(ate_o_fim.as_bytes()))
            .unwrap()
            .corpo,
        b"fim"
    );

    let grande = "HTTP/1.1 200 OK\r\nContent-Length: 9000000\r\n\r\n";
    assert_eq!(
        http::ler_resposta(&mut Cursor::new(grande.as_bytes())).unwrap_err(),
        crate::transporte::FalhaTransporte::RespostaExcessiva
    );
    for ruim in [
        "",
        "HTTP/1.1 abc OK\r\n\r\n",
        "FTP 200\r\n\r\n",
        "HTTP/1.1 200 OK\r\nsem-dois-pontos\r\n\r\n",
    ] {
        assert_eq!(
            http::ler_resposta(&mut Cursor::new(ruim.as_bytes())).unwrap_err(),
            crate::transporte::FalhaTransporte::Rede,
            "{ruim:?}"
        );
    }
}

#[test]
fn captcha_so_em_css_e_js_nao_e_exigido_e_controle_servido_e() {
    assert!(!tela::captcha_exigido(&tela_sbm()));
    assert!(!tela::captcha_exigido(&tela_go()));
    assert!(tela::captcha_exigido(
        "<form id='frmLogin'><input name='txtInfraCaptcha'></form>"
    ));
    assert!(tela::captcha_exigido(
        "<div id='x' class='a h-captcha'></div>"
    ));
    assert!(!tela::segundo_fator_exigido(&tela_sbm()));
    assert!(tela::segundo_fator_exigido("<input id='txtCodigo2FA'>"));
}

#[test]
fn link_do_controle_aceita_so_as_formas_observadas() {
    let url = format!(
        "controlador.php?acao=procedimento_controlar&reset=1&infra_sistema=100000100&infra_unidade_atual=110000849&infra_hash={HASH}"
    );
    assert_eq!(
        tela::link_do_controle(&painel(&onclick_do_controle())).unwrap(),
        Some(url.clone())
    );
    let janela = format!("window.location.href='{url}';");
    assert_eq!(tela::link_do_controle(&painel(&janela)).unwrap(), Some(url));
    assert_eq!(tela::link_do_controle(&controle("U")).unwrap(), None);
    for ruim in ["abrir('x')", "location.href='a' + b", "location.href=\"x\""] {
        assert_eq!(
            tela::link_do_controle(&painel(ruim)),
            Err(ErroLogin::Html(ErroHtml::valor(
                "link_do_controle_de_processos_nao_reconhecido"
            ))),
            "{ruim}"
        );
    }
}

#[test]
fn mensagem_de_recusa_ignora_o_aviso_de_navegador() {
    assert_eq!(tela::mensagem_de_recusa(&tela_sbm()), None);
    assert_eq!(
        tela::mensagem_de_recusa(&tela_recusada("Senha  inv\u{e1}lida.")),
        Some("Senha inv\u{e1}lida.".to_owned())
    );
    assert_eq!(
        tela::mensagem_de_recusa("<div class='infraMensagemErro'>Conta bloqueada</div>"),
        Some("Conta bloqueada".to_owned())
    );
}

#[cfg(unix)]
#[test]
fn gravacao_substitui_link_simbolico_nunca_o_segue() {
    let ambiente = Ambiente::novo();
    let alvo = ambiente.dir.join("alvo");
    privado(&alvo, "intacto");
    std::os::unix::fs::symlink(&alvo, &ambiente.arquivos.cookie).unwrap();
    let _erro =
        super::gravacao::gravar(&ambiente.arquivos, ORIGEM, &url_do_controle(), "A=1").unwrap_err();
    assert_eq!(Ambiente::ler(&alvo), "intacto");
    assert!(!ambiente.arquivos.url_controle.exists());
}

#[test]
fn geracao_confere_cookie_e_url_gravados_juntos() {
    let ambiente = Ambiente::novo();
    assert!(super::gravacao::mesma_geracao(&ambiente.arquivos, "u", "c"));
    let n = super::gravacao::gravar(&ambiente.arquivos, ORIGEM, &url_do_controle(), "A=1").unwrap();
    assert_eq!(n, 1);
    assert!(super::gravacao::mesma_geracao(
        &ambiente.arquivos,
        &url_do_controle(),
        "A=1"
    ));
    assert!(!super::gravacao::mesma_geracao(
        &ambiente.arquivos,
        &url_do_controle(),
        "A=2"
    ));
    let marcador = Ambiente::ler(&ambiente.geracao());
    assert!(!marcador.contains("A=1") && !marcador.contains(HASH));
}

// Os reprodutores da #224 (URL e cookie de gerações distintas) viraram
// asserções do comportamento corrigido em `tests_snapshot` (#222).

#[cfg(unix)]
#[test]
fn gravacao_atravessa_ancestral_link_simbolico_como_home_em_var_home() {
    // Silverblue: `/home` → `/var/home`; a sessão vai para o diretório real.
    let ambiente = Ambiente::novo();
    let real = ambiente.dir.join("var").join("home");
    std::fs::create_dir_all(&real).unwrap();
    std::os::unix::fs::symlink(&real, ambiente.dir.join("home")).unwrap();
    let pela_casa = ambiente.dir.join("home");
    let arquivos = crate::configuracao::ArquivosDeAcesso {
        url_controle: pela_casa.join("url-controle"),
        cookie: pela_casa.join("cookie"),
        origem: ambiente.arquivos.origem.clone(),
    };
    super::gravacao::gravar(&arquivos, ORIGEM, &url_do_controle(), "A=1").unwrap();
    assert_eq!(Ambiente::ler(&real.join("cookie")), "A=1\n");
    assert!(super::gravacao::mesma_geracao(
        &arquivos,
        &url_do_controle(),
        "A=1"
    ));
}

#[cfg(unix)]
#[test]
fn gravacao_recusa_diretorio_gravavel_por_outros() {
    let ambiente = Ambiente::novo();
    let aberto = ambiente.dir.join("aberto");
    std::fs::create_dir(&aberto).unwrap();
    let arquivos = crate::configuracao::ArquivosDeAcesso {
        url_controle: aberto.join("url-controle"),
        cookie: aberto.join("cookie"),
        origem: ambiente.arquivos.origem.clone(),
    };
    for bits in [0o770, 0o707, 0o777] {
        crate::indice_local::tests::definir_modo(&aberto, bits);
        let _erro =
            super::gravacao::gravar(&arquivos, ORIGEM, &url_do_controle(), "A=1").unwrap_err();
        assert_eq!(std::fs::read_dir(&aberto).unwrap().count(), 0);
    }
    // `/tmp`-style (sticky): aceito.
    crate::indice_local::tests::definir_modo(&aberto, 0o1777);
    super::gravacao::gravar(&arquivos, ORIGEM, &url_do_controle(), "A=1").unwrap();
}

#[test]
fn gravacao_da_sessao_nao_deixa_temporario() {
    let ambiente = Ambiente::novo();
    super::gravacao::gravar(&ambiente.arquivos, ORIGEM, &url_do_controle(), "A=1").unwrap();
    super::gravacao::regravar_url_do_controle(&ambiente.arquivos, &url_do_controle(), "A=1")
        .unwrap();
    assert_eq!(
        ambiente.nomes(),
        [
            "cookie",
            "credenciais",
            "origem",
            "url-controle",
            "url-controle.geracao"
        ]
    );
}
