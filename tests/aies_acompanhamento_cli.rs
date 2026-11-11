//! BDD-348-01/03: CLI real de acompanhamento, HTTPS loopback, anti-IES e SQLite.
//! O estado é hipótese sintética; não comprova paridade com a cassette `UnB`.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::acompanhamentos::{self, Acompanhamento, Observacao};
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

#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::falha;

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};

const ORIGEM: &str = "https://sei.treinamento.unb.br/sei";
const CENARIO: &str = "bdd-348-acompanhar";
const NUP: &str = "00001.000001/2026-01";

#[derive(Clone, Copy)]
enum EstadoInicial {
    GrupoG,
    SomenteH,
    VazioObservado,
    NaoObservado,
}

fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match escuta.accept() {
            Ok((conexao, endereco)) => {
                assert!(endereco.ip().is_loopback());
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

fn ler_pedido(canal: &mut impl Read) -> (Vec<u8>, String) {
    let mut cabecalho = Vec::new();
    let mut byte = [0];
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
        .and_then(|(_, q)| q.split('&').find_map(|p| p.strip_prefix("acao=")))
        .unwrap_or("sem_acao");
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
    let evento = format!("{metodo} {acao}");
    cabecalho.extend(corpo);
    (cabecalho, evento)
}

fn banco(estado: EstadoInicial) -> (Connection, Instalacao) {
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
    let itens = match estado {
        EstadoInicial::GrupoG => Some(vec![
            Acompanhamento {
                id: "831".into(),
                grupo: "G".into(),
                observacao: "Alvo".into(),
            },
            Acompanhamento {
                id: "832".into(),
                grupo: "H".into(),
                observacao: "Preservar".into(),
            },
        ]),
        EstadoInicial::SomenteH => Some(vec![Acompanhamento {
            id: "832".into(),
            grupo: "H".into(),
            observacao: "Preservar".into(),
        }]),
        EstadoInicial::VazioObservado => Some(vec![]),
        EstadoInicial::NaoObservado => None,
    };
    if let Some(itens) = itens {
        acompanhamentos::registrar(
            &mut db,
            &origem,
            CENARIO,
            "10",
            "TESTE",
            &Observacao { itens },
        )
        .unwrap();
    }
    (db, origem)
}

fn executar_cli(pasta: &Pasta, porta: u16, cert: &std::path::Path, args: &[&str]) -> Output {
    let origem = pasta.0.join("origem");
    let url = pasta.0.join("url");
    let cookie = pasta.0.join("cookie");
    Command::new(env!("CARGO_BIN_EXE_ies"))
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
        .unwrap()
}

fn executar(estado: EstadoInicial) -> (Output, Vec<String>, Option<Observacao>) {
    let (saida, eventos, estado, _) = executar_com(estado, false);
    (saida, eventos, estado)
}

fn executar_com(
    estado: EstadoInicial,
    confirmar: bool,
) -> (Output, Vec<String>, Option<Observacao>, Option<Output>) {
    executar_com_falha(estado, confirmar, false)
}

fn executar_com_falha(
    estado: EstadoInicial,
    confirmar: bool,
    falha_na_releitura: bool,
) -> (Output, Vec<String>, Option<Observacao>, Option<Output>) {
    executar_com_falha_modo(estado, confirmar, falha_na_releitura, false)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RespostaDeReleitura {
    Normal,
    Http503,
    Redirecionamento302,
    SemTabela,
}

fn resposta_injetada(resposta: RespostaDeReleitura) -> &'static [u8] {
    match resposta {
        RespostaDeReleitura::Http503 =>
            b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        RespostaDeReleitura::Redirecionamento302 =>
            b"HTTP/1.1 302 Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        RespostaDeReleitura::SemTabela =>
            b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: 13\r\nConnection: close\r\n\r\n<html></html>",
        RespostaDeReleitura::Normal => unreachable!(),
    }
}

fn executar_com_falha_modo(
    estado: EstadoInicial,
    confirmar: bool,
    falha_na_releitura: bool,
    modo_humano: bool,
) -> (Output, Vec<String>, Option<Observacao>, Option<Output>) {
    let resposta = if falha_na_releitura {
        RespostaDeReleitura::Http503
    } else {
        RespostaDeReleitura::Normal
    };
    executar_com_resposta_modo(estado, confirmar, resposta, modo_humano)
}

fn executar_com_resposta_modo(
    estado: EstadoInicial,
    confirmar: bool,
    resposta_na_releitura: RespostaDeReleitura,
    modo_humano: bool,
) -> (Output, Vec<String>, Option<Observacao>, Option<Output>) {
    let pasta = Pasta::nova();
    let (certificado, identidade_tls) = identidade(&pasta.0, "sei.treinamento.unb.br");
    let _origem = pasta.privado("origem", ORIGEM);
    let hash_chave = ["infra", "hash"].join("_");
    let _url = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&{hash_chave}={}",
            "a".repeat(64)
        ),
    );
    let _cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_servidor = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        let (db, origem) = banco(estado);
        while let Some(mut canal) = aceitar(&escuta, &parar_servidor) {
            atender_http(&mut canal, Some((&db, &origem, CENARIO))).unwrap();
        }
        acompanhamentos::ler(&db, &origem, CENARIO, "10", "TESTE").unwrap()
    });
    let eventos = Arc::new(Mutex::new(Vec::<String>::new()));
    let eventos_proxy = Arc::clone(&eventos);
    let parar_proxy = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade_tls).unwrap();
        let mut post_recebido = false;
        let mut leitura_interceptada = false;
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            let (connect, _) = ler_pedido(&mut tcp);
            assert!(connect.starts_with(b"CONNECT sei.treinamento.unb.br:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, evento) = ler_pedido(&mut canal);
            let interceptar = resposta_na_releitura != RespostaDeReleitura::Normal
                && post_recebido
                && !leitura_interceptada
                && evento == "GET acompanhamento_gerenciar";
            if interceptar {
                leitura_interceptada = true;
                eventos_proxy.lock().unwrap().push(evento);
                canal
                    .write_all(resposta_injetada(resposta_na_releitura))
                    .unwrap();
                canal.flush().unwrap();
                continue;
            }
            if evento == "POST acompanhamento_excluir" {
                post_recebido = true;
            }
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
    let mut args = vec![
        "--json",
        "processo",
        "acompanhar",
        NUP,
        "--deixar",
        "--grupo",
        "G",
    ];
    if confirmar {
        args.push("--confirmar");
    }
    if modo_humano {
        args.remove(0); // --json: exercitar a saída humana do mesmo BDD.
    }
    let saida = executar_cli(&pasta, porta, &certificado, &args);
    // Outra invocação da CLI, após a escrita, deve comprovar o novo estado.
    let verificacao =
        confirmar.then(|| executar_cli(&pasta, porta, &certificado, &args[..args.len() - 1]));
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let resultado = servidor.join().unwrap();
    (
        saida,
        eventos.lock().unwrap().clone(),
        resultado,
        verificacao,
    )
}

#[test]
fn bdd_348_01_previa_identifica_g_sem_remover_h() {
    let (saida, eventos, depois) = executar(EstadoInicial::GrupoG);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} eventos={eventos:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["operacao"], "deixar");
    assert_eq!(valor["grupo"], "G");
    assert_eq!(valor["id_acompanhamento"], "831");
    assert_eq!(valor["ja_estava"], false);
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert_eq!(valor["requisicoes"].as_u64(), Some(eventos.len() as u64));
    let itens = depois.unwrap().itens;
    assert_eq!(itens.len(), 2);
    assert_eq!(itens[0].id, "831");
    assert_eq!(itens[1].id, "832");
    assert!(
        eventos
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "{eventos:?}"
    );
    assert!(eventos.iter().any(|p| p == "GET acompanhamento_gerenciar"));
}

