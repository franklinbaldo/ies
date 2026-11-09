//! BDD-258-01: binário ies consulta relações em HTTPS loopback contra anti-IES/SQLite.
//! Evidência exclusivamente sintética; não afirma compatibilidade SEI real.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::processos::{self, ProcessoSimulado};
use iesei::relacionamentos;
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
const CENARIO: &str = "bdd-258-01-cli";
const NUP: &str = "00001.000001/2026-01";
const DESTINO: &str = "00001.000002/2026-01";

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

fn pedido(canal: &mut impl Read) -> (Vec<u8>, String) {
    let mut cabecalho = Vec::new();
    let mut byte = [0u8];
    while !cabecalho.ends_with(b"\r\n\r\n") {
        assert!(cabecalho.len() < 16_384);
        canal.read_exact(&mut byte).unwrap();
        cabecalho.push(byte[0]);
    }
    let texto = std::str::from_utf8(&cabecalho).unwrap();
    let primeira = texto.lines().next().unwrap();
    let metodo = primeira.split_whitespace().next().unwrap();
    let acao = primeira
        .split_once('?')
        .and_then(|(_, q)| {
            q.split('&').find_map(|p| {
                p.strip_prefix("acao=")
                    .or_else(|| p.strip_prefix("acao_ajax="))
            })
        })
        .unwrap_or("sem_acao");
    let evento = format!("{metodo} {acao}");
    let tamanho = texto
        .lines()
        .find_map(|linha| {
            let (chave, valor) = linha.split_once(':')?;
            chave
                .eq_ignore_ascii_case("Content-Length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 65_536);
    let mut corpo = vec![0; tamanho];
    canal.read_exact(&mut corpo).unwrap();
    cabecalho.extend(corpo);
    (cabecalho, evento)
}

fn banco(vinculo_inicial: bool) -> (Connection, Instalacao) {
    let mut db = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&db).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut db,
        CENARIO,
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    for (id, nup) in [("10", NUP), ("22", DESTINO)] {
        processos::guardar(
            &mut db,
            &origem,
            CENARIO,
            &ProcessoSimulado {
                id: id.into(),
                numero: nup.into(),
                unidade_origem: "TESTE".into(),
                aberto_em: vec!["TESTE".into()],
                anotacao_origem: None,
                destinos: vec![],
            },
        )
        .unwrap();
    }
    relacionamentos::observar(&db, &origem, CENARIO, "10").unwrap();
    if vinculo_inicial {
        relacionamentos::registrar(&db, &origem, CENARIO, "10", "22").unwrap();
    }
    (db, origem)
}

/// Um POST sintético em loopback, separado da execução da CLI real.
fn remover_localmente(endereco: std::net::SocketAddr) {
    let corpo = "hdnInfraItemId=22";
    let mut cliente = TcpStream::connect(endereco).unwrap();
    let pedido = format!(
        "POST /sei/controlador.php?acao=procedimento_excluir_relacionamento&id_procedimento=10 HTTP/1.1\r\nHost: sei.test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{corpo}",
        corpo.len()
    );
    cliente.write_all(pedido.as_bytes()).unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    assert!(
        resposta.starts_with("HTTP/1.1 302 Found"),
        "exclusão local deve responder 302: {resposta}"
    );
}

#[derive(Clone, Copy)]
enum Modo {
    Previa,
    ConfirmarAdicao,
    ConfirmarRemocao,
    AdicaoSemAceite,
}

fn adicionar_localmente(endereco: std::net::SocketAddr) {
    let corpo = format!(
        "hdnIdProtocolo=22&txtProtocolo={DESTINO}&txtIdentificacaoProtocolo=&sbmAdicionar=Adicionar"
    );
    let mut cliente = TcpStream::connect(endereco).unwrap();
    let pedido = format!(
        "POST /sei/controlador.php?acao=procedimento_relacionar&id_procedimento=10 HTTP/1.1\r\nHost: sei.test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{corpo}",
        corpo.len()
    );
    cliente.write_all(pedido.as_bytes()).unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    assert!(
        resposta.starts_with("HTTP/1.1 302 Found"),
        "POST sintético deve ter aceite: {resposta}"
    );
}

fn executar_com(
    vinculo_inicial: bool,
    post_local: bool,
    remover_local: bool,
    modo: Modo,
) -> (Output, Vec<String>, usize) {
    let pasta = Pasta::nova();
    let (cert, identidade_tls) = identidade(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    let hash_chave = ["infra", "hash"].join("_");
    let url = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&{hash_chave}={}",
            "a".repeat(64),
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_servidor = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        let (db, origem) = banco(vinculo_inicial);
        while let Some(mut canal) = aceitar(&escuta, &parar_servidor) {
            atender_http(&mut canal, Some((&db, &origem, CENARIO))).unwrap();
        }
        relacionamentos::listar(&db, &origem, CENARIO, "10")
            .unwrap()
            .unwrap()
            .len()
    });
    // A mutação é HTTP exclusivamente em loopback, antes de executar a CLI real.
    // A consulta seguinte deve ler o mesmo estado SQLite, não uma fixture fixa.
    if post_local {
        adicionar_localmente(endereco);
    }
    if remover_local {
        assert!(vinculo_inicial && !post_local);
        remover_localmente(endereco);
    }
    let eventos = Arc::new(Mutex::new(Vec::<String>::new()));
    let eventos_proxy = Arc::clone(&eventos);
    let parar_proxy = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade_tls).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            let (connect, _) = pedido(&mut tcp);
            assert!(connect.starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, evento) = pedido(&mut canal);
            let mut backend = TcpStream::connect(endereco).unwrap();
            backend.write_all(&bytes).unwrap();
            backend.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            backend.read_to_end(&mut resposta).unwrap();
            // O backend pode ter gravado; a confirmação sem aceite não é sucesso.
            if matches!(modo, Modo::AdicaoSemAceite) && evento == "POST procedimento_relacionar" {
                resposta = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 7\r\nConnection: close\r\n\r\n<html/>".to_vec();
            }
            eventos_proxy.lock().unwrap().push(evento);
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let mut argumentos = vec!["--json", "processo", "relacionar", NUP, "--com", DESTINO];
    if remover_local || matches!(modo, Modo::ConfirmarRemocao) {
        argumentos.push("--remover");
    }
    if !matches!(modo, Modo::Previa) {
        argumentos.push("--confirmar");
    }
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(argumentos)
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
        .unwrap();
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let total = servidor.join().unwrap();
    let trilha = eventos.lock().unwrap().clone();
    (saida, trilha, total)
}

