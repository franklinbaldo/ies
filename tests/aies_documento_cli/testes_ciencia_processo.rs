//! BDD-271-01/03: ciencia do processo por CLI real, TLS local e SQLite.
//! Hipotese sintetica; cassettes e treinamento sao provas separadas.
use super::*;
use iesei::processos::{self, ProcessoSimulado};

const NUP_CIENTE: &str = "00001.000001/2026-01";

// Bancada de número variável de pedidos: não confunde orçamento sintético
// com número fixado pelo teste. Guarda somente método/ação, nunca URL/cookie.
fn aceitar_ate_parar(
    listener: &TcpListener,
    parar: &std::sync::atomic::AtomicBool,
) -> Option<TcpStream> {
    use std::sync::atomic::Ordering;
    listener.set_nonblocking(true).unwrap();
    loop {
        match listener.accept() {
            Ok((tcp, endereco)) => {
                assert!(endereco.ip().is_loopback());
                tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                tcp.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
                return Some(tcp);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    return None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("loopback: {e}"),
        }
    }
}

fn encaminhar_com_trilha(
    canal: &mut impl Read,
    destino: &mut impl Write,
    trilha: &std::sync::Mutex<Vec<String>>,
) {
    let cab = cabecalho(canal);
    let texto = std::str::from_utf8(&cab).unwrap();
    let linha = texto.lines().next().unwrap();
    let metodo = linha.split_whitespace().next().unwrap();
    let acao = linha
        .split_whitespace()
        .nth(1)
        .unwrap()
        .split_once('?')
        .map(|(_, consulta)| consulta)
        .unwrap_or_default()
        .split('&')
        .find_map(|parte| parte.strip_prefix("acao="))
        .unwrap_or("sem_acao");
    trilha.lock().unwrap().push(format!("{metodo} {acao}"));
    let tamanho = texto
        .lines()
        .find_map(|linha| {
            let (nome, valor) = linha.split_once(':')?;
            nome.eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 64_000);
    destino.write_all(&cab).unwrap();
    let mut corpo = vec![0; tamanho];
    canal.read_exact(&mut corpo).unwrap();
    destino.write_all(&corpo).unwrap();
    destino.flush().unwrap();
}

fn executar_ciencia_processo(pasta: &Pasta, args: &[&str]) -> (Output, Vec<String>) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    let (cert, identidade) = certificado(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    let nome_hash = ["infra", "hash"].join("_");
    let url = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&{nome_hash}={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_servidor = Arc::clone(&parar);
    let banco = pasta.0.join("anti-ies.sqlite3");
    let simulador = thread::spawn(move || {
        let db = Connection::open(banco).unwrap();
        let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
        while let Some(mut tcp) = aceitar_ate_parar(&servidor, &parar_servidor) {
            atender_http(&mut tcp, Some((&db, &instalacao, CENARIO))).unwrap();
        }
    });
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar_proxy = Arc::clone(&parar);
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let capturada = Arc::clone(&trilha);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade).unwrap();
        while let Some(mut tcp) = aceitar_ate_parar(&proxy, &parar_proxy) {
            assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).expect("TLS efemero");
            let mut destino = TcpStream::connect(endereco).unwrap();
            encaminhar_com_trilha(&mut canal, &mut destino, &capturada);
            destino.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            destino.read_to_end(&mut resposta).unwrap();
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .current_dir(&pasta.0)
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
        .expect("CLI");
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    simulador.join().unwrap();
    let pedidos = trilha.lock().unwrap().clone();
    (saida, pedidos)
}

fn preparar_processo(pasta: &Pasta) {
    // Usa o bootstrap do anti-IES ja existente, sem DDL artificial.
    let inicial = executar(pasta, &["--json", "documento", "ciente", DOC], 4);
    assert_eq!(inicial.status.code(), Some(0));
    let mut banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    processos::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &ProcessoSimulado {
            id: "7".into(),
            numero: NUP_CIENTE.into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();
    banco.execute(
        "INSERT INTO sessao_usuario (origem, cenario, usuario) VALUES (?1, ?2, 'agente-de-teste')",
        rusqlite::params![ORIGEM, CENARIO],
    ).unwrap();
}

#[test]
fn bdd_271_01_processo_ciencia_previa_unica_e_lista_distinta() {
    let pasta = Pasta::nova();
    preparar_processo(&pasta);
    let (previa, pedidos_previos) =
        executar_ciencia_processo(&pasta, &["--json", "processo", "ciente", NUP_CIENTE]);
    assert_eq!(
        previa.status.code(),
        Some(0),
        "previa stdout={} stderr={}",
        String::from_utf8_lossy(&previa.stdout),
        String::from_utf8_lossy(&previa.stderr)
    );
    assert_eq!(json(&previa)["verificado"], false);
    assert!(
        !pedidos_previos
            .iter()
            .any(|p| p == "GET procedimento_ciencia")
    );
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let antes: i64 = banco
        .query_row("SELECT COUNT(*) FROM ciencia_processo", [], |r| r.get(0))
        .unwrap();
    assert_eq!(antes, 0, "previa nao escreve");
    drop(banco);

    // Tres GETs de abertura, um da tela, um mutador, nova arvore e uma Lista.
    let (resultado, pedidos_confirmacao) = executar_ciencia_processo(
        &pasta,
        &["--json", "processo", "ciente", NUP_CIENTE, "--confirmar"],
    );
    assert_eq!(
        resultado.status.code(),
        Some(0),
        "confirmacao stdout={} stderr={}",
        String::from_utf8_lossy(&resultado.stdout),
        String::from_utf8_lossy(&resultado.stderr)
    );
    let saida = json(&resultado);
    assert_eq!(saida["verificado"], true);
    assert_eq!(saida["numero"], NUP_CIENTE);
    assert_eq!(saida["id_procedimento"], "7");
    assert_eq!(saida["requisicoes"], pedidos_confirmacao.len());
    assert_eq!(
        pedidos_confirmacao
            .iter()
            .filter(|p| *p == "GET procedimento_ciencia")
            .count(),
        1
    );
    assert_eq!(
        pedidos_confirmacao
            .iter()
            .filter(|p| *p == "GET protocolo_ciencia_listar")
            .count(),
        1
    );
    assert_eq!(saida["usuario"], "agente-de-teste");
    let atividade = saida["id_atividade"]
        .as_str()
        .expect("atividade confirmada");
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let (id, usuario): (String, Option<String>) = banco.query_row(
        "SELECT id_atividade, usuario FROM ciencia_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento='7'",
        rusqlite::params![ORIGEM,CENARIO], |r| Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(id, atividade);
    assert_eq!(usuario.as_deref(), Some("agente-de-teste"));
    let documentos: i64 = banco
        .query_row("SELECT COUNT(*) FROM ciencia_documental", [], |r| r.get(0))
        .unwrap();
    assert_eq!(documentos, 0, "ciencia de processo nao e de documento");
    drop(banco);
    let (repeticao, pedidos_repeticao) = executar_ciencia_processo(
        &pasta,
        &["--json", "processo", "ciente", NUP_CIENTE, "--confirmar"],
    );
    assert!(
        !pedidos_repeticao
            .iter()
            .any(|p| p == "GET procedimento_ciencia")
    );
    assert_eq!(repeticao.status.code(), Some(1), "sem segundo GET mutador");
}

#[test]
fn bdd_271_03_processo_lista_invisivel_e_nao_confirmado_sem_retry() {
    let pasta = Pasta::nova();
    preparar_processo(&pasta);
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    banco.execute(
        "INSERT INTO ciencia_processo_lista_invisivel (origem,cenario,id_procedimento) VALUES (?1,?2,'7')",
        rusqlite::params![ORIGEM,CENARIO]).unwrap();
    drop(banco);
    let (resultado, pedidos_confirmacao) = executar_ciencia_processo(
        &pasta,
        &["--json", "processo", "ciente", NUP_CIENTE, "--confirmar"],
    );
    assert_eq!(
        resultado.status.code(),
        Some(1),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&resultado.stdout),
        String::from_utf8_lossy(&resultado.stderr)
    );
    let saida = json(&resultado);
    assert_eq!(saida["codigo"], "nao_confirmado");
    assert_eq!(saida["retentavel"], false);
    assert_eq!(saida["requisicoes"], pedidos_confirmacao.len());
    assert_eq!(
        pedidos_confirmacao
            .iter()
            .filter(|p| *p == "GET procedimento_ciencia")
            .count(),
        1
    );
    assert_eq!(
        pedidos_confirmacao
            .iter()
            .filter(|p| *p == "GET protocolo_ciencia_listar")
            .count(),
        1
    );
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let atividade: String = banco.query_row(
        "SELECT id_atividade FROM ciencia_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento='7'",
        rusqlite::params![ORIGEM,CENARIO], |r| r.get(0)).unwrap();
    drop(banco);
    let (repeticao, pedidos_repeticao) = executar_ciencia_processo(
        &pasta,
        &["--json", "processo", "ciente", NUP_CIENTE, "--confirmar"],
    );
    assert!(
        !pedidos_repeticao
            .iter()
            .any(|p| p == "GET procedimento_ciencia")
    );
    assert_eq!(repeticao.status.code(), Some(1));
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let atividades: Vec<String> = banco.prepare(
        "SELECT id_atividade FROM ciencia_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento='7'")
        .unwrap().query_map(rusqlite::params![ORIGEM,CENARIO], |r| r.get(0))
        .unwrap().map(Result::unwrap).collect();
    assert_eq!(atividades, vec![atividade], "nenhuma escrita duplicada");
}
