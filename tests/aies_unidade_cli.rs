//! #355 AIES-01/03/04: o binário ies real consome o anti-IES via TLS de teste.
//! Nenhuma flag insegura ou adaptação do transporte de produção.
#![cfg(unix)]

use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::detalhada_cassettes::{self, ControleDetalhado, LinhaDetalhada};
use iesei::instalacao::Instalacao;
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use native_tls::{Identity, TlsAcceptor};
use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::falha;

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "aies-unidade-cli";

enum VisaoFixture {
    Ausente,
    Resumida,
    Detalhada,
}

struct Pasta(PathBuf);
impl Pasta {
    fn nova() -> Self {
        let tempo = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let pasta = std::env::temp_dir().join(format!(
            "aies-unidade-{}-{}",
            std::process::id(),
            tempo.as_nanos()
        ));
        fs::create_dir(&pasta).unwrap();
        Self(pasta)
    }
    fn privado(&self, nome: &str, texto: &str) -> PathBuf {
        let arquivo = self.0.join(nome);
        fs::write(&arquivo, texto).unwrap();
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();
        arquivo
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
    assert!(estado.success(), "não gerou certificado efêmero");
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
    assert!(estado.success(), "não gerou identidade TLS");
    (
        cert,
        Identity::from_pkcs12(&fs::read(p12).unwrap(), "teste").unwrap(),
    )
}

fn aceitar(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let desde = Instant::now();
    loop {
        match listener.accept() {
            Ok((tcp, peer)) => {
                assert!(peer.ip().is_loopback());
                tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                tcp.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
                return tcp;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    desde.elapsed() < Duration::from_secs(10),
                    "sem GET de teste"
                );
                thread::sleep(Duration::from_millis(25));
            }
            Err(e) => panic!("accept: {e}"),
        }
    }
}

fn cabecalho(fluxo: &mut impl Read) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut c = [0];
    while !buf.ends_with(b"\r\n\r\n") {
        assert!(buf.len() < 8192);
        fluxo.read_exact(&mut c).unwrap();
        buf.push(c[0]);
    }
    buf
}

fn executar(snapshot: bool, esperada: Option<&str>) -> Output {
    executar_visao(
        if snapshot {
            VisaoFixture::Resumida
        } else {
            VisaoFixture::Ausente
        },
        esperada,
    )
}

fn executar_visao(visao: VisaoFixture, esperada: Option<&str>) -> Output {
    let pasta = Pasta::nova();
    let (cert, identidade) = certificado(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    // Assinatura sintética, montada apenas em arquivo temporário 0600.
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
    let simulador = thread::spawn(move || {
        let mut db = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&db).unwrap();
        let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
        if matches!(visao, VisaoFixture::Resumida) {
            banco_cassettes::guardar(
                &mut db,
                CENARIO,
                &ControleResumido {
                    instalacao: instalacao.clone(),
                    unidade: UnidadeExibida::nova("TESTE").unwrap(),
                    recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
                    gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
                },
            )
            .unwrap();
        }
        if matches!(visao, VisaoFixture::Detalhada) {
            detalhada_cassettes::guardar(
                &mut db,
                CENARIO,
                &ControleDetalhado {
                    instalacao: instalacao.clone(),
                    unidade: UnidadeExibida::nova("TESTE").unwrap(),
                    processos: TabelaObservada::nova(
                        Some(150),
                        FaixaObservada::nova(101, 101),
                        vec![LinhaDetalhada {
                            id: "901".to_owned(),
                            numero: "000901/2026".to_owned(),
                            visualizado: Some(true),
                            sigiloso: Some(false),
                        }],
                    )
                    .unwrap(),
                },
            )
            .unwrap();
        }
        let mut tcp = aceitar(&servidor);
        atender_http(&mut tcp, Some((&db, &instalacao, CENARIO))).unwrap();
    });

    // O cliente normal usa CONNECT + certificado válido para sua CA de teste.
    // O proxy faz somente uma leitura do anti-IES, nunca consulta SEI.
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let tunel = thread::spawn(move || {
        let mut tcp = aceitar(&proxy);
        assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
        tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
            .unwrap();
        let tls = TlsAcceptor::new(identidade).unwrap();
        let mut canal = tls.accept(tcp).expect("handshake TLS de teste");
        let pedido = cabecalho(&mut canal);
        assert!(pedido.starts_with(b"GET /sei/controlador.php?acao=procedimento_controlar&"));
        let mut alvo = TcpStream::connect(endereco).unwrap();
        alvo.write_all(&pedido).unwrap();
        alvo.shutdown(Shutdown::Write).unwrap();
        let mut resposta = Vec::new();
        alvo.read_to_end(&mut resposta).unwrap();
        canal.write_all(&resposta).unwrap();
        canal.flush().unwrap();
    });

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ies"));
    cmd.args(["--json", "unidade", "atual"]);
    if let Some(unidade) = esperada {
        cmd.args(["--unidade", unidade]);
    }
    let saida = cmd
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
    tunel.join().expect("único GET HTTPS");
    simulador.join().expect("único GET HTTP");
    saida
}

fn json(saida: &Output) -> Value {
    serde_json::from_slice(&saida.stdout).unwrap_or_else(|e| {
        panic!(
            "JSON inválido {e}, exit {:?}, stderr: {}",
            saida.status.code(),
            String::from_utf8_lossy(&saida.stderr)
        )
    })
}

// BDDs definidos previamente na #355, verificados contra a CLI real.
#[test]
fn aies_01_cli_real_ler_unidade_observada() {
    let saida = executar(true, None);
    assert_eq!(saida.status.code(), Some(0));
    let valor = json(&saida);
    assert_eq!(valor["unidade_exibida"], "TESTE");
    assert_eq!(valor["origem"], "controle_de_processos");
}

#[test]
fn aies_03_cli_real_recusa_controle_ausente() {
    let saida = executar(false, None);
    falha(&saida, "Status");
}

#[test]
fn aies_04_cli_real_recusa_unidade_divergente() {
    let saida = executar(true, Some("OUTRA"));
    falha(&saida, "UnidadeExibidaDiferenteDaSolicitada");
}

// AIES-02: a sessão já lembrava uma página posterior da visão D.
#[test]
fn aies_02_cli_real_unidade_na_detalhada_pagina_posterior() {
    let saida = executar_visao(VisaoFixture::Detalhada, None);
    assert_eq!(saida.status.code(), Some(0));
    let valor = json(&saida);
    assert_eq!(valor["unidade_exibida"], "TESTE");
    assert_eq!(valor["origem"], "controle_de_processos");
}

#[path = "aies_unidade_cli/colunas.rs"]
mod colunas;