fn executar(
    vinculo_inicial: bool,
    post_local: bool,
    remover_local: bool,
) -> (Output, Vec<String>, usize) {
    executar_com(vinculo_inicial, post_local, remover_local, Modo::Previa)
}

#[path = "aies_relacionar_cli/confirmacao.rs"]
mod confirmacao;

#[test]
fn bdd_258_01_cli_previa_nup_distinto_sem_criar_vinculo() {
    let (saida, eventos, relacionados) = executar(false, false, false);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} stdout={} http={eventos:?}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["nup"], NUP);
    assert_eq!(valor["destino"], DESTINO);
    assert_eq!(valor["id_destino"], "22");
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert_eq!(valor["sem_efeito"], false);
    assert_eq!(valor["requisicoes"].as_u64(), Some(eventos.len() as u64));
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "GET procedimento_relacionar")
            .count(),
        1
    );
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "POST protocolo_RI1023")
            .count(),
        1
    );
    assert!(
        eventos.iter().all(|e| e.starts_with("GET ")
            || e == "POST protocolo_RI1023"
            || e == "POST protocolo_pesquisa_rapida"),
        "a prévia jamais grava relação: {eventos:?}"
    );
    assert_eq!(
        relacionados, 0,
        "estado SQLite compartilhado permanece sem vínculo"
    );
}

#[test]
fn bdd_258_01_relacao_preexistente_nao_faz_consulta_ajax_nem_escrita() {
    let (saida, eventos, relacionados) = executar(true, false, false);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} stdout={} http={eventos:?}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["nup"], NUP);
    assert_eq!(valor["destino"], DESTINO);
    assert_eq!(valor["id_destino"], "22");
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert_eq!(valor["sem_efeito"], true);
    assert_eq!(valor["requisicoes"].as_u64(), Some(eventos.len() as u64));
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "GET procedimento_relacionar")
            .count(),
        1
    );
    assert!(
        !eventos
            .iter()
            .any(|e| e.as_str() == "POST protocolo_RI1023"),
        "vínculo publicado na tabela dispensa consulta complementar: {eventos:?}"
    );
    assert!(
        eventos
            .iter()
            .all(|e| e.starts_with("GET ") || e == "POST protocolo_pesquisa_rapida"),
        "a prévia não submete vínculo: {eventos:?}"
    );
    assert_eq!(
        relacionados, 1,
        "a prévia preserva a única relação existente"
    );
}

/// Apoio a BDD-258-02: um POST local anti-IES altera SQLite e a CLI real,
/// em outra conexão TLS, observa o vínculo sem executar outro POST mutador.
/// Não é a confirmação pela CLI, que permanece bloqueada antes da rede.
#[test]
fn bdd_258_02_parcial_post_local_e_previa_cli_enxergam_a_mesma_relacao() {
    let (saida, eventos, relacionados) = executar(false, true, false);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} stdout={} http={eventos:?}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["sem_efeito"], true);
    assert_eq!(valor["confirmado"], false);
    assert_eq!(relacionados, 1);
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "GET procedimento_relacionar")
            .count(),
        1
    );
    assert!(
        eventos
            .iter()
            .all(|e| e.starts_with("GET ") || e == "POST protocolo_pesquisa_rapida"),
        "a CLI lê o vínculo criado no simulador, mas não o recria: {eventos:?}"
    );
}

/// Apoio a BDD-258-03: após exclusão HTTP exclusivamente local no
/// simulador, a CLI real relê o SQLite e constata ausência bilateral.
/// Isto não é confirmação pela própria CLI, ainda bloqueada.
#[test]
fn bdd_258_03_parcial_exclusao_local_e_previa_cli_observam_ausencia() {
    let (saida, eventos, relacionados) = executar(true, false, true);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} stdout={} http={eventos:?}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["remover"], true);
    assert_eq!(valor["sem_efeito"], true);
    assert_eq!(valor["confirmado"], false);
    assert_eq!(relacionados, 0, "o SQLite deixou de registrar a aresta");
    assert!(
        eventos
            .iter()
            .all(|e| e.starts_with("GET ") || e == "POST protocolo_pesquisa_rapida")
    );
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "GET procedimento_relacionar")
            .count(),
        1
    );
}
