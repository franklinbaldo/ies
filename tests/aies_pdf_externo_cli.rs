//! #332 BDD-332-03/04: binário real → HTTPS local → anti-IES → SQLite → releitura.
//! Hipótese sintética; nunca comunica com uma instalação SEI.
#![cfg(unix)]

use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::pdf_externo::{self, Processo, TipoExterno};
use iesei::processos::{self, ProcessoSimulado};
use native_tls::TlsAcceptor;
use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const ORIGEM: &str = "https://sei.test/sei";
const NUP: &str = "00001.000001/2026-01";
const CENARIO: &str = "bdd-332-binario";

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};

fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match escuta.accept() {
            Ok((conexao, peer)) => {
                assert!(peer.ip().is_loopback(), "ensaio somente em loopback");
                conexao.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                conexao.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
                return Some(conexao);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    return None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("accept: {e}"),
        }
    }
}

fn pedido(canal: &mut impl Read) -> (Vec<u8>, String, String) {
    let mut cabecalho = Vec::new();
    let mut byte = [0];
    while !cabecalho.ends_with(b"\r\n\r\n") {
        assert!(cabecalho.len() < 16_384);
        canal.read_exact(&mut byte).unwrap();
        cabecalho.push(byte[0]);
    }
    let texto = std::str::from_utf8(&cabecalho).unwrap();
    let inicio = texto.lines().next().unwrap().to_owned();
    let tamanho = texto.lines().find_map(|linha| {
        let (nome, valor) = linha.split_once(':')?;
        nome.eq_ignore_ascii_case("content-length")
            .then(|| valor.trim().parse::<usize>().unwrap())
    }).unwrap_or(0);
    assert!(tamanho < 64_000, "PDF sintético deve ser pequeno");
    let mut corpo = vec![0; tamanho];
    canal.read_exact(&mut corpo).unwrap();
    cabecalho.extend(corpo);
    let acao = inicio.split_whitespace().nth(1).unwrap()
        .split_once('?')
        .and_then(|(_, query)| query.split('&').find_map(|par| par.strip_prefix("acao=")))
        .unwrap_or("desconhecida").to_owned();
    let metodo = inicio.split_whitespace().next().unwrap().to_owned();
    (cabecalho, metodo, acao)
}

fn preparar_banco() -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(&mut banco, CENARIO, &ControleResumido {
        instalacao: origem.clone(),
        unidade: UnidadeExibida::nova("TESTE").unwrap(),
        recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
    }).unwrap();
    processos::guardar(&mut banco, &origem, CENARIO, &ProcessoSimulado {
        id: "10".into(), numero: NUP.into(), unidade_origem: "TESTE".into(),
        aberto_em: vec!["TESTE".into()], anotacao_origem: None, destinos: vec![],
    }).unwrap();
    pdf_externo::semear(&banco, &origem, CENARIO,
        &Processo { id: "10".into(), nup: NUP.into(), unidade: "TESTE".into() },
        &TipoExterno { id: "293".into(), nome: "Anexo".into(), limite_bytes: 10 * 1024 * 1024 },
    ).unwrap();
    (banco, origem)
}

