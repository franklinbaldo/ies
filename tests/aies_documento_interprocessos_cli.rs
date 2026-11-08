//! #60 BDD-60-06: lote de documentos de processos distintos pelo binário real.
//! Hipótese sintética; não valida equivalência ao SEI de treinamento.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::documento_versoes::{self, DocumentoSimulado, Revisao};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use native_tls::{Identity, TlsAcceptor};
use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::num::NonZeroU32;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-60-06-cli";
const DOC_A: &str = "0011519";
const DOC_B: &str = "0011520";
const NUP_A: &str = "00001.000001/2026-01";
const NUP_B: &str = "00001.000002/2026-02";

struct Pasta(PathBuf);
impl Pasta {
    fn nova() -> Self {
        let instante = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let caminho = std::env::temp_dir().join(format!(
            "aies-documento-interprocessos-{}-{}",
            std::process::id(),
            instante.as_nanos()
        ));
        fs::create_dir(&caminho).unwrap();
        Self(caminho)
    }
    fn privado(&self, nome: &str, texto: &str) -> PathBuf {
        let caminho = self.0.join(nome);
        fs::write(&caminho, texto).unwrap();
        fs::set_permissions(&caminho, fs::Permissions::from_mode(0o600)).unwrap();
        caminho
    }
}
impl Drop for Pasta {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn certificado(pasta: &Path) -> (PathBuf, Identity) {
    let cert = pasta.join("cert.pem");
    let key = pasta.join("key.pem");
    let p12 = pasta.join("identidade.p12");
    let estado = Command::new("openssl")
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
            "-keyout",
        ])
        .arg(&key)
        .arg("-out")
        .arg(&cert)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(estado.success(), "certificado efêmero não criado");
    let estado = Command::new("openssl")
        .args(["pkcs12", "-export", "-passout", "pass:teste", "-inkey"])
        .arg(&key)
        .arg("-in")
        .arg(&cert)
        .arg("-out")
        .arg(&p12)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(estado.success(), "identidade TLS efêmera não criada");
    (
        cert,
        Identity::from_pkcs12(&fs::read(p12).unwrap(), "teste").unwrap(),
    )
}

fn cabecalho(canal: &mut impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut b = [0u8; 1];
    while !bytes.ends_with(b"\r\n\r\n") {
        assert!(bytes.len() < 16_384, "cabeçalho HTTP excessivo");
        canal.read_exact(&mut b).unwrap();
        bytes.push(b[0]);
    }
    bytes
}

fn aceitar_ate(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    escuta.set_nonblocking(true).unwrap();
    loop {
        match escuta.accept() {
            Ok((tcp, peer)) => {
                assert!(peer.ip().is_loopback());
                tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                tcp.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
                return Some(tcp);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::Acquire) {
                    return None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("accept: {e}"),
        }
    }
}

fn estado(com_segundo: bool) -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    for (id, processo, nup, numero, corpo) in [
        (
            "42",
            "7",
            NUP_A,
            DOC_A,
            "Conteudo exclusivo do primeiro processo",
        ),
        (
            "43",
            "8",
            NUP_B,
            DOC_B,
            "Conteudo exclusivo do segundo processo",
        ),
    ]
    .into_iter()
    .take(if com_segundo { 2 } else { 1 })
    {
        documento_versoes::guardar(
            &mut banco,
            CENARIO,
            &DocumentoSimulado {
                instalacao: origem.clone(),
                id_documento: id.into(),
                id_procedimento: processo.into(),
                numero_processo: nup.into(),
                numero: numero.into(),
                tipo: "Despacho".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "TESTE".into(),
                    unidade: "TESTE".into(),
                    modificacao: "01/11/2026".into(),
                    html_corpo: format!("<p>{corpo}</p>"),
                }],
            },
        )
        .unwrap();
    }
    (banco, origem)
}

fn acao_do_pedido(cab: &[u8]) -> String {
    let primeira = std::str::from_utf8(cab).unwrap().lines().next().unwrap();
    let metodo = primeira.split_whitespace().next().unwrap();
    let acao = primeira
        .split("acao=")
        .nth(1)
        .and_then(|s| s.split(&['&', ' '][..]).next())
        .unwrap_or("sem_acao");
    format!("{metodo} {acao}")
}

