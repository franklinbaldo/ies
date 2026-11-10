//! BDD-565-01, hipótese de colunas sem cassette específica:
//! CLI real -> HTTPS local -> anti-IES -> SQLite; duas leituras independentes.
use super::*;
use iesei::colunas_detalhe::{self, Coluna, Configuracao, Opcao};

fn configurar_banco(banco: &mut Connection, instalacao: &Instalacao) {
    banco_cassettes::iniciar(banco).unwrap();
    detalhada_cassettes::guardar(
        banco,
        CENARIO,
        &ControleDetalhado {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            processos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    colunas_detalhe::guardar(
        banco,
        instalacao,
        CENARIO,
        &Configuracao {
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            opcoes: vec![
                Opcao {
                    coluna: Coluna::Atribuicao,
                    marcada: true,
                },
                Opcao {
                    coluna: Coluna::Anotacao,
                    marcada: false,
                },
                Opcao {
                    coluna: Coluna::TipoProcesso,
                    marcada: false,
                },
            ],
        },
    )
    .unwrap();
}

fn atender_quatro_gets(listener: &TcpListener) {
    let mut banco = Connection::open_in_memory().unwrap();
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    configurar_banco(&mut banco, &instalacao);
    for _ in 0..4 {
        let mut tcp = aceitar(listener);
        atender_http(&mut tcp, Some((&banco, &instalacao, CENARIO))).unwrap();
    }
    let opcoes = colunas_detalhe::ler(&banco, &instalacao, CENARIO)
        .unwrap()
        .unwrap()
        .opcoes;
    assert_eq!(opcoes.iter().filter(|opcao| opcao.marcada).count(), 1);
}

fn encaminhar_quatro_gets(
    proxy: &TcpListener,
    destino: std::net::SocketAddr,
    identidade: Identity,
) {
    let tls = TlsAcceptor::new(identidade).unwrap();
    let mut rotas = Vec::new();
    for _ in 0..4 {
        let mut tcp = aceitar(proxy);
        assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
        tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
            .unwrap();
        let mut canal = tls.accept(tcp).unwrap();
        let pedido = cabecalho(&mut canal);
        assert!(pedido.starts_with(b"GET /sei/controlador.php?acao="));
        let rota = String::from_utf8_lossy(&pedido);
        rotas.push(rota.lines().next().unwrap().to_owned());
        let mut upstream = TcpStream::connect(destino).unwrap();
        upstream.write_all(&pedido).unwrap();
        upstream.shutdown(Shutdown::Write).unwrap();
        let mut resposta = Vec::new();
        upstream.read_to_end(&mut resposta).unwrap();
        canal.write_all(&resposta).unwrap();
        canal.flush().unwrap();
    }
    assert!(rotas[0].contains("acao=procedimento_controlar"));
    assert!(rotas[1].contains("acao=procedimento_configurar_detalhe"));
    assert!(rotas[2].contains("acao=procedimento_controlar"));
    assert!(rotas[3].contains("acao=procedimento_configurar_detalhe"));
}

fn cli_colunas(
    pasta: &Pasta,
    cert: &Path,
    proxy_porta: u16,
    url: &Path,
    cookie: &Path,
    origem: &Path,
    definir: bool,
) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ies"));
    if definir {
        cmd.args(["--json", "unidade", "colunas", "--definir", "todas"]);
    } else {
        cmd.args(["--json", "unidade", "colunas"]);
    }
    cmd.env("IES_URL_CONTROLE_FILE", url)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("SSL_CERT_FILE", cert)
        .env("https_proxy", format!("http://127.0.0.1:{proxy_porta}"))
        .env("HOME", &pasta.0)
        .env_remove("HTTPS_PROXY")
        .env_remove("all_proxy")
        .env_remove("ALL_PROXY")
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE")
        .output()
        .unwrap()
}

#[test]
fn bdd_565_01_previa_nao_grava_e_segunda_cli_rele_original() {
    let pasta = Pasta::nova();
    fs::set_permissions(&pasta.0, fs::Permissions::from_mode(0o700)).unwrap();
    let (cert, identidade) = certificado(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    let hash = ["infra", "hash"].join("_");
    let url = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&{hash}={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let destino = servidor.local_addr().unwrap();
    let backend = thread::spawn(move || atender_quatro_gets(&servidor));
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let tunel = thread::spawn(move || encaminhar_quatro_gets(&proxy, destino, identidade));

    let previa = cli_colunas(&pasta, &cert, porta, &url, &cookie, &origem, true);
    assert_eq!(
        previa.status.code(),
        Some(0),
        "{:?}",
        String::from_utf8_lossy(&previa.stderr)
    );
    let valor = json(&previa);
    assert_eq!(valor["antes"], serde_json::json!(["atribuicao"]));
    assert_eq!(
        valor["alvo"],
        serde_json::json!(["atribuicao", "anotacao", "tipo"])
    );
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert_eq!(valor["depois"], Value::Null);
    assert_eq!(valor["envio"]["metodo"], "POST");

    let releitura = cli_colunas(&pasta, &cert, porta, &url, &cookie, &origem, false);
    assert_eq!(
        releitura.status.code(),
        Some(0),
        "{:?}",
        String::from_utf8_lossy(&releitura.stderr)
    );
    let valor = json(&releitura);
    assert_eq!(valor["unidade"], "TESTE");
    // O GET independente publica os mesmos checks do SQLite, nunca o plano da prévia.
    let opcoes = valor["opcoes"].as_array().unwrap();
    assert_eq!(opcoes.iter().filter(|o| o["marcada"] == true).count(), 1);
    assert_eq!(opcoes[0]["coluna"], "atribuicao");
    assert_eq!(opcoes[0]["marcada"], true);
    assert_eq!(opcoes[1]["marcada"], false);
    assert_eq!(opcoes[2]["marcada"], false);
    tunel.join().unwrap();
    backend.join().unwrap();
}
