//! #88: BDD-88-01/02 contra o anti-IES no binário real, via TLS local.
//! Hipótese sintética; não valida equivalência ao SEI de treinamento.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::acesso_sessao;
use iesei::acesso_versao;
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
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-88-cli";
const DOC: &str = "0011519";

struct Pasta(PathBuf);
impl Pasta {
    fn nova() -> Self {
        let instante = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let caminho = std::env::temp_dir().join(format!(
            "aies-documento-{}-{}",
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
                    "pedido não chegou"
                );
                thread::sleep(Duration::from_millis(25));
            }
            Err(e) => panic!("accept: {e}"),
        }
    }
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

fn encaminhar(canal: &mut impl Read, destino: &mut impl Write) {
    let cab = cabecalho(canal);
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
}

fn estado(com_documento: bool, versao_restrita: bool) -> (Connection, Instalacao) {
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
    if com_documento {
        documento_versoes::guardar(
            &mut banco,
            CENARIO,
            &DocumentoSimulado {
                instalacao: instalacao.clone(),
                id_documento: "42".into(),
                id_procedimento: "7".into(),
                numero_processo: "00001.000001/2026-01".into(),
                numero: DOC.into(),
                tipo: "Despacho".into(),
                revisoes: vec![
                    Revisao {
                        versao: NonZeroU32::new(1).unwrap(),
                        usuario: "Ana".into(),
                        unidade: "TESTE".into(),
                        modificacao: "01/11/2026".into(),
                        html_corpo: "<p>Primeira redação</p>".into(),
                    },
                    Revisao {
                        versao: NonZeroU32::new(3).unwrap(),
                        usuario: "Bia".into(),
                        unidade: "TESTE".into(),
                        modificacao: "03/11/2026".into(),
                        html_corpo: "<p>Terceira redação</p>".into(),
                    },
                ],
            },
        )
        .unwrap();
    }
    if versao_restrita {
        acesso_versao::restringir(
            &banco,
            &instalacao,
            CENARIO,
            "42",
            NonZeroU32::new(1).unwrap(),
        )
        .unwrap();
    }
    (banco, instalacao)
}

fn executar(pasta: &Pasta, args: &[&str], requisicoes: usize) -> Output {
    executar_com_estado(pasta, args, requisicoes, true)
}

fn executar_com_estado(
    pasta: &Pasta,
    args: &[&str],
    requisicoes: usize,
    com_documento: bool,
) -> Output {
    executar_com_restricao(pasta, args, requisicoes, com_documento, false)
}

fn executar_com_restricao(
    pasta: &Pasta,
    args: &[&str],
    requisicoes: usize,
    com_documento: bool,
    versao_restrita: bool,
) -> Output {
    executar_com_sessao(pasta, args, requisicoes, com_documento, versao_restrita, false)
}

fn executar_com_sessao(
    pasta: &Pasta,
    args: &[&str],
    requisicoes: usize,
    com_documento: bool,
    versao_restrita: bool,
    sessao_bloqueada: bool,
) -> Output {
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
    let simulador = thread::spawn(move || {
        let (db, instalacao) = estado(com_documento, versao_restrita);
        if sessao_bloqueada {
            acesso_sessao::bloquear(&db, &instalacao, CENARIO).unwrap();
        }
        for _ in 0..requisicoes {
            let mut tcp = aceitar(&servidor);
            atender_http(&mut tcp, Some((&db, &instalacao, CENARIO))).unwrap();
        }
    });
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade).unwrap();
        for _ in 0..requisicoes {
            let mut tcp = aceitar(&proxy);
            assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).expect("TLS de teste");
            let mut destino = TcpStream::connect(endereco).unwrap();
            encaminhar(&mut canal, &mut destino);
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
    tunel.join().expect("proxy sem requisições restantes");
    simulador
        .join()
        .expect("anti-IES sem requisições restantes");
    saida
}
fn json(saida: &Output) -> Value {
    serde_json::from_slice(&saida.stdout).unwrap_or_else(|erro| {
        panic!(
            "JSON inválido: {erro}; exit={:?}; stderr={}",
            saida.status.code(),
            String::from_utf8_lossy(&saida.stderr)
        )
    })
}

