//! BDD-ANOT-01: prévia da anotação pela CLI real, TLS local e SQLite.
//! Caso sintético: não equivale a replay de cassette nem a treinamento.
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
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-anot-01-previa";
const NUP: &str = "00001.000001/2026-01";

fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match escuta.accept() {
            Ok((cliente, endereco)) => {
                assert!(endereco.ip().is_loopback());
                cliente
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                cliente
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return Some(cliente);
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

fn pedido_http(fonte: &mut impl Read) -> (Vec<u8>, String) {
    let mut bytes = Vec::new();
    let mut byte = [0u8];
    while !bytes.ends_with(b"\r\n\r\n") {
        assert!(bytes.len() < 16_384);
        fonte.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
    }
    let cabecalho = std::str::from_utf8(&bytes).unwrap();
    let primeira = cabecalho.lines().next().unwrap();
    let metodo = primeira.split_whitespace().next().unwrap();
    let acao = primeira
        .split_once('?')
        .and_then(|(_, consulta)| consulta.split('&').find_map(|s| s.strip_prefix("acao=")))
        .unwrap_or("sem_acao");
    let evento = format!("{metodo} {acao}");
    let tamanho = cabecalho
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
    bytes.extend(corpo);
    (bytes, evento)
}

fn estado(texto_anotacao: &str) -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: instalacao.clone(),
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
        &instalacao,
        CENARIO,
        &ProcessoSimulado {
            id: "10".into(),
            numero: NUP.into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: Some(texto_anotacao.to_owned()),
            destinos: vec![],
        },
    )
    .unwrap();
    (banco, instalacao)
}

#[derive(Clone, Copy)]
enum Caso<'a> {
    Consulta(&'a str),
    Registro,
    Retirada,
}

#[derive(Clone, Copy)]
enum Resultado {
    Previa,
    Confirmado,
    Recusado,
}

