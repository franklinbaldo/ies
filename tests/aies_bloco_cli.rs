//! BDD-256-01: CLI real, HTTPS local, anti-IES, mesmo SQLite efemero.
//! Numeros e estados sao hipoteses sinteticas, nao dados da cassette `UnB`.
#![cfg(unix)]

use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::blocos::{self, Linha as LinhaBloco, Pagina, Tipo};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
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

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-256-01-blocos";

struct Pasta(PathBuf);
impl Pasta {
    fn nova() -> Self {
        let agora = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let pasta = std::env::temp_dir().join(format!(
            "aies-retornos-{}-{}",
            std::process::id(),
            agora.as_nanos()
        ));
        fs::create_dir(&pasta).unwrap();
        fs::set_permissions(&pasta, fs::Permissions::from_mode(0o700)).unwrap();
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
    let status = Command::new("openssl")
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
    assert!(status.success(), "certificado local");
    let status = Command::new("openssl")
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
    assert!(status.success(), "identidade TLS local");
    (
        cert,
        Identity::from_pkcs12(&fs::read(p12).unwrap(), "teste").unwrap(),
    )
}
fn aceitar(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let inicio = Instant::now();
    loop {
        match listener.accept() {
            Ok((tcp, endereco)) => {
                assert!(endereco.ip().is_loopback());
                tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                tcp.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
                return tcp;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    inicio.elapsed() < Duration::from_secs(10),
                    "GET esperado ausente"
                );
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => panic!("accept: {e}"),
        }
    }
}
fn cabecalho(fluxo: &mut impl Read) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut byte = [0];
    while !buf.ends_with(b"\r\n\r\n") {
        assert!(buf.len() < 8192);
        fluxo.read_exact(&mut byte).unwrap();
        buf.push(byte[0]);
    }
    buf
}

fn registrar_blocos(banco: &mut Connection, instalacao: &Instalacao, interno: bool) {
    blocos::guardar(
        banco,
        instalacao,
        CENARIO,
        &Pagina {
            tipo: Tipo::Assinatura,
            total: 4,
            faixa: Some((1, 2)),
            linhas: vec![
                LinhaBloco {
                    numero: "2195".into(),
                    estado: "Gerado".into(),
                    descricao: "Despachos de teste".into(),
                },
                LinhaBloco {
                    numero: "2196".into(),
                    estado: "Disponibilizado".into(),
                    descricao: "Assinaturas pendentes".into(),
                },
            ],
        },
    )
    .unwrap();
    if interno {
        blocos::guardar_na_pagina(banco, instalacao, CENARIO, 1, &Pagina {
            tipo: Tipo::Interno,
            total: 4,
            faixa: Some((1, 2)),
            linhas: vec![
                LinhaBloco { numero: "3001".into(), estado: "Gerado".into(),
                    descricao: "Primeiro interno".into() },
                LinhaBloco { numero: "3002".into(), estado: "Gerado".into(),
                    descricao: "Segundo interno".into() },
            ],
        }).unwrap();
        blocos::guardar_na_pagina(banco, instalacao, CENARIO, 2, &Pagina {
            tipo: Tipo::Interno,
            total: 4,
            faixa: Some((3, 4)),
            linhas: vec![
                LinhaBloco { numero: "3003".into(), estado: "Disponibilizado".into(),
                    descricao: "Terceiro interno".into() },
                LinhaBloco { numero: "3004".into(), estado: "Gerado".into(),
                    descricao: "Quarto interno".into() },
            ],
        }).unwrap();
    }
}

