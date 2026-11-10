//! BDD-270-02: CLI Rust real -> HTTPS loopback -> anti-IES -> SQLite.
//! Hipótese sintética: a cassette original ainda requer confronto independente.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use iesei::processos::{self, ProcessoSimulado};
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
const CENARIO: &str = "bdd-270-anexacao";
const PRINCIPAL: &str = "00001.000001/2026-01";
const ANEXADO: &str = "00001.000002/2026-01";

fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match escuta.accept() {
            Ok((cliente, peer)) => {
                assert!(peer.ip().is_loopback());
                cliente.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                cliente.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
                return Some(cliente);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) { return None; }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("aceitar: {e}"),
        }
    }
}

fn pedido_http(fonte: &mut impl Read) -> (Vec<u8>, String) {
    let mut cabecalho = Vec::new();
    let mut byte = [0u8];
    while !cabecalho.ends_with(b"\r\n\r\n") {
        assert!(cabecalho.len() < 16_384);
        fonte.read_exact(&mut byte).unwrap();
        cabecalho.push(byte[0]);
    }
    let texto = std::str::from_utf8(&cabecalho).unwrap();
    let primeira = texto.lines().next().unwrap();
    let metodo = primeira.split_whitespace().next().unwrap();
    let acao = primeira.split_once('?')
        .and_then(|(_, q)| q.split('&').find_map(|par| par.strip_prefix("acao=").or_else(|| par.strip_prefix("acao_ajax="))))
        .unwrap_or("sem_acao");
    let tamanho = texto.lines().find_map(|linha| {
        let (chave, valor) = linha.split_once(':')?;
        chave.eq_ignore_ascii_case("content-length").then(|| valor.trim().parse::<usize>().unwrap())
    }).unwrap_or(0);
    assert!(tamanho < 64_000);
    let mut corpo = vec![0; tamanho];
    fonte.read_exact(&mut corpo).unwrap();
    cabecalho.extend(corpo);
    (cabecalho, format!("{metodo} {acao}"))
}

fn estado() -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(&mut banco, CENARIO, &ControleResumido {
        instalacao: origem.clone(),
        unidade: UnidadeExibida::nova("TESTE").unwrap(),
        recebidos: TabelaObservada::<Linha>::nova(Some(2), FaixaObservada::nova(1, 2),
            vec![("10", PRINCIPAL), ("22", ANEXADO)].into_iter().map(|(id, numero)| Linha {
                id: id.into(), numero: numero.into(), visualizado: false, sigiloso: false
            }).collect()).unwrap(),
        gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
    }).unwrap();
    for (id, numero) in [("10", PRINCIPAL), ("22", ANEXADO)] {
        processos::guardar(&mut banco, &origem, CENARIO, &ProcessoSimulado {
            id: id.into(), numero: numero.into(), unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()], anotacao_origem: None, destinos: vec![]
        }).unwrap();
    }
    (banco, origem)
}

fn cli(pasta: &Pasta, arquivos: [&std::path::Path; 4], proxy: u16, args: &[&str]) -> Output {
    let [controle, cookie, origem, certificado] = arquivos;
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env("IES_URL_CONTROLE_FILE", controle)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("SSL_CERT_FILE", certificado)
        .env("https_proxy", format!("http://127.0.0.1:{proxy}"))
        .env("HOME", &pasta.0)
        .env_remove("HTTPS_PROXY").env_remove("all_proxy").env_remove("ALL_PROXY")
        .env_remove("NO_PROXY").env_remove("no_proxy")
        .env_remove("IES_CONFIG_FILE").env_remove("IES_CREDENCIAIS_FILE")
        .output().unwrap()
}

#[derive(Clone, Copy)]
enum Modo { Aceite, RespostaIncerta }

fn executar(modo: Modo) {
    let pasta = Pasta::nova();
    let (cert, tls_id) = identidade(&pasta.0, "sei.test");
    let origem = pasta.privado("origem", ORIGEM);
    let controle = pasta.privado("url", &format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}", "a".repeat(64)
    ));
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let arquivos = [controle.as_path(), cookie.as_path(), origem.as_path(), cert.as_path()];
    let backend = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    backend.set_nonblocking(true).unwrap();
    let endereco = backend.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let sinal = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        let (banco, instalacao) = estado();
        while let Some(mut cliente) = aceitar(&backend, &sinal) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        banco.query_row("SELECT COUNT(*) FROM sim_anexacao", [], |r| r.get::<_, i64>(0)).unwrap()
    });
    let eventos = Arc::new(Mutex::new(Vec::<String>::new()));
    let pedidos = Arc::clone(&eventos);
    let sinal = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(tls_id).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &sinal) {
            let (connect, _) = pedido_http(&mut tcp);
            assert!(connect.starts_with(b"CONNECT"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (req, evento) = pedido_http(&mut canal);
            let mut destino = TcpStream::connect(endereco).unwrap();
            destino.write_all(&req).unwrap();
            destino.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            destino.read_to_end(&mut resposta).unwrap();
            if matches!(modo, Modo::RespostaIncerta) && evento == "POST procedimento_anexar" {
                // SQLite já recebeu a escrita, mas a resposta não prova aceite.
                resposta = b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec();
            }
            pedidos.lock().unwrap().push(evento);
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let saida = cli(&pasta, arquivos, porta, &["--json", "processo", "anexar", ANEXADO, "--a", PRINCIPAL, "--confirmar"]);
    let posterior = cli(&pasta, arquivos, porta, &[
        "--json", "processo", PRINCIPAL, "--sem-andamentos", "--sem-documentos",
        "--sem-propriedades", "--sem-marcadores"
    ]);
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let total = servidor.join().unwrap();
    let pedidos = eventos.lock().unwrap().clone();
    let esperado = if matches!(modo, Modo::Aceite) { 0 } else { 3 };
    assert_eq!(saida.status.code(), Some(esperado), "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout), String::from_utf8_lossy(&saida.stderr));
    let saida_json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    if matches!(modo, Modo::Aceite) {
        assert_eq!(saida_json["confirmado"], true);
        assert_eq!(saida_json["alterado"], true);
    } else {
        assert_eq!(saida_json["codigo"], "efeito_incerto");
        assert!(saida_json["erro"].as_str().unwrap().contains("não repita"));
    }
    assert_eq!(posterior.status.code(), Some(0), "leitura={}", String::from_utf8_lossy(&posterior.stdout));
    let leitura: serde_json::Value = serde_json::from_slice(&posterior.stdout).unwrap();
    assert_eq!(leitura["numero"], PRINCIPAL);
    assert_eq!(leitura["anexados"][0]["numero"], ANEXADO);
    assert_eq!(leitura["anexados"][0]["id_procedimento"], "22");
    assert_eq!(total, 1);
    assert_eq!(pedidos.iter().filter(|e| *e == "POST procedimento_anexar").count(), 1,
        "uma única escrita, sem retry; {pedidos:?}");
}

#[test]
fn bdd_270_02_confirmacao_e_releitura_independente() { executar(Modo::Aceite); }

#[test]
fn bdd_270_02_efeito_incerto_sem_retry_de_anexacao() { executar(Modo::RespostaIncerta); }