#[expect(
    clippy::too_many_lines,
    reason = "prova vertical única de prévia e confirmação pela CLI real, TLS, anti-IES e SQLite"
)]
fn executar_caso(caso: Caso<'_>, resultado: Resultado, modo_json: bool) {
    let confirmar = !matches!(resultado, Resultado::Previa);
    let resposta_recusada = matches!(resultado, Resultado::Recusado);
    let remover = matches!(caso, Caso::Retirada);
    let consulta = match caso {
        Caso::Consulta(texto) => Some(texto),
        Caso::Registro => None,
        Caso::Retirada => Some("Rascunho"),
    };
    let pasta = Pasta::nova();
    let (cert, identidade_tls) = identidade(&pasta.0, "sei.test");
    let origem = pasta.privado("origem", ORIGEM);
    let chave = ["infra", "hash"].join("_");
    let controle = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual=110000001&{chave}={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let backend = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    backend.set_nonblocking(true).unwrap();
    let endereco = backend.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let sinal = Arc::clone(&parar);
    let texto_inicial = consulta.unwrap_or("").to_owned();
    let servidor = thread::spawn(move || {
        let (banco, instalacao) = estado(&texto_inicial);
        while let Some(mut cliente) = aceitar(&backend, &sinal) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        processos::ler(&banco, &instalacao, CENARIO, "10")
            .unwrap()
            .unwrap()
            .anotacao_origem
    });
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let pedidos = Arc::clone(&trilha);
    let sinal = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade_tls).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &sinal) {
            let (connect, _) = pedido_http(&mut tcp);
            assert!(connect.starts_with(b"CONNECT"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (req, evento) = pedido_http(&mut canal);
            let mut destino = TcpStream::connect(endereco).unwrap();
            destino.write_all(&req).unwrap();
            destino.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            destino.read_to_end(&mut resposta).unwrap();
            let recusado = resposta_recusada && evento == "POST anotacao_registrar";
            pedidos.lock().unwrap().push(evento);
            if recusado {
                canal
                    .write_all(
                        b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .unwrap();
            } else {
                canal.write_all(&resposta).unwrap();
            }
            canal.flush().unwrap();
        }
    });
    let mut argumentos = if modo_json {
        vec!["--json", "processo", "anotar", NUP]
    } else {
        vec!["processo", "anotar", NUP]
    };
    if consulta.is_none() && !remover {
        argumentos.extend(["--texto", "Revisar minuta"]);
    }
    if remover {
        argumentos.push("--remover");
    }
    if confirmar {
        argumentos.push("--confirmar");
    }
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(argumentos)
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
    let persistido = servidor.join().unwrap();
    let pedidos = trilha.lock().unwrap().clone();
    assert_eq!(
        saida.status.code(),
        Some(i32::from(resposta_recusada)),
        "stdout={} stderr={} HTTP={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    if !remover && let Some(texto) = consulta {
        if modo_json {
            let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
            assert_eq!(json["nup"], NUP);
            assert_eq!(
                json["anotacao"],
                if texto.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::json!(texto)
                }
            );
            assert_eq!(json["alterado"], false);
            assert_eq!(json["requisicoes"], pedidos.len());
        } else {
            let humana = String::from_utf8_lossy(&saida.stdout);
            assert!(humana.contains(texto), "{humana}");
            if !texto.is_empty() {
                assert!(humana.contains("Anotação atual do processo"), "{humana}");
                assert!(humana.contains(&format!("{texto:?}")), "{humana}");
            }
            assert!(humana.contains("Nenhuma alteração"), "{humana}");
        }
        assert_eq!(persistido.as_deref(), Some(texto));
        assert!(
            pedidos.iter().all(|evento| evento.starts_with("GET ")),
            "{pedidos:?}"
        );
        assert_eq!(
            pedidos
                .iter()
                .filter(|e| *e == "GET anotacao_registrar")
                .count(),
            1
        );
        return;
    }
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    if resposta_recusada {
        // O servidor aplicou, mas o túnel devolveu 403: estado final não prova autoria.
        assert_eq!(json["codigo"], "nao_confirmado");
        assert_eq!(json["retentavel"], false);
    } else {
        assert_eq!(json["nup"], NUP);
        let texto_esperado = if remover { "" } else { "Revisar minuta" };
        let anotacao_esperada = if remover { "Rascunho" } else { "" };
        assert_eq!(json["texto"], texto_esperado);
        assert_eq!(json["anotacao_antes"], anotacao_esperada);
        assert_eq!(json["confirmado"], confirmar);
        assert_eq!(json["alterado"], confirmar);
        assert_eq!(json["requisicoes"], pedidos.len());
    }
    let anotacao_esperada = match (remover, confirmar) {
        (true, true) | (false, false) => "",
        (true, false) => "Rascunho",
        (false, true) => "Revisar minuta",
    };
    assert_eq!(persistido.as_deref(), Some(anotacao_esperada));
    assert_eq!(
        pedidos
            .iter()
            .filter(|e| *e == "GET anotacao_registrar")
            .count(),
        if confirmar { 2 } else { 1 }
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|e| *e == "POST anotacao_registrar")
            .count(),
        usize::from(confirmar)
    );
}

#[test]
fn bdd_anot_01_previa_publica_texto_sem_escrever() {
    executar_caso(Caso::Registro, Resultado::Previa, true);
}

#[test]
fn bdd_anot_02_confirma_uma_vez_e_rele_retorno_da_anotacao() {
    executar_caso(Caso::Registro, Resultado::Confirmado, true);
}

#[test]
fn bdd_anot_02_403_apos_post_aplicado_exige_nao_confirmado() {
    executar_caso(Caso::Registro, Resultado::Recusado, true);
}

#[test]
fn bdd_anot_03_consulta_texto_publicado_sem_post() {
    executar_caso(Caso::Consulta("Revisar minuta"), Resultado::Previa, true);
    executar_caso(Caso::Consulta("Revisar minuta"), Resultado::Previa, false);
    executar_caso(Caso::Consulta("   "), Resultado::Previa, true);
    executar_caso(Caso::Consulta("   "), Resultado::Previa, false);
}

#[test]
fn bdd_anot_03_consulta_sem_anotacao_publica_null() {
    executar_caso(Caso::Consulta(""), Resultado::Previa, true);
}

#[test]
fn bdd_anot_04_previa_remocao_sem_post() {
    executar_caso(Caso::Retirada, Resultado::Previa, true);
}

#[test]
fn bdd_anot_04_remove_uma_vez_e_confere_vazio_por_releitura() {
    executar_caso(Caso::Retirada, Resultado::Confirmado, true);
}

#[test]
fn bdd_anot_04_403_apos_remocao_aplicada_exige_nao_confirmado() {
    executar_caso(Caso::Retirada, Resultado::Recusado, true);
}
