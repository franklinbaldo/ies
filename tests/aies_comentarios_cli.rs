//! BDD-257-01: CLI Rust real -> HTTPS loopback -> anti-IES -> SQLite.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::comentarios::{self, Comentario, Observacao, Visao};
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
const CENARIO: &str = "bdd-257-comentarios";
const NUP: &str = "00001.000001/2026-01";

#[derive(Clone, Copy, Debug)]
enum Modo {
    Completa,
    Parcial,
    Desconhecida,
}

fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match escuta.accept() {
            Ok((cliente, peer)) => {
                assert!(peer.ip().is_loopback());
                cliente
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                cliente
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return Some(cliente);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    return None;
                }
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
    let metodo = texto.split_whitespace().next().unwrap().to_owned();
    let acao = texto
        .lines()
        .next()
        .unwrap()
        .split_once('?')
        .and_then(|(_, query)| {
            query
                .split('&')
                .find_map(|campo| campo.strip_prefix("acao="))
        })
        .unwrap_or("sem_acao")
        .to_owned();
    let tamanho = texto
        .lines()
        .find_map(|linha| {
            let (chave, valor) = linha.split_once(':')?;
            chave
                .eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 64_000);
    let mut corpo = vec![0; tamanho];
    fonte.read_exact(&mut corpo).unwrap();
    cabecalho.extend(corpo);
    (cabecalho, format!("{metodo} {acao}"))
}

fn estado(modo: Modo) -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(
                Some(1),
                FaixaObservada::nova(1, 1),
                vec![Linha {
                    id: "10".into(),
                    numero: NUP.into(),
                    visualizado: false,
                    sigiloso: false,
                }],
            )
            .unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
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
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();
    if !matches!(modo, Modo::Desconhecida) {
        comentarios::guardar(
            &mut banco,
            &origem,
            CENARIO,
            "10",
            &Observacao {
                visao: Visao::Processo,
                total_declarado: if matches!(modo, Modo::Parcial) { 2 } else { 1 },
                itens: vec![Comentario {
                    id: "42".into(),
                    data_hora: "10/11/2026 08:00".into(),
                    unidade: "TESTE".into(),
                    usuario: "operador".into(),
                    texto: "Comentário sintético & conferido".into(),
                    protocolo: None,
                    tipo: None,
                }],
            },
        )
        .unwrap();
    }
    (banco, origem)
}

fn executar(modo: Modo) {
    let pasta = Pasta::nova();
    let (cert, tls_id) = identidade(&pasta.0, "sei.test");
    let origem = pasta.privado("origem", ORIGEM);
    let controle = pasta.privado("url", &format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}", "a".repeat(64)
    ));
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let backend = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    backend.set_nonblocking(true).unwrap();
    let backend_addr = backend.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let proxy_port = proxy.local_addr().unwrap().port();
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = Arc::clone(&stop);
    let servidor = thread::spawn(move || {
        let (banco, instalacao) = estado(modo);
        while let Some(mut cliente) = aceitar(&backend, &stopped) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
    });
    let eventos = Arc::new(Mutex::new(Vec::<String>::new()));
    let vistos = Arc::clone(&eventos);
    let stopped = Arc::clone(&stop);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(tls_id).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &stopped) {
            let (connect, _) = pedido_http(&mut tcp);
            assert!(connect.starts_with(b"CONNECT "));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (req, evento) = pedido_http(&mut canal);
            let mut destino = TcpStream::connect(backend_addr).unwrap();
            destino.write_all(&req).unwrap();
            destino.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            destino.read_to_end(&mut resposta).unwrap();
            vistos.lock().unwrap().push(evento);
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let saida: Output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["--json", "processo", "comentarios", NUP])
        .env("IES_URL_CONTROLE_FILE", &controle)
        .env("IES_COOKIE_FILE", &cookie)
        .env("IES_ORIGEM_FILE", &origem)
        .env("SSL_CERT_FILE", &cert)
        .env("https_proxy", format!("http://127.0.0.1:{proxy_port}"))
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
    stop.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    servidor.join().unwrap();
    let eventos = eventos.lock().unwrap().clone();
    verificar(modo, &saida, &eventos);
}

fn verificar(modo: Modo, saida: &Output, eventos: &[String]) {
    let esperado = i32::from(!matches!(modo, Modo::Completa));
    assert_eq!(
        saida.status.code(),
        Some(esperado),
        "{modo:?}: stdout={} stderr={}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    if matches!(modo, Modo::Completa) {
        assert_eq!(json["nup"], NUP);
        assert_eq!(json["total"], 1);
        assert_eq!(json["lista_completa"], true);
        assert_eq!(json["comentarios"][0]["id"], "42");
        assert_eq!(json["comentarios"][0]["usuario"], "operador");
        assert_eq!(json["comentarios"][0]["unidade"], "TESTE");
        assert_eq!(json["comentarios"][0]["data_hora"], "10/11/2026 08:00");
        assert_eq!(
            json["comentarios"][0]["texto"],
            "Comentário sintético & conferido"
        );
    } else {
        assert!(
            json.get("comentarios").is_none(),
            "lista incompleta ou desconhecida não pode virar vazia ou completa"
        );
    }
    assert!(
        eventos.iter().all(|e| e.starts_with("GET ")),
        "consulta não pode enviar POST: {eventos:?}"
    );
    let listagens = eventos
        .iter()
        .filter(|e| e.as_str() == "GET comentario_listar")
        .count();
    assert_eq!(
        listagens,
        usize::from(!matches!(modo, Modo::Desconhecida)),
        "link só é consultado quando há observação explícita"
    );
}

#[test]
fn bdd_257_01_lista_completa_na_cli_real() {
    executar(Modo::Completa);
}
#[test]
fn bdd_257_01_pagina_incompleta_nao_e_lista_total() {
    executar(Modo::Parcial);
}
#[test]
fn bdd_257_01_ausencia_de_foto_nao_e_lista_vazia() {
    executar(Modo::Desconhecida);
}
