//! #60 BDD-60-06: lote de documentos de processos distintos pelo binário real.
//! Hipótese sintética; não valida equivalência ao SEI de treinamento.
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::cancelamento_documento::{self, Aplicacao};
use iesei::documento_versoes::{self, DocumentoSimulado, Revisao};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::processos::{self, ProcessoSimulado};
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
pub(super) const DOC_A: &str = "0011519";
pub(super) const DOC_B: &str = "0011520";
pub(super) const NUP_A: &str = "00001.000001/2026-01";
pub(super) const NUP_B: &str = "00001.000002/2026-02";

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

fn semear_documento_sem_src(banco: &Connection, origem: &Instalacao) {
    // Hipotese minima no SQLite: identidade publicada, mas sem link de conteudo.
    use iesei::documentos_simulados::{self as internos, Processo, Tipo};
    internos::registrar_processo(
        banco,
        origem,
        CENARIO,
        &Processo {
            id: "7".into(),
            nup: NUP_A.into(),
            unidade: "TESTE".into(),
        },
    )
    .unwrap();
    internos::registrar_tipo(
        banco,
        origem,
        CENARIO,
        "7",
        &Tipo {
            id: "291".into(),
            nome: "Despacho".into(),
        },
    )
    .unwrap();
    let sem_conteudo =
        internos::criar(banco, origem, CENARIO, "7", "291", "Metadados sem conteudo").unwrap();
    assert_eq!(sem_conteudo.numero_sei, "1000001");
}

fn estado(
    com_segundo: bool,
    sem_src: bool,
    cancelado: bool,
    duplicado: bool,
) -> (Connection, Instalacao) {
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
    // O NUP da recusa precisa ser navegável: sem processo cadastrado o
    // anti-IES devolve 503 na pesquisa e testa apenas transporte, não
    // o BDD-60-05. A árvore mantém somente DOC_A sob NUP_A.
    processos::guardar(
        &mut banco,
        &origem,
        CENARIO,
        &ProcessoSimulado {
            id: "7".into(),
            numero: NUP_A.into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
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

    if duplicado {
        // Hipótese mínima para BDD-60-05: dois nós com mesmo número SEI
        // e IDs distintos no mesmo processo, sem qualquer corpo substituto.
        documento_versoes::guardar(
            &mut banco,
            CENARIO,
            &DocumentoSimulado {
                instalacao: origem.clone(),
                id_documento: "44".into(),
                id_procedimento: "7".into(),
                numero_processo: NUP_A.into(),
                numero: DOC_A.into(),
                tipo: "Despacho".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "TESTE".into(),
                    unidade: "TESTE".into(),
                    modificacao: "01/11/2026".into(),
                    html_corpo: "<p>Outro corpo que não pode substituir DOC_A</p>".into(),
                }],
            },
        )
        .unwrap();
    }
    if sem_src {
        semear_documento_sem_src(&banco, &origem);
    }
    if cancelado {
        assert_eq!(
            cancelamento_documento::aplicar(
                &banco,
                &origem,
                CENARIO,
                "42",
                "7",
                "Motivo sintético"
            )
            .unwrap(),
            Aplicacao::Aplicada
        );
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
pub(super) fn executar(com_segundo: bool, args: &[&str]) -> (Output, Vec<String>) {
    executar_com_estado(com_segundo, false, false, false, args)
}

pub(super) fn executar_sem_src(args: &[&str]) -> (Output, Vec<String>) {
    executar_com_estado(true, true, false, false, args)
}

pub(super) fn executar_cancelado(args: &[&str]) -> (Output, Vec<String>) {
    executar_com_estado(true, false, true, false, args)
}

pub(super) fn executar_numero_repetido(args: &[&str]) -> (Output, Vec<String>) {
    executar_com_estado(true, false, false, true, args)
}

fn executar_com_estado(
    com_segundo: bool,
    sem_src: bool,
    cancelado: bool,
    duplicado: bool,
    args: &[&str],
) -> (Output, Vec<String>) {
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
        let (db, instalacao) = estado(com_segundo, sem_src, cancelado, duplicado);
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

pub(super) fn json(saida: &Output) -> Value {
    serde_json::from_slice(&saida.stdout).unwrap_or_else(|e| {
        panic!(
            "JSON inválido: {e}; exit={:?}; stderr={}",
            saida.status.code(),
            String::from_utf8_lossy(&saida.stderr)
        )
    })
}