#[test]
fn bdd_88_01_02_cli_binario_versao_historica_e_foto_idempotente() {
    let pasta = Pasta::nova();
    let destino = pasta.0.join("saida");
    fs::create_dir(&destino).unwrap();
    let dir = destino.to_str().unwrap();
    let args = [
        "--json",
        "documento",
        DOC,
        "--versao",
        "1",
        "--saida",
        dir,
        "--sem-cache",
    ];
    // Pesquisa, tela, árvore, histórico e versão: seis pedidos.
    let primeira = executar(&pasta, &args, 6);
    assert_eq!(
        primeira.status.code(),
        Some(0),
        "CLI falhou: {}",
        String::from_utf8_lossy(&primeira.stderr)
    );
    let v = json(&primeira);
    assert_eq!(v["documentos"][0]["numero"], DOC);
    assert_eq!(v["documentos"][0]["versao"], 1);
    assert_eq!(v["documentos"][0]["versao_atual"], 3);
    assert_eq!(v["documentos"][0]["arquivo"]["nome"], "0011519.v1.html");
    assert_eq!(v["documentos"][0]["arquivo"]["gravado"], true);
    assert_eq!(v["requisicoes"], 6);
    let foto = destino.join("0011519.v1.html");
    let original = fs::read_to_string(&foto).unwrap();
    assert!(original.contains("Primeira redação"));
    assert!(!original.contains("Terceira redação"));

    let segunda = executar(&pasta, &args, 6);
    assert_eq!(
        segunda.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&segunda.stderr)
    );
    assert_eq!(json(&segunda)["documentos"][0]["arquivo"]["gravado"], false);
    assert_eq!(fs::read_to_string(&foto).unwrap(), original);

    // A foto editada falha na checagem local, sem abrir conexão.
    fs::write(&foto, "<html><body>Edição local</body></html>").unwrap();
    let terceira = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .output()
        .unwrap();
    assert_eq!(terceira.status.code(), Some(1));
    assert_eq!(json(&terceira)["subcodigo"], "foto_local_divergente");
    assert_eq!(
        fs::read_to_string(&foto).unwrap(),
        "<html><body>Edição local</body></html>"
    );
}

#[test]
fn bdd_88_03_cli_nao_inventa_versao_ausente_do_historico() {
    let pasta = Pasta::nova();
    // O documento tem apenas versões 1 e 3. A CLI deve recusar a
    // versão 2 como não observada, sem declarar que nunca existiu.
    // Controle, pesquisa, tela, árvore e histórico: cinco pedidos.
    let saida = executar(
        &pasta,
        &["--json", "documento", DOC, "--versao", "2", "--sem-cache"],
        5,
    );
    assert_eq!(saida.status.code(), Some(1));
    let valor = json(&saida);
    assert_eq!(valor["documento"], DOC);
    assert_eq!(valor["subcodigo"], "versao_nao_observada");
    assert!(!pasta.0.join("0011519.v2.html").exists());
}

/// BDD-88-03: uma sessão funcional não concede acesso a um DOC ausente
/// do estado compartilhado; a recusa vem da pesquisa HTTP do simulador.
#[test]
fn bdd_88_03_cli_documento_nao_acessivel_na_sessao() {
    let pasta = Pasta::nova();
    let destino = pasta.0.join("saida");
    fs::create_dir(&destino).unwrap();
    let args = [
        "--json",
        "documento",
        DOC,
        "--versao",
        "1",
        "--saida",
        destino.to_str().unwrap(),
        "--sem-cache",
    ];
    // Controle conhecido, busca sem snapshot de DOC: um GET e um POST
    // local de leitura. O simulador recusa com 503, sem inventar ausência.
    let saida = executar_com_estado(&pasta, &args, 2, false);
    assert_ne!(
        saida.status.code(),
        Some(0),
        "documento ausente virou sucesso"
    );
    let valor = json(&saida);
    assert!(valor["subcodigo"].is_string(), "recusa sem causa: {valor}");
    assert!(
        valor["documentos"].is_null(),
        "publicou documento não observado: {valor}"
    );
    assert!(!destino.join("0011519.v1.html").exists());
}

#[test]
fn bdd_88_03_cli_recusa_versao_sem_acesso_na_sessao() {
    let pasta = Pasta::nova();
    let destino = pasta.0.join("saida");
    fs::create_dir(&destino).unwrap();
    let args = [
        "--json",
        "documento",
        DOC,
        "--versao",
        "1",
        "--saida",
        destino.to_str().unwrap(),
        "--sem-cache",
    ];
    // Controle, pesquisa, tela, árvore, lista e versão restrita:
    // nenhum POST modificador e nenhuma foto da versão não acessível.
    let saida = executar_com_restricao(&pasta, &args, 6, true, true);
    assert_eq!(saida.status.code(), Some(1), "não recusou versão restrita");
    let valor = json(&saida);
    assert_eq!(valor["status_http"], 403, "recusa sem HTTP 403: {valor}");
    assert_eq!(valor["documento"], DOC);
    assert!(valor["documentos"].is_null());
    assert!(!destino.join("0011519.v1.html").exists());

    let humano = executar_com_restricao(
        &pasta,
        &["documento", DOC, "--versao", "1", "--sem-cache"],
        6,
        true,
        true,
    );
    assert_eq!(humano.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&humano.stderr).contains("HTTP 403"),
        "recusa humana sem HTTP 403"
    );
}

#[test]
fn bdd_88_03_cli_recusa_sessao_sem_acesso_antes_da_pesquisa() {
    let pasta = Pasta::nova();
    let destino = pasta.0.join("saida");
    fs::create_dir(&destino).unwrap();
    let args = [
        "--json",
        "documento",
        DOC,
        "--versao",
        "1",
        "--saida",
        destino.to_str().unwrap(),
        "--sem-cache",
    ];
    // O primeiro GET de Controle responde pela sessão inacessível;
    // não há pesquisa nem leitura do histórico, muito menos foto.
    let saida = executar_com_sessao(&pasta, &args, 1, true, false, true);
    assert_eq!(saida.status.code(), Some(1));
    let valor = json(&saida);
    assert_eq!(valor["status_http"], 403, "sessão não recusada: {valor}");
    assert!(valor["documentos"].is_null());
    assert!(!destino.join("0011519.v1.html").exists());
}
