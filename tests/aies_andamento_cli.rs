//! BDD-259-01/02/03: CLI real contra HTTPS local, anti-IES e SQLite.
//! Hipótese sintética, não prova paridade com uma instalação SEI.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::andamentos::{self, Historico, Linha as LinhaDoHistorico};
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use iesei::processos::{self, ProcessoSimulado};
use native_tls::{Identity, TlsAcceptor};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-259-01-andamento";
const NUP: &str = "00001.000001/2026-01";
const TEXTO: &str = "  Registro\nnovo  ";

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
    let primeira = texto.lines().next().unwrap();
    let metodo = primeira.split_whitespace().next().unwrap();
    let acao = primeira
        .split_once('?')
        .and_then(|(_, query)| query.split('&').find_map(|s| s.strip_prefix("acao=")))
        .unwrap_or("sem_acao");
    let evento = format!("{metodo} {acao}");
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
    (cabecalho, evento)
}

fn estado() -> (Connection, Instalacao) {
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
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();
    andamentos::guardar(
        &banco,
        &instalacao,
        CENARIO,
        "10",
        &Historico {
            total_declarado: 1,
            linhas: vec![LinhaDoHistorico {
                ocorrido_em: "09/11/2026 10:00".into(),
                unidade: "TESTE".into(),
                usuario: "aluno".into(),
                descricao: "Andamento anterior".into(),
            }],
        },
    )
    .unwrap();
    (banco, instalacao)
}

fn servir_tunel(
    proxy: TcpListener,
    endereco: SocketAddr,
    tls_id: Identity,
    sinal: &AtomicBool,
    pedidos: &Mutex<Vec<String>>,
    injecao: &AtomicBool,
    falha_releitura: bool,
) {
    let tls = TlsAcceptor::new(tls_id).unwrap();
    let mut post_confirmado = false;
    while let Some(mut tcp) = aceitar(&proxy, sinal) {
        let (connect, _) = pedido_http(&mut tcp);
        assert!(connect.starts_with(b"CONNECT"));
        tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
            .unwrap();
        let mut canal = tls.accept(tcp).unwrap();
        let (req, evento) = pedido_http(&mut canal);
        if falha_releitura
            && post_confirmado
            && !injecao.load(Ordering::SeqCst)
            && evento.starts_with("GET ")
        {
            // O POST chegou ao anti-IES e persistiu no SQLite. Só a
            // primeira leitura posterior fica indisponível (hipótese).
            pedidos.lock().unwrap().push(evento);
            injecao.store(true, Ordering::SeqCst);
            canal
                .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .unwrap();
            canal.flush().unwrap();
            continue;
        }
        let mut destino = TcpStream::connect(endereco).unwrap();
        destino.write_all(&req).unwrap();
        destino.shutdown(Shutdown::Write).unwrap();
        let mut resposta = Vec::new();
        destino.read_to_end(&mut resposta).unwrap();
        if evento == "POST procedimento_atualizar_andamento" {
            post_confirmado = true;
        }
        pedidos.lock().unwrap().push(evento);
        canal.write_all(&resposta).unwrap();
        canal.flush().unwrap();
    }
}

fn executar(confirmar: bool, falha_releitura: bool) {
    let pasta = Pasta::nova();
    let (cert, tls_id) = identidade(&pasta.0, "sei.test");
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
    let porta_proxy = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let sinal = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        let (banco, instalacao) = estado();
        while let Some(mut cliente) = aceitar(&backend, &sinal) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        andamentos::ler(&banco, &instalacao, CENARIO, "10")
            .unwrap()
            .unwrap()
    });
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let pedidos = Arc::clone(&trilha);
    let sinal = Arc::clone(&parar);
    let falha_aplicada = Arc::new(AtomicBool::new(false));
    let injecao = Arc::clone(&falha_aplicada);
    let tunel = thread::spawn(move || {
        servir_tunel(
            proxy,
            endereco,
            tls_id,
            &sinal,
            &pedidos,
            &injecao,
            falha_releitura,
        );
    });
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args({
            let mut argumentos = vec!["--json", "processo", "andamento", NUP, "--texto", TEXTO];
            if confirmar {
                argumentos.push("--confirmar");
            }
            argumentos
        })
        .env("IES_URL_CONTROLE_FILE", &controle)
        .env("IES_COOKIE_FILE", &cookie)
        .env("IES_ORIGEM_FILE", &origem)
        .env("SSL_CERT_FILE", &cert)
        .env("https_proxy", format!("http://127.0.0.1:{porta_proxy}"))
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
    let segunda =
        confirmar.then(|| segunda_leitura(&pasta, &controle, &cookie, &origem, &cert, porta_proxy));
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let persistido = servidor.join().unwrap();
    let pedidos = trilha.lock().unwrap();
    if falha_releitura {
        assert!(
            falha_aplicada.load(Ordering::SeqCst),
            "falha pós-POST não exercitada"
        );
        verificar_incerto(&saida, segunda.as_ref(), &persistido, &pedidos);
    } else {
        verificar(confirmar, &saida, segunda.as_ref(), &persistido, &pedidos);
    }
}

