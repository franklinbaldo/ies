//! Fluxos de sucesso por família observada, com a sequência exata de
//! pedidos, cookies enviados, corpo do POST e arquivos gravados.
use super::tests_apoio::*;
use super::{Conexao, entrar_via};

fn controle_ok() -> RespostaHttpAlias {
    r200(&controle("UNIDADE-TESTE"), &[])
}
type RespostaHttpAlias = super::RespostaHttp;

#[test]
fn sip_com_acessar_loga_pela_raiz_e_grava_a_sessao() {
    let ambiente = Ambiente::novo();
    let mut roteiro = Roteiro::novo(vec![
        r302(
            LOGIN_SIP,
            &[
                "PHPSESSID=raiz; path=/; secure; HttpOnly; SameSite=Lax",
                "seitreinamento=seitreinamento; path=/",
            ],
        ),
        r200(&tela_acessar(), &[]),
        r302(
            "https://sei.exemplo.gov.br/sei/inicializar.php",
            &["PHPSESSID=autenticada; path=/; secure; HttpOnly"],
        ),
        r302(
            &url_do_controle().replace("https://sei.exemplo.gov.br/sei/", ""),
            &[],
        ),
        controle_ok(),
    ]);
    let resultado = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
    assert_eq!(
        roteiro.sequencia(),
        [
            format!("GET {RAIZ}"),
            format!("GET {LOGIN_SIP}"),
            format!("POST {LOGIN_SIP}"),
            "GET https://sei.exemplo.gov.br/sei/inicializar.php".to_owned(),
            format!("GET {}", url_do_controle()),
        ]
    );
    let p = &roteiro.pedidos;
    assert_eq!(p[0].cookie, "");
    assert_eq!(p[1].cookie, "PHPSESSID=raiz; seitreinamento=seitreinamento");
    assert_eq!(p[2].cookie, "PHPSESSID=raiz; seitreinamento=seitreinamento");
    assert_eq!(
        p[3].cookie,
        "PHPSESSID=autenticada; seitreinamento=seitreinamento"
    );
    assert_eq!(
        p[2].nomes,
        ["txtUsuario", "pwdSenha", "selOrgao", "Acessar", "hdnAcao"]
    );
    assert_eq!(
        p[2].corpo,
        format!("{CORPO_SENHA}&selOrgao=0&Acessar=&hdnAcao=2")
    );
    assert_eq!(p[2].origem, "https://sei.exemplo.gov.br");
    assert_eq!(p[2].referer, LOGIN_SIP);
    assert_eq!(resultado.conexao, Conexao::LoginRealizado);
    assert_eq!(resultado.unidade_exibida, "UNIDADE-TESTE");
    assert_eq!(resultado.instalacao, ORIGEM);
    assert_eq!(resultado.requisicoes, 5);
    assert_eq!(
        Ambiente::ler(&ambiente.arquivos.cookie).trim(),
        "PHPSESSID=autenticada; seitreinamento=seitreinamento"
    );
    assert_eq!(
        Ambiente::ler(&ambiente.arquivos.url_controle).trim(),
        url_do_controle()
    );
    assert_eq!(Ambiente::ler(&ambiente.arquivos.origem).trim(), ORIGEM);
    #[cfg(unix)]
    for arquivo in [
        &ambiente.arquivos.cookie,
        &ambiente.arquivos.url_controle,
        &ambiente.geracao(),
    ] {
        assert_eq!(Ambiente::modo(arquivo), 0o600, "{}", arquivo.display());
    }
    let geracao: serde_json::Value =
        serde_json::from_str(&Ambiente::ler(&ambiente.geracao())).unwrap();
    assert_eq!(geracao["geracao"], 1);
    assert_eq!(geracao["instalacao"], ORIGEM);
    // Sem temporários deixados para trás.
    assert_eq!(
        ambiente.nomes(),
        [
            "cookie",
            "credenciais",
            "origem",
            "url-controle",
            "url-controle.geracao",
            "url-controle.lock"
        ]
    );
    assert!(roteiro.esgotado());
}