fn executar(incerteza: bool) -> (Vec<Output>, Vec<String>, usize) {
    let pasta = Pasta::nova();
    let (cert, identidade_tls) = identidade(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    let chave = ["infra", "hash"].join("_");
    let controle = pasta.privado("url", &format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&{chave}={}", "a".repeat(64)
    ));
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let pdf = pasta.0.join("peticao.pdf");
    fs::write(&pdf, b"%PDF-1.4\n1 0 obj\n%%EOF\n").unwrap();

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let destino = escuta.local_addr().unwrap();
    let parar = Arc::new(AtomicBool::new(false));
    let sinal = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        let (banco, instalacao) = preparar_banco();
        while let Some(mut cliente) = aceitar(&escuta, &sinal) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        pdf_externo::documentos(&banco, &instalacao, CENARIO, "10").unwrap().len()
    });

    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let sinal_proxy = Arc::clone(&parar);
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let registro = Arc::clone(&trilha);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade_tls).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &sinal_proxy) {
            let (connect, _, _) = pedido(&mut tcp);
            assert!(connect.starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, metodo, acao) = pedido(&mut canal);
            let mut backend = TcpStream::connect(destino).unwrap();
            backend.write_all(&bytes).unwrap();
            backend.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            backend.read_to_end(&mut resposta).unwrap();
            let status = std::str::from_utf8(&resposta[..resposta.iter().position(|b| *b == b'\r').unwrap()])
                .unwrap().split_whitespace().nth(1).unwrap().to_owned();
            registro.lock().unwrap().push(format!("{metodo} {acao} {status}"));
            if incerteza && metodo == "POST" && acao == "documento_receber" && status == "302" {
                resposta = String::from_utf8(resposta).unwrap()
                    .replace("id_documento=8000000000001", "id_documento=8000000000999")
                    .into_bytes();
            }
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let base = ["--json", "documento", "incluir-externo", "--processo", NUP, "--arquivo"];
    let pdf_str = pdf.to_str().unwrap();
    let confirmar = [base.as_slice(), &[pdf_str, "--tipo", "Anexo", "--confirmar"]].concat();
    let consultar = ["--json", "processo", NUP, "--sem-andamentos", "--sem-propriedades", "--sem-marcadores"];
    let saidas = [&confirmar[..], &consultar[..]].iter().map(|args| {
        Command::new(env!("CARGO_BIN_EXE_ies"))
            .args(*args)
            .env("IES_ANTI_IES_TESTE", "1")
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
            .output().unwrap()
    }).collect();
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let total = servidor.join().unwrap();
    let eventos = trilha.lock().unwrap().clone();
    (saidas, eventos, total)
}

fn json(saida: &Output) -> Value {
    serde_json::from_slice(&saida.stdout).unwrap_or_else(|erro| panic!(
        "saída inválida {erro}: status={:?}, stderr={}", saida.status.code(),
        String::from_utf8_lossy(&saida.stderr)
    ))
}

#[test]
fn bdd_332_03_binario_confirma_upload_cadastro_e_releitura_independente() {
    let (saidas, eventos, total) = executar(false);
    assert_eq!(saidas[0].status.code(), Some(0),
        "stdout={} stderr={} eventos={eventos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr));
    let incluido = json(&saidas[0]);
    assert_eq!(incluido["confirmado"], true);
    assert_eq!(incluido["numero_sei"], "050000001");
    assert_eq!(incluido["reconciliado_por_leitura"], true);
    assert_eq!(saidas[1].status.code(), Some(0), "leitura externa: {:?}", saidas[1].stderr);
    assert!(json(&saidas[1])["documentos"].to_string().contains("050000001"));
    assert_eq!(total, 1, "upload temporário não duplica o documento");
    for acao in ["documento_upload_anexo", "documento_receber"] {
        assert_eq!(eventos.iter().filter(|e| *e == &format!("POST {acao} 200")
            || *e == &format!("POST {acao} 302")).count(),
            if acao == "documento_receber" { 2 } else { 1 },
            "recarga de tipo, upload e cadastro não podem causar retry: {eventos:?}");
    }
    assert!(eventos.iter().rposition(|e| e.starts_with("GET procedimento_visualizar"))
        > eventos.iter().position(|e| e == "POST documento_receber 302"));
}

#[test]
fn bdd_332_04_resposta_sem_autoria_retorna_tres_sem_repetir_post() {
    let (saidas, eventos, total) = executar(true);
    assert_eq!(saidas[0].status.code(), Some(3), "stdout={} stderr={} eventos={eventos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr));
    let incerto = json(&saidas[0]);
    assert_eq!(incerto["confirmado"], false);
    assert_eq!(incerto["efeito_incerto"], true);
    assert_eq!(incerto["retentavel"], false);
    assert_eq!(incerto["cadastro_tentado"], true);
    assert_eq!(total, 1, "efeito existiu mesmo com resposta adulterada");
    assert_eq!(eventos.iter().filter(|e| *e == "POST documento_upload_anexo 200").count(), 1);
    assert_eq!(eventos.iter().filter(|e| *e == "POST documento_receber 302").count(), 1);
}
