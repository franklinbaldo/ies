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
use std::path::Path;

const ORIGEM: &str = "https://sei.treinamento.unb.br/sei";
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

fn executar(json: bool, remover: bool) -> (Output, Vec<String>, usize, String) {
    executar_apos_post_local(json, remover, None)
}

fn executar_apos_post_local(
    json: bool,
    remover: bool,
    escolha_previa: Option<&str>,
) -> (Output, Vec<String>, usize, String) {
    let (saida, _, eventos, unidades, final_atribuicao) =
        executar_cenario(json, remover, escolha_previa, false);
    (saida, eventos, unidades, final_atribuicao)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RespostaPost {
    Original,
    Recusa403,
}

// O backend persiste antes de a bancada substituir o aceite por recusa.
// Assim, o teste distingue estado final de prova causal do próprio POST.
fn responder_local(
    endereco: std::net::SocketAddr,
    pedido: &[u8],
    evento: &str,
    resposta_post: RespostaPost,
) -> Vec<u8> {
    let mut backend = TcpStream::connect(endereco).unwrap();
    backend.write_all(pedido).unwrap();
    backend.shutdown(Shutdown::Write).unwrap();
    let mut resposta = Vec::new();
    backend.read_to_end(&mut resposta).unwrap();
    if resposta_post == RespostaPost::Recusa403
        && evento == "POST procedimento_atribuicao_cadastrar"
    {
        resposta =
            b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec();
    }
    resposta
}

// Uma bancada e a mesma sessão em duas invocações: escrita e releitura
// independente por 'ies processo NUP', sem qualquer rede externa.
fn executar_cenario(
    json: bool,
    remover: bool,
    escolha_previa: Option<&str>,
    confirmar: bool,
) -> (Output, Option<Output>, Vec<String>, usize, String) {
    executar_cenario_com_resposta(
        json,
        remover,
        escolha_previa,
        confirmar,
        RespostaPost::Original,
    )
}

fn executar_cenario_com_resposta(
    json: bool,
    remover: bool,
    escolha_previa: Option<&str>,
    confirmar: bool,
    resposta_post: RespostaPost,
) -> (Output, Option<Output>, Vec<String>, usize, String) {
    let pasta = Pasta::nova();
    let (cert, identidade_tls) = identidade(&pasta.0, "sei.treinamento.unb.br");
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
    if let Some(escolha) = escolha_previa {
        estado_pos_post::aplicar_post_no_sqlite_por_http_local(endereco, escolha);
    }
    let eventos = Arc::new(Mutex::new(Vec::<String>::new()));
    let eventos_proxy = Arc::clone(&eventos);
    let parar_proxy = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade_tls).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            let (connect, _) = pedido(&mut tcp);
            assert!(connect.starts_with(b"CONNECT sei.treinamento.unb.br:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, evento) = pedido(&mut canal);
            let resposta = responder_local(endereco, &bytes, &evento, resposta_post);
            eventos_proxy.lock().unwrap().push(evento);
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let mut argumentos = Vec::new();
    if json {
        argumentos.push("--json");
    }
    argumentos.extend(["processo", "atribuir", NUP]);
    if remover {
        argumentos.push("--remover");
    } else {
        argumentos.extend(["--usuario", "aluno2"]);
    }
    if confirmar {
        argumentos.push("--confirmar");
    }
    let arquivos = [
        url.as_path(),
        cookie.as_path(),
        origem.as_path(),
        cert.as_path(),
    ];
    let saida = executar_cli(arquivos, &pasta, porta, &argumentos);
    let releitura = confirmar.then(|| {
        executar_cli(
            arquivos,
            &pasta,
            porta,
            &[
                "--json",
                "processo",
                NUP,
                "--sem-andamentos",
                "--sem-propriedades",
                "--sem-marcadores",
                "--sem-documentos",
            ],
        )
    });
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let (total, atribuicao_final) = servidor.join().unwrap();
    let trilha = eventos.lock().unwrap().clone();
    (saida, releitura, trilha, total, atribuicao_final)
}

// A mesma sessão privada serve às duas invocações independentes da CLI.
fn executar_cli(arquivos: [&Path; 4], pasta: &Pasta, porta: u16, argumentos: &[&str]) -> Output {
    let [url, cookie, origem, cert] = arquivos;
    Command::new(env!("CARGO_BIN_EXE_ies"))
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
        .unwrap()
}

#[path = "aies_atribuicao_cli/previas.rs"]
mod previas;

#[path = "aies_atribuicao_cli/estado_pos_post.rs"]
mod estado_pos_post;

#[path = "aies_atribuicao_cli/confirmacao.rs"]
mod confirmacao;