#[test]
fn sbm_acessar_desabilitado_pelo_onsubmit_nao_participa_e_infra_url_e_preservado() {
    let ambiente = Ambiente::novo();
    let mut roteiro = Roteiro::novo(vec![
        r302("https://sei.exemplo.gov.br/sei/", &[]),
        r302(
            LOGIN_SIP_INFRA_URL,
            &[
                "PIINGRESSAFFINITY=1793.606|9ad4; Path=/; Secure; HttpOnly",
                "PHPSESSID=p1; path=/",
            ],
        ),
        r200(
            &tela_sbm(),
            &["_cfuvid=x.1-2; HttpOnly; SameSite=None; Secure; Path=/; Domain=exemplo.gov.br"],
        ),
        r302(&url_do_controle(), &[]),
        controle_ok(),
    ]);
    let resultado = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
    assert_eq!(
        roteiro.sequencia(),
        [
            format!("GET {RAIZ}"),
            "GET https://sei.exemplo.gov.br/sei/".to_owned(),
            format!("GET {LOGIN_SIP_INFRA_URL}"),
            format!("POST {LOGIN_SIP_INFRA_URL}"),
            format!("GET {}", url_do_controle()),
        ]
    );
    let post = &roteiro.pedidos[3];
    assert_eq!(
        post.nomes,
        ["txtUsuario", "pwdSenha", "selOrgao", "hdnAcao"]
    );
    assert_eq!(post.corpo, format!("{CORPO_SENHA}&selOrgao=0&hdnAcao=2"));
    // Domain que contém o host é aceito; tudo segue só para o host.
    assert_eq!(
        post.cookie,
        "PIINGRESSAFFINITY=1793.606|9ad4; PHPSESSID=p1; _cfuvid=x.1-2"
    );
    assert_eq!(resultado.requisicoes, 5);
}

#[test]
fn http_do_mesmo_host_e_promovido_a_https_sem_pedir_a_url_http() {
    let ambiente = Ambiente::novo();
    let mut roteiro = Roteiro::novo(vec![
        r302("http://sei.exemplo.gov.br/sei/", &[]),
        r302(LOGIN_SIP_INFRA_URL, &["PHPSESSID=u1; path=/; secure"]),
        r200(&tela_sbm(), &[]),
        r302(&url_do_controle(), &[]),
        controle_ok(),
    ]);
    entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
    assert_eq!(roteiro.pedidos[1].url, "https://sei.exemplo.gov.br/sei/");
    assert!(
        roteiro
            .pedidos
            .iter()
            .all(|p| p.url.starts_with("https://"))
    );
}

#[test]
fn modulo_proprio_resolve_action_relativa_contra_a_url_do_modulo() {
    let ambiente = Ambiente::novo();
    super::tests_apoio::privado(
        &ambiente.credenciais,
        "usuario=aluno.teste\nsenha=x\norgao=SEAD\n",
    );
    let modulo = format!("https://{MODULO_GO}");
    let mut roteiro = Roteiro::novo(vec![
        r302("https://sei.exemplo.gov.br/sei/", &[]),
        r302(&format!("http://{MODULO_GO}"), &[]),
        r200(
            &tela_go(),
            &[
                "PHPSESSID=g1; path=/; secure; HttpOnly",
                "TS011e0291=01ab; Path=/; Domain=.sei.exemplo.gov.br",
            ],
        ),
        r302(&url_do_controle(), &[]),
        controle_ok(),
    ]);
    entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
    let post = &roteiro.pedidos[3];
    assert_eq!(roteiro.pedidos[2].url, modulo);
    assert_eq!(
        post.url,
        "https://sei.exemplo.gov.br/sei/modulos/goias/login-sei/login.php?sigla_orgao_sistema=GOVERNADORIA&sigla_sistema=SEI&infra_url=L3NlaS8="
    );
    assert_eq!(post.referer, modulo);
    assert_eq!(
        post.corpo,
        "txtUsuario=aluno.teste&pwdSenha=x&selOrgao=5&hdnAcao=2"
    );
    assert_eq!(post.cookie, "PHPSESSID=g1; TS011e0291=01ab");
}

#[test]
fn painel_de_controle_segue_so_o_link_do_menu_publicado() {
    let ambiente = Ambiente::novo();
    let painel_url = format!(
        "https://{HOST}/sei/controlador.php?acao=painel_controle_visualizar&infra_hash={HASH}"
    );
    let destino = format!(
        "https://{HOST}/sei/controlador.php?acao=procedimento_controlar&reset=1&infra_sistema=100000100&infra_unidade_atual=110000849&infra_hash={HASH}"
    );
    let mut roteiro = Roteiro::novo(vec![
        r302(LOGIN_SIP, &[]),
        r200(&tela_acessar(), &["PHPSESSID=a; path=/"]),
        r302(&painel_url, &[]),
        r200(&painel(&onclick_do_controle()), &[]),
        controle_ok(),
    ]);
    let resultado = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
    assert_eq!(roteiro.pedidos[3].url, painel_url);
    assert_eq!(roteiro.pedidos[4].url, destino);
    assert_eq!(resultado.requisicoes, 5);
    assert_eq!(
        Ambiente::ler(&ambiente.arquivos.url_controle).trim(),
        destino
    );
}

