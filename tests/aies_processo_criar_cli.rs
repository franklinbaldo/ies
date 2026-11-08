//! BDD-245-01: a CLI real consulta o anti-IES sem criar processo.
//! Bancada HTTPS exclusiva de loopback com SQLite persistente durante o cenário.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::criacao_processo::{self, TipoDeProcesso};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use native_tls::TlsAcceptor;
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-245-cli";
const TIPO: &str = "Administração Geral: Ensaio";

fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match escuta.accept() {
            Ok((conexao, peer)) => {
                assert!(peer.ip().is_loopback());
                conexao
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                conexao
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return Some(conexao);
            }
            Err(erro) if erro.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    return None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(erro) => panic!("aceitar: {erro}"),
        }
    }
}

fn cabecalho(fonte: &mut impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut byte = [0u8; 1];
    while !bytes.ends_with(b"\r\n\r\n") {
        assert!(bytes.len() < 16_384);
        fonte.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
    }
    bytes
}

fn pedido(canal: &mut impl Read) -> (Vec<u8>, String) {
    let mut bytes = cabecalho(canal);
    let texto = std::str::from_utf8(&bytes).unwrap();
    let primeira = texto.lines().next().unwrap().to_owned();
    let tamanho = texto
        .lines()
        .find_map(|linha| {
            let (nome, valor) = linha.split_once(':')?;
            nome.eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 64_000);
    let mut corpo = vec![0; tamanho];
    canal.read_exact(&mut corpo).unwrap();
    bytes.extend(corpo);
    (bytes, primeira)
}

fn banco() -> (Connection, Instalacao) {
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
    criacao_processo::registrar(
        &banco,
        &instalacao,
        CENARIO,
        &TipoDeProcesso {
            id: "12".into(),
            nome: TIPO.into(),
            id_assunto: "21".into(),
            assunto: "Assunto de Ensaio".into(),
        },
    )
    .unwrap();
    (banco, instalacao)
}

fn quantidade(banco: &Connection) -> i64 {
    banco
        .query_row("SELECT COUNT(*) FROM sim_processo", [], |linha| {
            linha.get(0)
        })
        .unwrap()
}

fn executar(args: &[&str]) -> (Output, Vec<String>, (i64, i64), usize) {
    let pasta = Pasta::nova();
    let (cert, identidade) = identidade(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    let chave = ["infra", "hash"].join("_");
    let controle = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&{chave}={}",
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
        let (banco, instalacao) = banco();
        let antes = quantidade(&banco);
        while let Some(mut cliente) = aceitar(&escuta, &parar_sim) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        let tipos = criacao_processo::listar(&banco, &instalacao, CENARIO)
            .unwrap()
            .unwrap()
            .len();
        ((antes, quantidade(&banco)), tipos)
    });

    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar_proxy = Arc::clone(&parar);
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let registro = Arc::clone(&trilha);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, primeira) = pedido(&mut canal);
            let metodo = primeira.split_whitespace().next().unwrap();
            let alvo = primeira.split_whitespace().nth(1).unwrap();
            // Nunca armazenar a URL, pois pode conter token de sessão.
            let acao = alvo
                .split_once('?')
                .and_then(|(_, query)| {
                    query
                        .split('&')
                        .find_map(|parte| parte.strip_prefix("acao="))
                })
                .unwrap_or("desconhecida");
            let mut backend = TcpStream::connect(destino).unwrap();
            backend.write_all(&bytes).unwrap();
            backend.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            backend.read_to_end(&mut resposta).unwrap();
            let status = std::str::from_utf8(&resposta)
                .ok()
                .and_then(|valor| valor.split_whitespace().nth(1))
                .unwrap_or("?")
                .to_owned();
            registro
                .lock()
                .unwrap()
                .push(format!("{metodo} {acao} {status}"));
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });

    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
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
        .unwrap();
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let (contagens, tipos) = simulador.join().unwrap();
    let pedidos = trilha.lock().unwrap().clone();
    (saida, pedidos, contagens, tipos)
}

#[test]
fn bdd_245_01_previa_por_cli_real_sem_mutacao_sqlite() {
    let (saida, pedidos, contagens, tipos) = executar(&[
        "--json",
        "processo",
        "criar",
        "--tipo",
        TIPO,
        "--especificacao",
        "Ensaio sem gravação",
    ]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    let resultado: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(resultado["confirmado"], false);
    assert_eq!(resultado["tipo"], TIPO);
    assert_eq!(resultado["especificacao"], "Ensaio sem gravação");
    assert_eq!(resultado["assunto"], "Assunto de Ensaio");
    assert_eq!(resultado["nivel"], "publico");
    assert_eq!(resultado["unidade"], "TESTE");
    assert!(resultado["nup"].is_null());
    assert_eq!(resultado["requisicoes"], 3);
    assert_eq!(
        pedidos,
        vec![
            "GET procedimento_controlar 200",
            "GET procedimento_escolher_tipo 200",
            "GET procedimento_gerar 200",
        ]
    );
    assert_eq!(contagens, (0, 0), "prévia não pode criar processo");
    assert_eq!(tipos, 1, "tabela de escolha persiste sem ser alterada");
}
