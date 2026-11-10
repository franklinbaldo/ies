//! Executor HTTPS local do BDD-633 com a CLI real.
use super::*;


fn estado_final(cena: Cena, banco: &Connection, instalacao: &Instalacao) -> (String, String) {
    let controle = banco_cassettes::ler(&banco, &instalacao, CENARIO)
        .unwrap()
        .unwrap();
    let processo = processos::ler(&banco, &instalacao, CENARIO, "10")
        .unwrap()
        .unwrap();
    if matches!(cena, Cena::ValidacaoExigida | Cena::ValidacaoAutorizada) {
        let esperado = if matches!(cena, Cena::ValidacaoAutorizada) {
            iesei::validacao_acesso::Validacao::Realizada
        } else {
            iesei::validacao_acesso::Validacao::Exigida
        };
        assert_eq!(
            iesei::validacao_acesso::ler(&banco, &instalacao, CENARIO, "10").unwrap(),
            Some(esperado),
        );
    }
    (controle.recebidos.linhas[0].numero.clone(), processo.numero)
}

pub(super) fn executar(cena: Cena) -> (Output, Vec<String>, (String, String)) {
    let pasta = Pasta::nova();
    let (cert, identidade_tls) = identidade(&pasta.0, "sei.test");
    let origem = pasta.privado("origem", ORIGEM);
    let chave = ["infra", "hash"].join("_");
    let controle = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual=110000001&{chave}={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_servidor = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        let (banco, instalacao) = estado(cena);
        while let Some(mut cliente) = aceitar(&escuta, &parar_servidor) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        estado_final(cena, &banco, &instalacao)
    });
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let registro = Arc::clone(&trilha);
    let parar_proxy = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade_tls).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            let (connect, _) = pedido_http(&mut tcp);
            assert!(connect.starts_with(b"CONNECT"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, evento) = pedido_http(&mut canal);
            let mut backend = TcpStream::connect(endereco).unwrap();
            backend.write_all(&bytes).unwrap();
            backend.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            backend.read_to_end(&mut resposta).unwrap();
            registro.lock().unwrap().push(evento);
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    // Senha de ensaio criada em execução e restrita ao arquivo 0600;
    // nenhum segredo real ou valor de senha é versionado.
    let credenciais = matches!(cena, Cena::ValidacaoAutorizada).then(|| {
        pasta.privado(
            "credenciais",
            &format!(
                "usuario=teste\nsenha=ensaio-{}\norgao=TESTE\n",
                std::process::id()
            ),
        )
    });
    let mut comando = Command::new(env!("CARGO_BIN_EXE_ies"));
    comando
        .args([
            "--json",
            "processo",
            PEDIDO,
            "--sem-andamentos",
            "--sem-propriedades",
            "--sem-marcadores",
        ])
        .env("IES_URL_CONTROLE_FILE", controle)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("SSL_CERT_FILE", cert)
        .env("https_proxy", format!("http://127.0.0.1:{porta}"))
        .env("HOME", &pasta.0)
        .env_remove("HTTPS_PROXY")
        .env_remove("all_proxy")
        .env_remove("ALL_PROXY")
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE");
    if let Some(caminho) = credenciais {
        comando.env("IES_CREDENCIAIS_FILE", caminho);
    }
    let saida = comando.output().unwrap();
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let identidade_final = servidor.join().unwrap();
    let eventos = trilha.lock().unwrap().clone();
    (saida, eventos, identidade_final)
}