#[test]
fn sessao_gravada_valida_dispensa_o_login_com_um_get() {
    let ambiente = Ambiente::novo();
    ambiente.gravar_sessao(&url_do_controle(), "PHPSESSID=gravada");
    let mut roteiro = Roteiro::novo(vec![controle_ok()]);
    let mut pedido = ambiente.pedido(false);
    pedido.credenciais = None;
    let resultado = entrar_via(&pedido, &mut roteiro).unwrap();
    assert_eq!(roteiro.sequencia(), [format!("GET {}", url_do_controle())]);
    assert_eq!(roteiro.pedidos[0].cookie, "PHPSESSID=gravada");
    assert_eq!(resultado.conexao, Conexao::SessaoExistenteValida);
    assert_eq!(resultado.requisicoes, 1);
    assert_eq!(resultado.unidade_exibida, "UNIDADE-TESTE");
    assert_eq!(
        Ambiente::ler(&ambiente.arquivos.cookie).trim(),
        "PHPSESSID=gravada"
    );
}

fn login_completo() -> Vec<RespostaHttpAlias> {
    vec![
        r302(LOGIN_SIP, &[]),
        r200(&tela_acessar(), &["PHPSESSID=nova; path=/"]),
        r302(&url_do_controle(), &[]),
        controle_ok(),
    ]
}

#[test]
fn sessao_vencida_por_tela_de_login_ou_302_vai_ao_login() {
    for vencida in [r200(&tela_acessar(), &[]), r302(LOGIN_SIP, &[])] {
        let ambiente = Ambiente::novo();
        ambiente.gravar_sessao(&url_do_controle(), "PHPSESSID=velha");
        let mut respostas = vec![vencida];
        respostas.extend(login_completo());
        let mut roteiro = Roteiro::novo(respostas);
        let resultado = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
        assert_eq!(roteiro.pedidos[1].url, RAIZ);
        assert_eq!(
            roteiro.pedidos[1].cookie, "",
            "pote novo, sem a sessão velha"
        );
        assert_eq!(resultado.conexao, Conexao::LoginRealizado);
        assert_eq!(resultado.requisicoes, 5);
        assert_eq!(
            Ambiente::ler(&ambiente.arquivos.cookie).trim(),
            "PHPSESSID=nova"
        );
    }
}

#[test]
fn forcar_dispensa_a_conferencia_e_a_geracao_avanca() {
    let ambiente = Ambiente::novo();
    let mut roteiro = Roteiro::novo(login_completo());
    entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
    let mut roteiro = Roteiro::novo(login_completo());
    let resultado = entrar_via(&ambiente.pedido(true), &mut roteiro).unwrap();
    assert_eq!(roteiro.pedidos[0].url, RAIZ);
    assert_eq!(resultado.requisicoes, 4);
    let geracao: serde_json::Value =
        serde_json::from_str(&Ambiente::ler(&ambiente.geracao())).unwrap();
    assert_eq!(geracao["geracao"], 2);
}

#[test]
fn arquivos_de_geracoes_misturadas_vao_ao_login_sem_o_get() {
    let ambiente = Ambiente::novo();
    let mut roteiro = Roteiro::novo(login_completo());
    entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
    // Outro processo trocou só o cookie: o marcador não confere.
    privado(&ambiente.arquivos.cookie, "PHPSESSID=de-outra-geracao\n");
    let mut roteiro = Roteiro::novo(login_completo());
    entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
    assert_eq!(roteiro.pedidos[0].url, RAIZ);
}

#[test]
fn lock_de_outro_processo_impede_qualquer_pedido_ate_ser_liberado() {
    let ambiente = Ambiente::novo();
    let trava = super::gravacao::travar_o_login(
        &ambiente.arquivos.url_controle,
        std::time::Duration::from_millis(50),
    )
    .unwrap();
    let mut roteiro = Roteiro::novo(login_completo());
    let erro = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap_err();
    assert_eq!(erro, super::ErroLogin::LoginEmAndamentoEmOutroProcesso);
    assert!(roteiro.pedidos.is_empty());
    drop(trava);
    entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
}

#[test]
fn unidade_esperada_diferente_nao_grava_a_sessao() {
    let ambiente = Ambiente::novo();
    let mut roteiro = Roteiro::novo(login_completo());
    let mut pedido = ambiente.pedido(false);
    pedido.unidade_esperada = Some("OUTRA");
    let _erro = entrar_via(&pedido, &mut roteiro).unwrap_err();
    assert!(!ambiente.arquivos.cookie.exists());
    assert!(!ambiente.arquivos.url_controle.exists());
}

#[test]
fn arquivos_vazios_do_setup_vao_ao_login_sem_o_get_e_sao_regravados() {
    let ambiente = Ambiente::novo();
    privado(&ambiente.arquivos.url_controle, "");
    privado(&ambiente.arquivos.cookie, "");
    let mut roteiro = Roteiro::novo(login_completo());
    let resultado = entrar_via(&ambiente.pedido(false), &mut roteiro).unwrap();
    assert_eq!(roteiro.pedidos[0].url, RAIZ);
    assert_eq!(resultado.requisicoes, 4);
    assert_eq!(
        Ambiente::ler(&ambiente.arquivos.cookie).trim(),
        "PHPSESSID=nova"
    );
}
