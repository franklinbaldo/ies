//! BDD-264-01: CLI real de atribuição, TLS local e anti-IES/SQLite.
//! Evidência exclusivamente sintética; não afirma compatibilidade SEI real.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::atribuicoes::{self, Observacao, Usuario};
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
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
const CENARIO: &str = "bdd-264-01-cli";
const NUP: &str = "00001.000001/2026-01";

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

fn banco() -> (Connection, Instalacao) {
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
    processos::guardar(
        &mut db,
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
    atribuicoes::registrar(
        &mut db,
        &origem,
        CENARIO,
        "10",
        "TESTE",
        &Observacao {
            login_atual: "aluno1".into(),
            usuarios: vec![
                Usuario {
                    id: "7".into(),
                    login: "aluno1".into(),
                    nome: "Pessoa 1".into(),
                },
                Usuario {
                    id: "8".into(),
                    login: "aluno2".into(),
                    nome: "Pessoa 2".into(),
                },
            ],
        },
    )
    .unwrap();
    (db, origem)
}

fn executar(json: bool) -> (Output, Vec<String>, usize, String) {
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
        let (db, origem) = banco();
        while let Some(mut canal) = aceitar(&escuta, &parar_servidor) {
            atender_http(&mut canal, Some((&db, &origem, CENARIO))).unwrap();
        }
        (
            processos::ler(&db, &origem, CENARIO, "10")
                .unwrap()
                .unwrap()
                .aberto_em
                .len(),
            atribuicoes::ler(&db, &origem, CENARIO, "10", "TESTE")
                .unwrap()
                .unwrap()
                .login_atual,
        )
    });
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
            eventos_proxy.lock().unwrap().push(evento);
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(if json {
            vec!["--json", "processo", "atribuir", NUP, "--usuario", "aluno2"]
        } else {
            vec!["processo", "atribuir", NUP, "--usuario", "aluno2"]
        })
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
    let (total, atribuicao_final) = servidor.join().unwrap();
    let trilha = eventos.lock().unwrap().clone();
    (saida, trilha, total, atribuicao_final)
}

#[test]
fn bdd_264_01_cli_json_previa_atribuicao_sem_escrever() {
    let (saida, eventos, unidades_abertas, atribuicao_final) = executar(true);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} stdout={} eventos={eventos:?}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["nup"], NUP);
    assert_eq!(valor["unidade"], "TESTE");
    assert_eq!(valor["usuario_atual"], "aluno1");
    assert_eq!(valor["usuario_pedido"], "aluno2");
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert_eq!(valor["requisicoes"].as_u64(), Some(eventos.len() as u64));
    assert_eq!(eventos.len(), 5, "{eventos:?}");
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "GET procedimento_atribuicao_cadastrar")
            .count(),
        1
    );
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.starts_with("POST "))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["POST protocolo_pesquisa_rapida"],
        "nenhum POST de atribuição: {eventos:?}"
    );
    assert_eq!(
        unidades_abertas, 1,
        "bancada não altera unidades do processo"
    );
    assert_eq!(
        atribuicao_final, "aluno1",
        "prévia não grava atribuição no SQLite"
    );
}

#[test]
fn bdd_264_01_cli_humana_exibe_atribuicao_atual_e_pretendida() {
    let (saida, eventos, unidades_abertas, atribuicao_final) = executar(false);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} stdout={} eventos={eventos:?}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let texto = String::from_utf8(saida.stdout).unwrap();
    assert!(texto.contains("Atribuição atual: aluno1"), "{texto}");
    assert!(texto.contains("Atribuir a aluno2"), "{texto}");
    assert_eq!(eventos.len(), 5);
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.starts_with("POST "))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["POST protocolo_pesquisa_rapida"]
    );
    assert_eq!(unidades_abertas, 1);
    assert_eq!(
        atribuicao_final, "aluno1",
        "prévia não grava atribuição no SQLite"
    );
}


#[test]
fn confirmar_publico_recusa_sem_sessao_e_orienta_a_previa() {
    // BDD-264-02/03 ainda pendentes: sem validação live, nem o binário
    // consegue confirmar. O usuário precisa entender o próximo passo.
    // Arquivos privados inexistentes: qualquer tentativa de sessão falha.
    for escolha in [["--usuario", "aluno2"], ["--remover", ""]] {
        let mut argumentos = vec!["--json", "processo", "atribuir", NUP];
        if escolha[0] == "--remover" {
            argumentos.push("--remover");
        } else {
            argumentos.extend(["--usuario", "aluno2"]);
        }
        argumentos.push("--confirmar");
        let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
            .args(&argumentos)
            .env("IES_ORIGEM_FILE", "/arquivo/inexistente/ies-origem")
            .env("IES_COOKIE_FILE", "/arquivo/inexistente/ies-cookie")
            .env("IES_URL_CONTROLE_FILE", "/arquivo/inexistente/ies-url")
            .env("HOME", "/arquivo/inexistente/ies-home")
            .output()
            .unwrap();
        assert_eq!(saida.status.code(), Some(1));
        let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
        assert_eq!(
            json["subcodigo"],
            "escrita_de_atribuicao_ainda_nao_verificada"
        );
        assert!(
            json["mensagem"].as_str().unwrap_or("").contains("prévia"),
            "a recusa precisa explicar a alternativa sem escrita: {json}"
        );
    }
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["processo", "atribuir", NUP, "--usuario", "aluno2", "--confirmar"])
        .env("IES_ORIGEM_FILE", "/arquivo/inexistente/ies-origem")
        .env("IES_COOKIE_FILE", "/arquivo/inexistente/ies-cookie")
        .env("IES_URL_CONTROLE_FILE", "/arquivo/inexistente/ies-url")
        .env("HOME", "/arquivo/inexistente/ies-home")
        .output()
        .unwrap();
    assert_eq!(saida.status.code(), Some(1));
    let erro = String::from_utf8(saida.stderr).unwrap();
    assert!(erro.contains("prévia"), "mensagem humana sem orientação: {erro}");
}
