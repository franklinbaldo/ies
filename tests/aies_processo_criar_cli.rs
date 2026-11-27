//! Criação de processo pela CLI real em estado sintético do anti-IES.
//! Bancada HTTPS exclusiva de loopback com SQLite persistente durante o cenário.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::criacao_processo::{self, TipoDeProcesso};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
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
const CENARIO: &str = "bdd-245-cli";
const TIPO: &str = "Administração Geral: Ensaio";
const ESPECIFICACAO_PERDIDA: &str = "Cadastro com resposta perdida";

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
    let mut corpo = vec![0; tamanho];
    canal.read_exact(&mut corpo).unwrap();
    bytes.extend(corpo);
    (bytes, primeira)
}

fn banco() -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    criacao_processo::registrar(
        &banco,
        &instalacao,
        CENARIO,
        &TipoDeProcesso {
            id: "12".into(),
            nome: TIPO.into(),
            id_assunto: "21".into(),
            assunto: "000.01 - Assunto de Ensaio".into(),
        },
    )
    .unwrap();
    (banco, instalacao)
}

fn quantidade(banco: &Connection) -> i64 {
    banco
        .query_row("SELECT COUNT(*) FROM sim_processo", [], |linha| {
            linha.get(0)
        })
        .unwrap()
}

fn processar_tunel(
    mut tcp: TcpStream,
    tls: &TlsAcceptor,
    destino: std::net::SocketAddr,
    registro: &Mutex<Vec<String>>,
    perder_resposta: bool,
) {
    assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
    tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
        .unwrap();
    let mut canal = tls.accept(tcp).unwrap();
    let (bytes, primeira) = pedido(&mut canal);
    let metodo = primeira.split_whitespace().next().unwrap();
    let alvo = primeira.split_whitespace().nth(1).unwrap();
    // Nunca armazenar a URL, pois pode conter token de sessão.
    let acao = alvo
        .split_once('?')
        .and_then(|(_, query)| {
            query
                .split('&')
                .find_map(|parte| parte.strip_prefix("acao="))
        })
        .unwrap_or("desconhecida");
    let mut backend = TcpStream::connect(destino).unwrap();
    backend.write_all(&bytes).unwrap();
    backend.shutdown(Shutdown::Write).unwrap();
    let mut resposta = Vec::new();
    backend.read_to_end(&mut resposta).unwrap();
    let status = std::str::from_utf8(&resposta)
        .ok()
        .and_then(|valor| valor.split_whitespace().nth(1))
        .unwrap_or("?")
        .to_owned();
    registro
        .lock()
        .unwrap()
        .push(format!("{metodo} {acao} {status}"));
    // Hipótese sintética: o banco aplica o POST, mas a CLI perde a resposta.
    if perder_resposta && metodo == "POST" && acao == "procedimento_gerar" {
        assert_eq!(status, "302");
        return;
    }
    canal.write_all(&resposta).unwrap();
    canal.flush().unwrap();
}

fn executar_em_bancada<T: Send + 'static>(
    args: &[&str],
    reler: bool,
    perder_resposta: bool,
    preparar: fn() -> (Connection, Instalacao),
    observar: fn(&Connection, &Instalacao) -> T,
) -> (Output, Option<Output>, Vec<String>, T, T) {
    executar_em_bancada_com_interferencia(
        args,
        reler,
        perder_resposta,
        preparar,
        observar,
        |_, _| {},
    )
}