fn executar(interno: bool) -> (Output, Vec<String>) {
    let esperados = if interno { 3usize } else { 2usize };
    let pasta = Pasta::nova();
    let (cert, identidade) = certificado(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    let nome_hash = ["infra", "hash"].join("_");
    let controle = pasta.privado(
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
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
        banco_cassettes::guardar(
            &mut banco,
            CENARIO,
            &ControleResumido {
                instalacao: instalacao.clone(),
                unidade: UnidadeExibida::nova("TESTE").unwrap(),
                recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
                gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            },
        )
        .unwrap();
        registrar_blocos(&mut banco, &instalacao, interno);
        for _ in 0..esperados {
            let mut fluxo = aceitar(&servidor);
            atender_http(&mut fluxo, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
    });

    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade).unwrap();
        let mut alvos = Vec::new();
        for _ in 0..esperados {
            let mut tcp = aceitar(&proxy);
            assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).expect("handshake TLS de teste");
            let mut pedido = cabecalho(&mut canal);
            let cabecalhos = String::from_utf8_lossy(&pedido);
            assert!(cabecalhos.starts_with("GET /sei/controlador.php?")
                || cabecalhos.starts_with("POST /sei/controlador.php?"));
            alvos.push(cabecalhos.lines().next().unwrap().to_owned());
            if alvos.last().is_some_and(|linha| linha.starts_with("POST ")) {
                let tamanho = cabecalhos.lines().find_map(|linha| {
                    linha.split_once(':').and_then(|(nome, v)|
                        nome.eq_ignore_ascii_case("content-length")
                            .then(|| v.trim().parse::<usize>().ok()).flatten())
                }).expect("Content-Length do POST");
                assert!(tamanho < 16_384);
                let mut corpo = vec![0u8; tamanho];
                canal.read_exact(&mut corpo).unwrap();
                pedido.extend_from_slice(&corpo);
            }
            let mut destino = TcpStream::connect(endereco).unwrap();
            destino.write_all(&pedido).unwrap();
            destino.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            destino.read_to_end(&mut resposta).unwrap();
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
        alvos
    });

    let argumentos: Vec<&str> = if interno {
        vec!["--json", "bloco", "listar", "--tipo", "interno", "--pagina", "2"]
    } else {
        vec!["--json", "bloco", "listar", "--tipo", "assinatura"]
    };
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(argumentos)
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
        .env_remove("IES_CREDENCIAIS_FILE")
        .output()
        .expect("binário ies");
    let pedidos = tunel.join().expect("GET HTTPS");
    simulador.join().expect("GET anti-IES");
    assert_eq!(pedidos.len(), esperados);
    assert_eq!(pedidos.iter().filter(|p| p.starts_with("GET ")).count(), 2);
    assert_eq!(pedidos.iter().filter(|p| p.starts_with("POST ")).count(), usize::from(interno));
    assert!(pedidos[0].contains("acao=procedimento_controlar"));
    assert!(pedidos[1].contains(if interno {"acao=bloco_interno_listar"} else {"acao=bloco_assinatura_listar"}));
    if interno { assert!(pedidos[2].contains("acao=bloco_interno_listar")); }
    (saida, pedidos)
}
fn json(saida: &Output) -> Value {
    serde_json::from_slice(&saida.stdout).unwrap_or_else(|e| {
        panic!(
            "JSON invalido {e}, exit {:?}, stderr: {}",
            saida.status.code(),
            String::from_utf8_lossy(&saida.stderr)
        )
    })
}

// Expectativas sobre o contrato da CLI, nao geradas a partir do HTML do servidor.
#[test]
fn bdd_256_01_cli_real_lista_somente_pagina_observada_no_sqlite() {
    let (saida, pedidos) = executar(false);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    assert_eq!(pedidos.len(), 2);
    let valor = json(&saida);
    assert_eq!(valor["unidade"], "TESTE");
    assert_eq!(valor["origem"], "web");
    assert_eq!(valor["requisicoes"], 2);
    assert_eq!(valor["lista"]["tipo"], "assinatura");
    assert_eq!(valor["lista"]["total_declarado"], 4);
    assert_eq!(valor["lista"]["faixa_observada"]["inicio"], 1);
    assert_eq!(valor["lista"]["faixa_observada"]["fim"], 2);
    let itens = valor["lista"]["itens"].as_array().unwrap();
    assert_eq!(itens.len(), 2);
    assert_eq!(itens[0]["numero"], "2195");
    assert_eq!(itens[0]["estado"], "Gerado");
    assert_eq!(itens[0]["descricao"], "Despachos de teste");
    assert!(itens[0]["quantidade_itens"].is_null());
    assert_eq!(itens[1]["numero"], "2196");
    assert_eq!(itens[1]["estado"], "Disponibilizado");
    assert_eq!(itens[1]["descricao"], "Assinaturas pendentes");
    assert!(itens[1]["quantidade_itens"].is_null());
    assert!(!valor.to_string().contains("infra_hash"));
    assert!(!valor.to_string().contains("PHPSESSID"));
}

/// BDD-256-02: o POST devolve exclusivamente a segunda pagina observada.
/// A expectativa nao e derivada do HTML gerado pelo anti-IES.
#[test]
fn bdd_256_02_cli_real_pagina_2_interna_sqlite() {
    let (saida, pedidos) = executar(true);
    assert_eq!(saida.status.code(), Some(0),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr));
    assert_eq!(pedidos.len(), 3);
    let valor = json(&saida);
    assert_eq!(valor["unidade"], "TESTE");
    assert_eq!(valor["requisicoes"], 3);
    assert_eq!(valor["lista"]["tipo"], "interno");
    assert_eq!(valor["lista"]["total_declarado"], 4);
    assert_eq!(valor["lista"]["faixa_observada"]["inicio"], 3);
    assert_eq!(valor["lista"]["faixa_observada"]["fim"], 4);
    let itens = valor["lista"]["itens"].as_array().unwrap();
    assert_eq!(itens.len(), 2);
    assert_eq!(itens[0]["numero"], "3003");
    assert_eq!(itens[1]["numero"], "3004");
    assert_eq!(itens[0]["estado"], "Disponibilizado");
    assert!(itens.iter().all(|x| x["quantidade_itens"].is_null()));
    assert!(!valor.to_string().contains("3001"));
    assert!(!valor.to_string().contains("3002"));
    assert!(!valor.to_string().contains("infra_hash"));
}