#[test]
fn bdd_348_03_grupo_g_ausente_nao_toca_h() {
    let (saida, eventos, depois) = executar(EstadoInicial::SomenteH);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} eventos={eventos:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["operacao"], "deixar");
    assert_eq!(valor["grupo"], "G");
    assert!(valor["id_acompanhamento"].is_null());
    assert_eq!(valor["ja_estava"], true);
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    let itens = depois.unwrap().itens;
    assert_eq!(itens.len(), 1);
    assert_eq!(itens[0].id, "832");
    assert_eq!(itens[0].grupo, "H");
    assert!(
        eventos
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "{eventos:?}"
    );
}

#[test]
fn observacao_ausente_nao_inventa_lista_vazia() {
    let (saida, eventos, depois) = executar(EstadoInicial::NaoObservado);
    assert_ne!(saida.status.code(), Some(0));
    assert!(depois.is_none());
    assert!(
        eventos
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "{eventos:?}"
    );
}

#[test]
fn observacao_explicitamente_vazia_permite_no_op_sem_escrita() {
    let (saida, eventos, depois) = executar(EstadoInicial::VazioObservado);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} eventos={eventos:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["operacao"], "deixar");
    assert_eq!(valor["grupo"], "G");
    assert!(valor["id_acompanhamento"].is_null());
    assert_eq!(valor["ja_estava"], true);
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert_eq!(depois.unwrap().itens, Vec::<Acompanhamento>::new());
    assert!(eventos.iter().any(|p| p == "GET acompanhamento_gerenciar"));
    assert!(
        eventos
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "{eventos:?}"
    );
}

#[path = "aies_acompanhamento_cli/confirmacao.rs"]
mod confirmacao;