fn executar_em_bancada_com_interferencia<T: Send + 'static>(
    args: &[&str],
    reler: bool,
    perder_resposta: bool,
    preparar: fn() -> (Connection, Instalacao),
    observar: fn(&Connection, &Instalacao) -> T,
    interferir: impl Fn(&Connection, &Instalacao) + Send + 'static,
) -> (Output, Option<Output>, Vec<String>, T, T) {
    let pasta = Pasta::nova();
    let (cert, identidade) = identidade(&pasta.0, "sei.test");
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

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let destino = escuta.local_addr().unwrap();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_sim = Arc::clone(&parar);
    let simulador = thread::spawn(move || {
        let (banco, instalacao) = preparar();
        let antes = observar(&banco, &instalacao);
        while let Some(mut cliente) = aceitar(&escuta, &parar_sim) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
            interferir(&banco, &instalacao);
        }
        (antes, observar(&banco, &instalacao))
    });

    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar_proxy = Arc::clone(&parar);
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let registro = Arc::clone(&trilha);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade).unwrap();
        while let Some(tcp) = aceitar(&proxy, &parar_proxy) {
            processar_tunel(tcp, &tls, destino, &registro, perder_resposta);
        }
    });

    let executar_cli = |argumentos: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_ies"))
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
            .unwrap()
    };
    let saida = executar_cli(args);
    let leitura = if reler && saida.status.success() {
        let resultado: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
        let nup = resultado["nup"].as_str().unwrap();
        Some(executar_cli(&[
            "--json",
            "processo",
            nup,
            "--sem-documentos",
            "--sem-andamentos",
            "--sem-marcadores",
        ]))
    } else {
        None
    };
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let (antes, depois) = simulador.join().unwrap();
    let pedidos = trilha.lock().unwrap().clone();
    (saida, leitura, pedidos, antes, depois)
}

fn observar_criacao(banco: &Connection, instalacao: &Instalacao) -> (i64, usize) {
    let tipos = criacao_processo::listar(banco, instalacao, CENARIO)
        .unwrap()
        .unwrap()
        .len();
    (quantidade(banco), tipos)
}

fn executar_com_releitura(
    args: &[&str],
    reler: bool,
    perder_resposta: bool,
    concorrente: bool,
) -> (Output, Option<Output>, Vec<String>, (i64, i64), usize) {
    let (saida, leitura, pedidos, antes, depois) = executar_em_bancada_com_interferencia(
        args,
        reler,
        perder_resposta,
        banco,
        observar_criacao,
        move |banco, instalacao| {
            // Outro ator cria depois do POST e antes da releitura, no mesmo banco.
            // A injeção é sintética e não conta como requisição da CLI sob teste.
            if concorrente && quantidade(banco) == 1 {
                criacao_processo::criar(
                    banco,
                    instalacao,
                    CENARIO,
                    "12",
                    ESPECIFICACAO_PERDIDA,
                    "TESTE",
                )
                .unwrap();
            }
        },
    );
    (saida, leitura, pedidos, (antes.0, depois.0), depois.1)
}

#[path = "aies_processo_criar_cli/envio.rs"]
mod envio;

fn executar(args: &[&str]) -> (Output, Vec<String>, (i64, i64), usize) {
    let (saida, _, pedidos, contagens, tipos) = executar_com_releitura(args, false, false, false);
    (saida, pedidos, contagens, tipos)
}