fn segunda_leitura(
    pasta: &Pasta,
    controle: &std::path::Path,
    cookie: &std::path::Path,
    origem: &std::path::Path,
    cert: &std::path::Path,
    porta_proxy: u16,
) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "--json",
            "processo",
            NUP,
            "--andamentos-completos",
            "--sem-documentos",
            "--sem-propriedades",
            "--sem-marcadores",
        ])
        .env("IES_URL_CONTROLE_FILE", controle)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("SSL_CERT_FILE", cert)
        .env("https_proxy", format!("http://127.0.0.1:{porta_proxy}"))
        .env("HOME", &pasta.0)
        .env_remove("HTTPS_PROXY")
        .env_remove("all_proxy")
        .env_remove("ALL_PROXY")
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE")
        .output()
        .unwrap()
}

fn verificar(
    confirmar: bool,
    saida: &std::process::Output,
    segunda: Option<&Output>,
    persistido: &Historico,
    pedidos: &[String],
) {
    assert_eq!(persistido.total_declarado, if confirmar { 2 } else { 1 });
    assert_eq!(persistido.linhas.len(), if confirmar { 2 } else { 1 });
    assert_eq!(
        persistido.linhas[0].descricao,
        if confirmar {
            TEXTO
        } else {
            "Andamento anterior"
        }
    );
    assert_eq!(
        persistido.linhas.last().unwrap().descricao,
        "Andamento anterior"
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["numero"], NUP);
    assert_eq!(json["texto"], TEXTO);
    assert_eq!(json["confirmado"], confirmar);
    assert_eq!(json["alterado"], confirmar);
    assert_eq!(json["total_antes"], 1);
    assert_eq!(
        json["total_depois"],
        if confirmar {
            serde_json::json!(2)
        } else {
            serde_json::Value::Null
        }
    );
    assert!(
        pedidos
            .iter()
            .any(|p| p == "GET procedimento_consultar_historico")
    );
    assert!(
        pedidos
            .iter()
            .any(|p| p == "GET procedimento_atualizar_andamento")
    );
    if confirmar {
        assert_eq!(
            pedidos
                .iter()
                .filter(|p| p.as_str() == "POST procedimento_atualizar_andamento")
                .count(),
            1,
            "{pedidos:?}"
        );
        assert_eq!(
            pedidos.last().map(String::as_str),
            Some("POST procedimento_consultar_historico")
        );
        let segunda = segunda.expect("segunda CLI do BDD-259-02");
        assert_eq!(
            segunda.status.code(),
            Some(0),
            "segunda CLI: stdout={} stderr={}",
            String::from_utf8_lossy(&segunda.stdout),
            String::from_utf8_lossy(&segunda.stderr)
        );
        let resultado: serde_json::Value = serde_json::from_slice(&segunda.stdout).unwrap();
        assert_eq!(resultado["andamentos"]["tipo"], "completo");
        assert_eq!(resultado["andamentos"]["total_declarado"], 2);
        let itens = resultado["andamentos"]["itens"].as_array().unwrap();
        assert_eq!(itens.len(), 2);
        assert_eq!(
            itens[0]["descricao"]
                .as_str()
                .unwrap()
                .split_whitespace()
                .collect::<Vec<_>>(),
            TEXTO.split_whitespace().collect::<Vec<_>>(),
        );
        assert_eq!(itens[1]["descricao"], "Andamento anterior");
    } else {
        assert!(pedidos.iter().all(|p| p.starts_with("GET ")), "{pedidos:?}");
    }
}

fn verificar_incerto(
    saida: &Output,
    segunda: Option<&Output>,
    persistido: &Historico,
    pedidos: &[String],
) {
    assert_eq!(
        saida.status.code(),
        Some(3),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["codigo"], "efeito_incerto");
    assert_eq!(json["subcodigo"], "efeito_incerto_no_andamento");
    let mensagem = json["erro"].as_str().unwrap();
    assert!(mensagem.contains("não repita --confirmar"), "{mensagem}");
    assert!(mensagem.contains("histórico"), "{mensagem}");

    // A falha na releitura da primeira CLI não reverte o POST já aplicado.
    // A segunda CLI é independente e consulta o mesmo SQLite.
    assert_eq!(persistido.total_declarado, 2);
    assert_eq!(persistido.linhas.len(), 2);
    assert_eq!(persistido.linhas[0].descricao, TEXTO);
    assert_eq!(persistido.linhas[1].descricao, "Andamento anterior");
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.as_str() == "POST procedimento_atualizar_andamento")
            .count(),
        1,
        "{pedidos:?}"
    );
    let leitura = segunda.expect("consulta independente do histórico");
    assert_eq!(
        leitura.status.code(),
        Some(0),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&leitura.stdout),
        String::from_utf8_lossy(&leitura.stderr)
    );
    let historico: serde_json::Value = serde_json::from_slice(&leitura.stdout).unwrap();
    let itens = historico["andamentos"]["itens"].as_array().unwrap();
    assert_eq!(itens.len(), 2);
    assert_eq!(
        itens[0]["descricao"]
            .as_str()
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>(),
        TEXTO.split_whitespace().collect::<Vec<_>>()
    );
    assert_eq!(itens[1]["descricao"], "Andamento anterior");
}

#[test]
fn bdd_259_01_previa_nao_escreve_no_historico() {
    executar(false, false);
}

#[test]
fn bdd_259_02_confirmacao_unica_com_releitura_da_cli() {
    executar(true, false);
}

#[test]
fn bdd_259_03_efeito_incerto_sem_reenvio_e_consulta_independente() {
    executar(true, true);
}