/// TCP e TLS exclusivamente no loopback, sem mudanças no transporte de produção.
/// A saída do subprocesso encerra os dois ouvintes sem assumir número fixo
/// de leituras: um segundo DOC pode ou não reutilizar ação de pesquisa.
fn executar(com_segundo: bool, args: &[&str]) -> (Output, Vec<String>) {
    let pasta = Pasta::nova();
    let (cert, identidade) = certificado(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    let nome_hash = ["infra", "hash"].join("_");
    let url = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&{nome_hash}={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_servidor = Arc::clone(&parar);
    let simulador = thread::spawn(move || {
        let (db, instalacao) = estado(com_segundo);
        while let Some(mut tcp) = aceitar_ate(&servidor, &parar_servidor) {
            atender_http(&mut tcp, Some((&db, &instalacao, CENARIO))).unwrap();
        }
    });
    let pedidos = Arc::new(Mutex::new(Vec::<String>::new()));
    let pedidos_proxy = Arc::clone(&pedidos);
    let parar_proxy = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade).unwrap();
        while let Some(mut tcp) = aceitar_ate(&proxy, &parar_proxy) {
            assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).expect("TLS de teste");
            let mut destino = TcpStream::connect(endereco).unwrap();
            let cab = cabecalho(&mut canal);
            pedidos_proxy.lock().unwrap().push(acao_do_pedido(&cab));
            let texto = std::str::from_utf8(&cab).unwrap();
            let tamanho = texto
                .lines()
                .find_map(|l| {
                    let (nome, valor) = l.split_once(':')?;
                    nome.eq_ignore_ascii_case("content-length")
                        .then(|| valor.trim().parse::<usize>().unwrap())
                })
                .unwrap_or(0);
            assert!(tamanho < 64_000);
            destino.write_all(&cab).unwrap();
            let mut corpo = vec![0; tamanho];
            canal.read_exact(&mut corpo).unwrap();
            destino.write_all(&corpo).unwrap();
            destino.flush().unwrap();
            destino.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            destino.read_to_end(&mut resposta).unwrap();
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env("IES_URL_CONTROLE_FILE", url)
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
        .env_remove("IES_CREDENCIAIS_FILE")
        .output()
        .expect("binário ies");
    parar.store(true, Ordering::Release);
    tunel.join().expect("proxy TLS");
    simulador.join().expect("anti-IES SQLite");
    let trilha = pedidos.lock().unwrap().clone();
    (saida, trilha)
}

fn json(saida: &Output) -> Value {
    serde_json::from_slice(&saida.stdout).unwrap_or_else(|e| {
        panic!(
            "JSON inválido: {e}; exit={:?}; stderr={}",
            saida.status.code(),
            String::from_utf8_lossy(&saida.stderr)
        )
    })
}

#[test]
fn bdd_60_06_lote_interprocessos_preserva_identidades_e_conteudos() {
    let (saida, trilha) = executar(true, &["--json", "documento", DOC_A, DOC_B, "--sem-cache"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = json(&saida);
    let documentos = valor["documentos"].as_array().expect("lista de documentos");
    assert_eq!(documentos.len(), 2, "{valor}");
    for (doc, numero, nup, exclusivo, alheio) in [
        (
            &documentos[0],
            DOC_A,
            NUP_A,
            "primeiro processo",
            "segundo processo",
        ),
        (
            &documentos[1],
            DOC_B,
            NUP_B,
            "segundo processo",
            "primeiro processo",
        ),
    ] {
        assert_eq!(doc["numero"], numero);
        assert_eq!(doc["processo"]["nup"], nup);
        assert!(doc["texto"].as_str().unwrap().contains(exclusivo), "{doc}");
        assert!(!doc["texto"].as_str().unwrap().contains(alheio), "{doc}");
    }
    assert_eq!(valor["requisicoes"].as_u64(), Some(trilha.len() as u64));
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "não há escrita, apenas navegação permitida: {trilha:?}"
    );
}

#[test]
fn bdd_60_06_lote_nao_aceita_documento_sem_identidade_comprovada() {
    let (saida, trilha) = executar(false, &["--json", "documento", DOC_A, DOC_B, "--sem-cache"]);
    assert_ne!(
        saida.status.code(),
        Some(0),
        "não pode usar o conteúdo de A como B"
    );
    let valor = json(&saida);
    assert_eq!(valor["documento"], DOC_B, "{valor}");
    assert_eq!(valor["documentos_lidos_antes"], 1, "{valor}");
    assert!(
        valor.get("documentos").is_none(),
        "não publicar lote parcial: {valor}"
    );
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "a falha não autoriza escrita: {trilha:?}"
    );
}

#[test]
fn bdd_60_05_cli_recusa_documento_de_outro_processo() {
    let (saida, trilha) = executar(
        true,
        &[
            "--json",
            "documento",
            DOC_B,
            "--processo",
            NUP_A,
            "--sem-cache",
        ],
    );
    assert_ne!(
        saida.status.code(),
        Some(0),
        "documento alheio não pode ser lido"
    );
    let valor = json(&saida);
    assert_eq!(valor["documento"], DOC_B, "{valor}");
    assert!(valor.get("documentos").is_none(), "{valor}");
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "a recusa não autoriza escrita: {trilha:?}"
    );
}
