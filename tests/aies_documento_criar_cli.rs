//! BDD-246-01/02/04: CLI real com anti-IES e TLS só em loopback.
#![cfg(unix)]

use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::documentos_simulados::{self as documentos, Processo, Tipo};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::processos::{self, ProcessoSimulado};
use native_tls::TlsAcceptor;
use rusqlite::Connection;
use serde_json::Value;
#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::falha_por;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const ORIGEM: &str = "https://sei.test/sei";
const NUP: &str = "00001.000001/2026-01";
const CENARIO: &str = "bdd-246-cli";

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};

/// Aceita até o fim da execução.
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
    let mut corpo = vec![0u8; tamanho];
    canal.read_exact(&mut corpo).unwrap();
    bytes.extend(corpo);
    (bytes, primeira)
}

/// Remove da resposta sintética a prova de autoria da gravação.
fn resposta_sem_prova_de_autoria(resposta: &[u8]) -> Vec<u8> {
    let fim = resposta
        .windows(4)
        .position(|parte| parte == b"\r\n\r\n")
        .unwrap();
    let texto = std::str::from_utf8(&resposta[..fim]).unwrap();
    assert!(texto.starts_with("HTTP/1.1 302"));
    let linhas = texto
        .lines()
        .map(|linha| {
            if linha.starts_with("Location: ") {
                "Location: controlador.php?acao=procedimento_trabalhar&id_protocolo=10"
            } else {
                linha
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n");
    let mut alterado = format!("{linhas}\r\n\r\n").into_bytes();
    alterado.extend_from_slice(&resposta[fim + 4..]);
    alterado
}

fn preparar_banco(duplicar_tipo: bool) -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
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
    documentos::registrar_processo(
        &banco,
        &origem,
        CENARIO,
        &Processo {
            id: "10".into(),
            nup: NUP.into(),
            unidade: "TESTE".into(),
        },
    )
    .unwrap();
    documentos::registrar_tipo(
        &banco,
        &origem,
        CENARIO,
        "10",
        &Tipo {
            id: "291".into(),
            nome: "Despacho".into(),
        },
    )
    .unwrap();
    if duplicar_tipo {
        documentos::registrar_tipo(
            &banco,
            &origem,
            CENARIO,
            "10",
            &Tipo {
                id: "292".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
    }
    documentos::criar(&banco, &origem, CENARIO, "10", "291", "Anterior").unwrap();
    (banco, origem)
}

fn nomes_dos_campos_do_post(metodo: &str, acao: &str, bytes: &[u8]) -> Vec<String> {
    if metodo != "POST" || acao != "documento_gerar" {
        return Vec::new();
    }
    let inicio = bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
    std::str::from_utf8(&bytes[inicio..])
        .unwrap_or("")
        .split('&')
        .filter_map(|campo| campo.split_once('=').map(|(nome, _)| nome.to_owned()))
        .collect()
}

fn executar(comandos: &[&[&str]], resposta_incerta: bool) -> (Vec<Output>, Vec<String>, usize) {
    executar_com_cena(comandos, resposta_incerta, false)
}

fn executar_com_cena(
    comandos: &[&[&str]],
    resposta_incerta: bool,
    duplicar_tipo: bool,
) -> (Vec<Output>, Vec<String>, usize) {
    let pasta = Pasta::nova();
    let (cert, identity) = identidade(&pasta.0, "sei.test");
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
        let (banco, instalacao) = preparar_banco(duplicar_tipo);
        while let Some(mut cliente) = aceitar(&escuta, &parar_sim) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        documentos::documentos(&banco, &instalacao, CENARIO, "10")
            .unwrap()
            .len()
    });
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar_proxy = Arc::clone(&parar);
    let requisicoes = Arc::new(Mutex::new(Vec::<String>::new()));
    let registro = Arc::clone(&requisicoes);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identity).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).expect("TLS exclusivamente de teste");
            let (bytes, primeira) = pedido(&mut canal);
            let metodo = primeira.split_whitespace().next().unwrap();
            let rota = primeira.split_whitespace().nth(1).unwrap();
            // Nunca registrar a URL: ela pode incluir credenciais de sessão.
            let acao = rota
                .split_once('?')
                .and_then(|(_, query)| query.split('&').find_map(|p| p.strip_prefix("acao=")))
                .unwrap_or("desconhecida");
            let mut backend = TcpStream::connect(destino).unwrap();
            backend.write_all(&bytes).unwrap();
            backend.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            backend.read_to_end(&mut resposta).unwrap();
            let status = resposta
                .split(|b| *b == b'\r')
                .next()
                .and_then(|linha| std::str::from_utf8(linha).ok())
                .and_then(|linha| linha.split_whitespace().nth(1))
                .unwrap_or("?");
            // Só nomes dos campos do POST criador, jamais valores.
            let nomes = nomes_dos_campos_do_post(metodo, acao, &bytes);
            registro
                .lock()
                .unwrap()
                .push(format!("{metodo} {acao}: {status} {nomes:?}"));
            if resposta_incerta && metodo == "POST" && acao == "documento_gerar" && status == "302"
            {
                resposta = resposta_sem_prova_de_autoria(&resposta);
            }
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });

    let resultados = comandos
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
                .expect("execução do binário real")
        })
        .collect();
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let quantidade = simulador.join().unwrap();
    let trilha = requisicoes.lock().unwrap().clone();
    (resultados, trilha, quantidade)
}

