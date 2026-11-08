//! BDD-333-01/02: CLI real, HTTPS de loopback e estado SQLite do anti-IES.
//! A hipótese sintética não substitui validação de compatibilidade com SEI.
#![cfg(unix)]

use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::documentos_simulados::{self, Processo};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::processos::{self, Destino, ProcessoSimulado};
use native_tls::{Identity, TlsAcceptor};
use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-333-cli";
const NUP: &str = "00001.000001/2026-01";

struct Pasta(PathBuf);
impl Pasta {
    fn nova() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pasta = std::env::temp_dir().join(format!("aies-envio-{}-{nanos}", std::process::id()));
        fs::create_dir(&pasta).unwrap();
        Self(pasta)
    }

    fn privado(&self, nome: &str, conteudo: &str) -> PathBuf {
        let caminho = self.0.join(nome);
        fs::write(&caminho, conteudo).unwrap();
        fs::set_permissions(&caminho, fs::Permissions::from_mode(0o600)).unwrap();
        caminho
    }
}
impl Drop for Pasta {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn identidade(pasta: &Path) -> (PathBuf, Identity) {
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

fn aceitar(listener: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
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

fn cabecalho(fonte: &mut impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut byte = [0];
    while !bytes.ends_with(b"\r\n\r\n") {
        assert!(bytes.len() < 16_384);
        fonte.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
    }
    bytes
}

fn pedido(fonte: &mut impl Read) -> (Vec<u8>, String, String) {
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

fn preparar_banco() -> (Connection, Instalacao) {
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
            anotacao_origem: Some("Nota da origem".into()),
            destinos: vec![Destino {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Unidade de teste".into(),
            }],
        },
    )
    .unwrap();
    (banco, origem)
}

/// Trilha tem apenas método e ação; nunca URL completa, corpo ou cookie.
fn executar(
    comandos: &[&[&str]],
    falhar_releitura: bool,
) -> (Vec<Output>, Vec<String>, ProcessoSimulado) {
    let pasta = Pasta::nova();
    let (cert, identidade) = identidade(&pasta.0);
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
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let destino = escuta.local_addr().unwrap();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_sim = Arc::clone(&parar);
    let simulador = thread::spawn(move || {
        let (banco, instalacao) = preparar_banco();
        let antes = processos::ler(&banco, &instalacao, CENARIO, "10")
            .unwrap()
            .unwrap();
        assert_eq!(antes.aberto_em, ["ORIGEM"]);
        while let Some(mut tcp) = aceitar(&escuta, &parar_sim) {
            atender_http(&mut tcp, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        processos::ler(&banco, &instalacao, CENARIO, "10")
            .unwrap()
            .unwrap()
    });
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar_proxy = Arc::clone(&parar);
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let capturada = Arc::clone(&trilha);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade).unwrap();
        let mut enviado = false;
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, metodo, acao) = pedido(&mut canal);
            // A hipótese de incerteza só atinge a releitura posterior à escrita.
            let bloquear = falhar_releitura && enviado && metodo == "GET";
            let mut resposta = if bloquear {
                b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: text/plain\r\nContent-Length: 7\r\nConnection: close\r\n\r\nIndispo".to_vec()
            } else {
                let mut backend = TcpStream::connect(destino).unwrap();
                backend.write_all(&bytes).unwrap();
                backend.shutdown(Shutdown::Write).unwrap();
                let mut resposta = Vec::new();
                backend.read_to_end(&mut resposta).unwrap();
                resposta
            };
            if metodo == "POST" && acao == "procedimento_enviar" {
                // Só valores de controles sintéticos, nunca headers, URL ou cookie.
                let inicio = bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
                let formulario = std::str::from_utf8(&bytes[inicio..]).unwrap();
                let vistos: Vec<_> = formulario
                    .split('&')
                    .filter(|p| {
                        ["hdnIdProtocolos=", "hdnUnidades=", "chkSinManterAberto=",
                         "chkSinRemoverAnotacoes="].iter().any(|n| p.starts_with(n))
                    })
                    .collect();
                eprintln!("BDD-333 campos de escrita sintética: {vistos:?}");
                enviado = true;
            }
            capturada.lock().unwrap().push(format!("{metodo} {acao}"));
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
            resposta.clear();
        }
    });
    let saidas = comandos
        .iter()
        .map(|args| {
            Command::new(env!("CARGO_BIN_EXE_ies"))
                .args(*args)
                .env("IES_URL_CONTROLE_FILE", &controle)
                .env("IES_COOKIE_FILE", &cookie)
                .env("IES_ORIGEM_FILE", &origem)
                .env("SSL_CERT_FILE", &cert)
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
                .expect("binário da CLI")
        })
        .collect();
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let finalizado = simulador.join().unwrap();
    let pedidos = trilha.lock().unwrap().clone();
    (saidas, pedidos, finalizado)
}

fn args(confirmar: bool) -> Vec<&'static str> {
    let mut comando = vec![
        "--json",
        "processo",
        "enviar",
        NUP,
        "--para",
        "DESTINO",
        "--manter-aberto",
    ];
    if confirmar {
        comando.push("--confirmar");
    }
    comando
}

fn json(saida: &Output) -> Value {
    serde_json::from_slice(&saida.stdout).unwrap_or_else(|erro| {
        panic!(
            "JSON inválido {erro}; exit {:?}; stdout {}; stderr {}",
            saida.status.code(),
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        )
    })
}

#[test]
fn bdd_333_01_02_previa_nao_muta_e_confirmacao_unica_atualiza_sqlite() {
    let previa = args(false);
    let confirmacao = args(true);
    let (saidas, pedidos, depois) = executar(&[&previa, &confirmacao], false);
    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "prévia {} pedidos {pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout)
    );
    let p = json(&saidas[0]);
    assert_eq!(p["confirmado"], false);
    assert_eq!(p["alterado"], false);
    assert_eq!(p["aberto_em"], serde_json::json!(["ORIGEM"]));
    assert_eq!(
        saidas[1].status.code(),
        Some(0),
        "confirmação stdout {} stderr {} pedidos {pedidos:?}",
        String::from_utf8_lossy(&saidas[1].stdout),
        String::from_utf8_lossy(&saidas[1].stderr)
    );
    let c = json(&saidas[1]);
    assert_eq!(c["confirmado"], true);
    assert_eq!(c["alterado"], true);
    assert_eq!(c["depois"], serde_json::json!(["ORIGEM", "DESTINO"]));
    assert_eq!(depois.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        1
    );
    assert!(
        pedidos
            .iter()
            .rposition(|p| p == "GET procedimento_visualizar")
            > pedidos.iter().position(|p| p == "POST procedimento_enviar"),
        "a confirmação depende de uma leitura posterior ao POST: {pedidos:?}"
    );
}

#[test]
fn bdd_333_02_efeito_incerto_depois_do_post_nao_repete_escrita() {
    let confirmacao = args(true);
    let (saidas, pedidos, depois) = executar(&[&confirmacao], true);
    assert_eq!(
        saidas[0].status.code(),
        Some(3),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    assert_eq!(json(&saidas[0])["codigo"], "efeito_incerto");
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        1
    );
    assert_eq!(
        depois.aberto_em,
        ["ORIGEM", "DESTINO"],
        "a mutação ocorreu apesar de não haver prova independente"
    );
}
