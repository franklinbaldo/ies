//! #355 AIES-05: binário real, TLS local, anti-IES e SQLite efêmero compartilhado.
//! Meses e números abaixo são hipóteses sintéticas; não são registros da `UnB`.
#![cfg(unix)]

use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::retornos_cassettes::{self, Mes, Retorno, Situacao};
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
const CENARIO: &str = "aies-05-retornos";

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

fn mes(instalacao: &Instalacao, numero: u8) -> Mes {
    let (fim, numeros) = match numero {
        10 => ("31", ["001/2026", "002/2026"]),
        11 => ("30", ["003/2026", "004/2026"]),
        _ => panic!("fixture restrita a dois meses"),
    };
    Mes {
        instalacao: instalacao.clone(),
        data_inicial: format!("01%2F{numero:02}%2F2026"),
        data_final: format!("{fim}%2F{numero:02}%2F2026"),
        itens: vec![
            Retorno {
                situacao: Situacao::Aguardando,
                numero: numeros[0].into(),
                unidade: "U-TESTE-A".into(),
                data_programada: format!("15/{numero:02}/2026"),
                retornado_em: None,
                prazo_restante: None,
            },
            Retorno {
                situacao: Situacao::Devolver,
                numero: numeros[1].into(),
                unidade: "U-TESTE-B".into(),
                data_programada: format!("20/{numero:02}/2026"),
                retornado_em: Some(format!("21/{numero:02}/2026")),
                prazo_restante: Some("2 dias".into()),
            },
        ],
    }
}

fn executar(mes_solicitado: &str, esperados: usize) -> (Output, Vec<String>) {
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
        for mes_numero in [10, 11] {
            retornos_cassettes::guardar(&mut banco, CENARIO, &mes(&instalacao, mes_numero))
                .unwrap();
        }
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
            let pedido = cabecalho(&mut canal);
            assert!(pedido.starts_with(b"GET /sei/controlador.php?"));
            let linha = String::from_utf8_lossy(&pedido);
            alvos.push(linha.lines().next().unwrap().to_owned());
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

    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["--json", "unidade", "retornos", "--mes", mes_solicitado])
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
    assert!(pedidos.iter().all(|p| p.starts_with("GET ")));
    assert!(pedidos[0].contains("acao=procedimento_controlar"));
    assert!(pedidos[1].contains("acao=retorno_programado_listar"));
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

// AIES-05 da issue #355. Os fatos esperados não são derivados do HTML gerado.
#[test]
fn aies_05_cli_mes_observado_preserva_os_dois_retornos() {
    for (mes, registros, ultimo) in [
        ("2026-10", ["001/2026", "002/2026"], 3),
        ("2026-11", ["003/2026", "004/2026"], 4),
    ] {
        let (saida, pedidos) = executar(mes, ultimo);
        assert_eq!(saida.status.code(), Some(0), "{mes}: {:?}", saida.status);
        let valor = json(&saida);
        assert_eq!(valor["mes"], mes);
        assert_eq!(valor["unidade"], "TESTE");
        assert_eq!(valor["total_declarado"], 2);
        assert_eq!(valor["requisicoes"], ultimo);
        assert_eq!(valor["retornos"][0]["situacao"], "aguardando");
        assert_eq!(valor["retornos"][0]["numero"], registros[0]);
        assert_eq!(valor["retornos"][0]["unidade"], "U-TESTE-A");
        assert_eq!(
            valor["retornos"][0]["data_programada"],
            format!("15/{}/2026", &mes[5..])
        );
        assert!(valor["retornos"][0]["data_retorno"].is_null());
        assert_eq!(valor["retornos"][1]["situacao"], "devolver");
        assert_eq!(valor["retornos"][1]["numero"], registros[1]);
        assert_eq!(valor["retornos"][1]["unidade"], "U-TESTE-B");
        assert_eq!(
            valor["retornos"][1]["data_retorno"],
            format!("21/{}/2026", &mes[5..])
        );
        assert_eq!(valor["retornos"][1]["prazo_restante"], "2 dias");
        assert!(pedidos.last().unwrap().contains("data_inicial=01%2F"));
    }
}
#[test]
fn aies_05_cli_mes_nao_observado_nao_vira_lista_vazia() {
    let (saida, pedidos) = executar("2026-09", 2);
    falha(&saida, "MesDosRetornosInacessivel");
    assert!(!pedidos.iter().any(|p| p.contains("data_inicial=")));
}