#[test]
fn previa_por_cli_real_sem_mutacao_sqlite() {
    let (saida, pedidos, contagens, tipos) = executar(&[
        "--json",
        "processo",
        "criar",
        "--tipo",
        TIPO,
        "--especificacao",
        "Ensaio sem gravação",
    ]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    let resultado: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(resultado["confirmado"], false);
    assert_eq!(resultado["tipo"], TIPO);
    assert_eq!(resultado["especificacao"], "Ensaio sem gravação");
    assert_eq!(resultado["assunto"], "000.01 - Assunto de Ensaio");
    assert_eq!(resultado["nivel"], "publico");
    assert_eq!(resultado["unidade"], "TESTE");
    assert!(resultado["nup"].is_null());
    assert_eq!(resultado["requisicoes"], 3);
    assert_eq!(
        pedidos,
        vec![
            "GET procedimento_controlar 200",
            "GET procedimento_escolher_tipo 200",
            "GET procedimento_gerar 200",
        ]
    );
    assert_eq!(contagens, (0, 0), "prévia não pode criar processo");
    assert_eq!(tipos, 1, "tabela de escolha persiste sem ser alterada");
}

#[test]
fn criacao_unica_e_releitura_independente_via_cli_real() {
    let (saida, leitura, pedidos, contagens, tipos) = executar_com_releitura(
        &[
            "--json",
            "processo",
            "criar",
            "--tipo",
            TIPO,
            "--especificacao",
            "Cadastro causal de ensaio",
            "--confirmar",
        ],
        true,
        false,
        false,
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    let resultado: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(resultado["confirmado"], true);
    assert_eq!(resultado["tipo"], TIPO);
    assert_eq!(resultado["nivel"], "publico");
    assert_eq!(resultado["especificacao"], "Cadastro causal de ensaio");
    assert!(resultado["nup"].as_str().is_some_and(|nup| !nup.is_empty()));
    assert!(pedidos.iter().any(|p| p == "POST procedimento_gerar 302"));
    assert!(pedidos.iter().any(|p| p == "GET procedimento_alterar 200"));
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.starts_with("POST procedimento_gerar "))
            .count(),
        1,
        "a consulta independente nunca repete a escrita",
    );
    let leitura = leitura.expect("a criação foi confirmada");
    assert_eq!(
        leitura.status.code(),
        Some(0),
        "consulta do NUP stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&leitura.stdout),
        String::from_utf8_lossy(&leitura.stderr),
    );
    let confirmado: serde_json::Value = serde_json::from_slice(&leitura.stdout).unwrap();
    assert_eq!(confirmado["numero"], resultado["nup"]);
    assert_eq!(confirmado["propriedades"]["tipo"], TIPO);
    assert_eq!(
        confirmado["propriedades"]["especificacao"],
        "Cadastro causal de ensaio"
    );
    assert_eq!(confirmado["propriedades"]["nivel_de_acesso"], "publico");
    assert_eq!(
        confirmado["propriedades"]["assuntos"][0]["codigo"],
        "000.01"
    );
    assert_eq!(
        confirmado["propriedades"]["assuntos"][0]["descricao"],
        "Assunto de Ensaio"
    );
    assert_eq!(confirmado["propriedades"]["data_de_geracao"], "01/01/2026");
    assert_eq!(contagens, (0, 1), "um único processo antes/depois");
    assert_eq!(tipos, 1, "cadastro de tipos permanece");
}

#[test]
fn confirmacao_recusa_tipo_ausente_sem_criar() {
    let (saida, pedidos, contagens, tipos) = executar(&[
        "--json",
        "processo",
        "criar",
        "--tipo",
        "Tipo não cadastrado no cenário",
        "--especificacao",
        "Não deve criar",
        "--confirmar",
    ]);
    assert_eq!(saida.status.code(), Some(1), "pedidos={pedidos:?}");
    assert!(
        pedidos
            .iter()
            .any(|p| p.starts_with("GET procedimento_escolher_tipo "))
    );
    assert!(
        pedidos
            .iter()
            .all(|p| !p.starts_with("POST procedimento_gerar "))
    );
    assert_eq!(contagens, (0, 0));
    assert_eq!(tipos, 1);
}

#[test]
fn confirmacao_recusa_especificacao_acima_do_limite_servido() {
    // O cadastro sintético publica 250 unidades UTF-16, não um limite
    // universal do SEI. 126 caracteres suplementares ocupam 252 unidades.
    let especificacao = "😀".repeat(126);
    let (saida, pedidos, contagens, tipos) = executar(&[
        "--json",
        "processo",
        "criar",
        "--tipo",
        TIPO,
        "--especificacao",
        &especificacao,
        "--confirmar",
    ]);
    assert_eq!(saida.status.code(), Some(1), "pedidos={pedidos:?}");
    assert!(pedidos.iter().any(|p| p == "GET procedimento_gerar 200"));
    assert!(
        pedidos
            .iter()
            .all(|p| !p.starts_with("POST procedimento_gerar "))
    );
    assert_eq!(contagens, (0, 0));
    assert_eq!(tipos, 1);
}

#[path = "aies_processo_criar_cli/concorrencia.rs"]
mod concorrencia;