fn criar(confirmar: bool) -> Vec<&'static str> {
    let mut args = vec![
        "--json",
        "documento",
        "criar",
        "--processo",
        NUP,
        "--tipo",
        "Despacho",
        "--nivel",
        "publico",
    ];
    if confirmar {
        args.push("--confirmar");
    }
    args
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|erro| {
        panic!(
            "JSON inválido: {erro}; status {:?}; stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn bdd_246_01_cli_previa_sem_post_criador() {
    let args = criar(false);
    let (saidas, trilha, quantidade) = executar(&[&args], false);
    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "{:?}: {trilha:?}",
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    let resultado = json(&saidas[0]);
    assert_eq!(resultado["nup"], NUP);
    assert_eq!(resultado["tipo"], "Despacho");
    assert_eq!(resultado["nivel"], "publico");
    assert_eq!(resultado["confirmado"], false);
    assert_eq!(quantidade, 1);
    assert_eq!(
        trilha
            .iter()
            .filter(|p| p.starts_with("POST documento_gerar:"))
            .count(),
        0
    );
}

#[test]
fn bdd_246_02_cli_confirma_um_post_e_releitura() {
    let args = criar(true);
    let consultar = [
        "--json",
        "processo",
        NUP,
        "--sem-andamentos",
        "--sem-propriedades",
        "--sem-marcadores",
    ];
    let (saidas, trilha, quantidade) = executar(&[&args, &consultar], false);
    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "stdout={} stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    let resultado = json(&saidas[0]);
    assert_eq!(resultado["numero_sei"], "1000002");
    assert_eq!(resultado["confirmado"], true);
    assert_eq!(
        saidas[1].status.code(),
        Some(0),
        "leitura independente: {}",
        String::from_utf8_lossy(&saidas[1].stdout)
    );
    let processo = json(&saidas[1]);
    let arvore = processo["documentos"]
        .as_array()
        .expect("árvore da CLI publicada");
    assert_eq!(
        arvore.len(),
        2,
        "consulta independente preserva original e criado"
    );
    assert_eq!(arvore[0]["numero_sei"], "1000001");
    assert_eq!(arvore[1]["numero_sei"], "1000002");
    assert_eq!(quantidade, 2, "original preservado + uma única criação");
    assert_eq!(
        trilha
            .iter()
            .filter(|p| p.starts_with("POST documento_gerar:"))
            .count(),
        1
    );
    assert!(
        trilha
            .iter()
            .rposition(|p| p.starts_with("GET procedimento_visualizar:"))
            > trilha
                .iter()
                .position(|p| p.starts_with("POST documento_gerar:")),
        "releitura deve acontecer depois do único POST: {trilha:?}"
    );
}

#[test]
fn bdd_246_04_cli_nao_atribui_autoria_sem_location_correspondente() {
    let args = criar(true);
    let (saidas, trilha, quantidade) = executar(&[&args], true);
    falha_por(&saidas[0], "TentativaDeCriacao", "NaoConfirmado");
    assert_eq!(quantidade, 2, "escrita pode ter ocorrido apesar do erro");
    assert_eq!(
        trilha
            .iter()
            .filter(|p| p.starts_with("POST documento_gerar:"))
            .count(),
        1
    );
}

#[path = "aies_documento_criar_cli/cancelamento.rs"]
mod cancelamento;

#[path = "aies_documento_criar_cli/recusas.rs"]
mod recusas;
