//! Preparação do TLS local, parsing do proxy e SQLite efêmero da bancada.
use super::*;

pub(super) fn identidade(pasta: &Path) -> (PathBuf, Identity) {
    let cert = pasta.join("cert.pem");
    let key = pasta.join("key.pem");
    let p12 = pasta.join("identidade.p12");
    assert!(
        Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-sha256",
                "-days",
                "1",
                "-subj",
                "/CN=sei.test",
                "-addext",
                "subjectAltName=DNS:sei.test",
                "-keyout"
            ])
            .arg(&key)
            .arg("-out")
            .arg(&cert)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("openssl")
            .args(["pkcs12", "-export", "-passout", "pass:teste", "-inkey"])
            .arg(&key)
            .arg("-in")
            .arg(&cert)
            .arg("-out")
            .arg(&p12)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    (
        cert,
        Identity::from_pkcs12(&fs::read(p12).unwrap(), "teste").unwrap(),
    )
}

pub(super) fn aceitar(listener: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match listener.accept() {
            Ok((tcp, endereco)) => {
                assert!(endereco.ip().is_loopback());
                tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                tcp.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
                return Some(tcp);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    return None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("aceitar TLS de teste: {e}"),
        }
    }
}

pub(super) fn cabecalho(fonte: &mut impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut byte = [0];
    while !bytes.ends_with(b"\r\n\r\n") {
        assert!(bytes.len() < 16_384);
        fonte.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
    }
    bytes
}

pub(super) fn pedido(fonte: &mut impl Read) -> (Vec<u8>, String, String) {
    let mut bytes = cabecalho(fonte);
    let cabecalho = std::str::from_utf8(&bytes).unwrap();
    let primeira = cabecalho.lines().next().unwrap();
    let mut partes = primeira.split_whitespace();
    let metodo = partes.next().unwrap().to_owned();
    let alvo = partes.next().unwrap();
    let acao = alvo
        .split_once('?')
        .and_then(|(_, q)| {
            q.split('&').find_map(|item| {
                item.strip_prefix("acao=")
                    .or_else(|| item.strip_prefix("acao_ajax="))
            })
        })
        .unwrap_or("sem_acao")
        .to_owned();
    let tamanho = cabecalho
        .lines()
        .find_map(|linha| {
            let (nome, valor) = linha.split_once(':')?;
            nome.eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 64_000);
    let mut corpo = vec![0; tamanho];
    fonte.read_exact(&mut corpo).unwrap();
    bytes.extend(corpo);
    (bytes, metodo, acao)
}

pub(super) fn preparar_banco(anotacao: Option<&str>) -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("ORIGEM").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    documentos_simulados::registrar_processo(
        &banco,
        &origem,
        CENARIO,
        &Processo {
            id: "10".into(),
            nup: NUP.into(),
            unidade: "ORIGEM".into(),
        },
    )
    .unwrap();
    processos::guardar(
        &mut banco,
        &origem,
        CENARIO,
        &ProcessoSimulado {
            id: "10".into(),
            numero: NUP.into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: anotacao.map(str::to_owned),
            destinos: vec![
                Destino {
                    id: "2".into(),
                    sigla: "DESTINO".into(),
                    descricao: "DESTINO - Unidade de teste".into(),
                },
                Destino {
                    id: "3".into(),
                    sigla: "ARQUIVO".into(),
                    descricao: "ARQUIVO - Unidade de teste".into(),
                },
            ],
        },
    )
    .unwrap();
    (banco, origem)
}
