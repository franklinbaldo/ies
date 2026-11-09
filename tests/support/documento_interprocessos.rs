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

#[path = "documento_interprocessos_estado.rs"]
mod estado_simulado;
use estado_simulado::{EstadoExtra, estado};

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
    executar_com_estado(com_segundo, EstadoExtra::Nenhum, args)
}

pub(super) fn executar_sem_src(args: &[&str]) -> (Output, Vec<String>) {
    executar_com_estado(true, EstadoExtra::SemSrc, args)
}

pub(super) fn executar_cancelado(args: &[&str]) -> (Output, Vec<String>) {
    executar_com_estado(true, EstadoExtra::Cancelado, args)
}

pub(super) fn executar_numero_repetido(args: &[&str]) -> (Output, Vec<String>) {
    executar_com_estado(true, EstadoExtra::NumeroRepetido, args)
}

/// A árvore oferece um href válido; a mesma identidade é revogada somente
/// no GET de conteúdo, antes de o anti-IES consultar o SQLite.
pub(super) fn executar_revogado_ao_visualizar(args: &[&str]) -> (Output, Vec<String>) {
    executar_com_estado(true, EstadoExtra::RevogadoAoVisualizar, args)
}

/// A identidade no título do GET diverge da árvore previamente lida.
pub(super) fn executar_titulo_trocado(args: &[&str]) -> (Output, Vec<String>) {
    executar_com_estado(true, EstadoExtra::TituloTrocadoAoVisualizar, args)
}

/// Espia apenas a primeira linha sem consumir a requisição HTTP original.
fn eh_get_de_conteudo(tcp: &TcpStream) -> bool {
    let mut inicio = [0_u8; 1024];
    let tamanho = tcp.peek(&mut inicio).unwrap();
    let linha = String::from_utf8_lossy(&inicio[..tamanho]);
    linha.starts_with("GET ") && linha.contains("acao=documento_visualizar")
}

/// A árvore já foi publicada; a origem altera o título antes do GET do
/// conteúdo. Hipótese de corrida sintética, não transição capturada na UnB.
fn trocar_titulo_antes_de_servir_corpo(
    tcp: &TcpStream,
    banco: &Connection,
    instalacao: &Instalacao,
) -> bool {
    if !eh_get_de_conteudo(tcp) {
        return false;
    }
    let alterados = banco
        .execute(
            "UPDATE documento_simulado SET numero=?1
             WHERE origem=?2 AND cenario=?3 AND id_documento=?4 AND numero=?5",
            rusqlite::params![DOC_B, instalacao.url_base(), CENARIO, "42", DOC_A],
        )
        .unwrap();
    assert_eq!(alterados, 1, "a hipótese altera a identidade do documento");
    true
}

/// Simula a revogação do href já publicado entre a árvore e a resposta
/// documental. O peek não consome bytes: o servidor recebe o HTTP original.
fn revogar_antes_de_servir_corpo(
    tcp: &TcpStream,
    banco: &Connection,
    instalacao: &Instalacao,
) -> bool {
    if !eh_get_de_conteudo(tcp) {
        return false;
    }
    assert_eq!(
        cancelamento_documento::aplicar(
            banco,
            instalacao,
            CENARIO,
            "42",
            "7",
            "Revogado entre leituras"
        )
        .unwrap(),
        Aplicacao::Aplicada
    );
    true
}

fn executar_com_estado(
    com_segundo: bool,
    modo: EstadoExtra,
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
        let (db, instalacao) = estado(com_segundo, modo);
        let mut transicao = false;
        while let Some(mut tcp) = aceitar_ate(&servidor, &parar_servidor) {
            if !transicao {
                transicao = match modo {
                    EstadoExtra::RevogadoAoVisualizar => {
                        revogar_antes_de_servir_corpo(&tcp, &db, &instalacao)
                    }
                    EstadoExtra::TituloTrocadoAoVisualizar => {
                        trocar_titulo_antes_de_servir_corpo(&tcp, &db, &instalacao)
                    }
                    _ => false,
                };
            }
            atender_http(&mut tcp, Some((&db, &instalacao, CENARIO))).unwrap();
        }
        if matches!(modo, EstadoExtra::RevogadoAoVisualizar) {
            assert!(transicao, "CLI não tentou seguir a URL antes oferecida");
            assert!(matches!(
                cancelamento_documento::consultar(&db, &instalacao, CENARIO, "42", "7"),
                Ok(Some(cancelamento_documento::Situacao::Cancelado { .. }))
            ));
        }
        if matches!(modo, EstadoExtra::TituloTrocadoAoVisualizar) {
            assert!(transicao, "CLI não buscou o corpo depois de ler a árvore");
            let observado: String = db.query_row(
                "SELECT numero FROM documento_simulado
                 WHERE origem=?1 AND cenario=?2 AND id_documento='42'",
                rusqlite::params![instalacao.url_base(), CENARIO],
                |linha| linha.get(0),
            ).unwrap();
            assert_eq!(observado, DOC_B, "a transição persiste no SQLite");
            assert_eq!(
                documento_versoes::ler(&db, &instalacao, CENARIO, "43")
                    .unwrap().unwrap().numero,
                DOC_B,
                "o segundo documento não foi alterado"
            );
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
