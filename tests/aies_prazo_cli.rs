//! BDD-272-01/02: a CLI real lê, prevê e confirma prazo no anti-IES.
//! Hipótese sintética: TLS loopback e um único SQLite efêmero por execução.
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
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-272-01-prazo";
const PEDIDO: &str = "00001.000001/2026-01";

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
    let texto = std::str::from_utf8(&bytes).unwrap();
    let primeira = texto.lines().next().unwrap();
    let evento = primeira.split_whitespace().next().unwrap().to_owned();
    let acao = primeira
        .split_once('?')
        .and_then(|(_, consulta)| {
            consulta
                .split('&')
                .find_map(|parte| parte.strip_prefix("acao="))
        })
        .unwrap_or("sem_acao")
        .to_owned();
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
    bytes.extend(corpo);
    (bytes, format!("{evento} {acao}"))
}

fn estado(caso: &str) -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();

    // A identidade foi observada no Controle, e o prazo é estado sintético
    // explícito no SQLite; sem essa observação não se pode dizer 'sem prazo'.
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
                    numero: PEDIDO.into(),
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
            numero: PEDIDO.into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();
    let (data, concluido): (Option<&str>, bool) = match caso {
        "sem" => (None, false),
        "data" => (Some("20/11/2026"), false),
        "concluido" => (None, true),
        _ => panic!("caso não previsto"),
    };
    banco
        .execute(
            "INSERT INTO prazo_processo
         (origem, cenario, id_procedimento, vence_em, concluido)
         VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![instalacao.url_base(), CENARIO, "10", data, concluido],
        )
        .unwrap();
    (banco, instalacao)
}

fn executar(caso: &str) -> (Output, Vec<String>, (Option<String>, bool)) {
    executar_com_argumentos(caso, &[])
}

fn executar_com_argumentos(
    caso: &str,
    extras: &[&str],
) -> (Output, Vec<String>, (Option<String>, bool)) {
    executar_com_resposta(caso, extras, false)
}

fn executar_com_resposta(
    caso: &str,
    extras: &[&str],
    negar_aceite: bool,
) -> (Output, Vec<String>, (Option<String>, bool)) {
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
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_servidor = Arc::clone(&parar);
    let caso = caso.to_owned();
    let servidor = thread::spawn(move || {
        let (banco, instalacao) = estado(&caso);
        while let Some(mut cliente) = aceitar(&escuta, &parar_servidor) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        banco
            .query_row(
                "SELECT vence_em, concluido FROM prazo_processo
             WHERE origem=?1 AND cenario=?2 AND id_procedimento='10'",
                rusqlite::params![instalacao.url_base(), CENARIO],
                |linha| Ok((linha.get(0)?, linha.get(1)?)),
            )
            .unwrap()
    });
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let registro = Arc::clone(&trilha);
    let parar_proxy = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade_tls).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            let (connect, _) = pedido_http(&mut tcp);
            assert!(connect.starts_with(b"CONNECT"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, evento) = pedido_http(&mut canal);
            let mut backend = TcpStream::connect(endereco).unwrap();
            backend.write_all(&bytes).unwrap();
            backend.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            backend.read_to_end(&mut resposta).unwrap();
            if negar_aceite && evento == "POST controle_prazo_definir" {
                // O SQLite já persistiu o efeito, mas a rede nega o aceite.
                // Uma confirmação insegura não pode declarar sucesso nem refazer POST.
                resposta = b"HTTP/1.1 403 Forbidden\r\nContent-Type: text/html\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec();
            }
            registro.lock().unwrap().push(evento);
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["--json", "processo", "prazo", PEDIDO])
        .args(extras)
        .env("IES_URL_CONTROLE_FILE", controle)
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
    let identidade_final = servidor.join().unwrap();
    let eventos = trilha.lock().unwrap().clone();
    (saida, eventos, identidade_final)
}

#[test]
fn bdd_272_01_cli_real_le_tres_estados_sem_escrita() {
    for (caso, tipo, data) in [
        ("sem", "sem_prazo", None),
        ("data", "data", Some("20/11/2026")),
        ("concluido", "concluido", None),
    ] {
        let (saida, pedidos, estado_final) = executar(caso);
        assert_eq!(
            saida.status.code(),
            Some(0),
            "caso={caso} stdout={} stderr={} pedidos={pedidos:?}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
        assert_eq!(json["nup"], PEDIDO);
        assert_eq!(json["atual"]["tipo"], tipo, "caso={caso}");
        match data {
            Some(data) => assert_eq!(json["atual"]["data"], data),
            None => assert!(json["atual"].get("data").is_none()),
        }
        assert_eq!(json["confirmado"], false);
        assert_eq!(estado_final, (data.map(str::to_owned), caso == "concluido"));
        assert!(
            pedidos.iter().any(|p| p == "GET controle_prazo_definir"),
            "formulário de prazo deve vir do servidor: {pedidos:?}"
        );
        assert!(
            pedidos.iter().any(|p| p == "GET procedimento_visualizar"),
            "a CLI deve abrir a árvore do processo: {pedidos:?}"
        );
        assert!(
            pedidos.iter().all(|p| p.starts_with("GET ")),
            "leitura não pode modificar o simulador: {pedidos:?}"
        );
        assert_eq!(json["requisicoes"], pedidos.len());
    }
}

#[test]
fn bdd_272_02_cli_real_previsualiza_dias_sem_post_e_sem_mutacao() {
    let (saida, pedidos, estado_final) = executar_com_argumentos("sem", &["--dias", "15"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["confirmado"], false);
    assert_eq!(json["solicitado"]["dias"], 15);
    assert_eq!(json["atual"]["tipo"], "sem_prazo");
    assert_eq!(estado_final, (None, false));
    assert!(pedidos.iter().all(|p| p.starts_with("GET ")));
    assert_eq!(json["requisicoes"], pedidos.len());
}

#[test]
fn bdd_272_02_cli_real_confirma_data_por_post_unico_e_releitura() {
    let (saida, pedidos, estado_final) =
        executar_com_argumentos("sem", &["--ate", "20/11/2026", "--confirmar"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["confirmado"], true);
    assert_eq!(json["solicitado"]["ate"], "20/11/2026");
    assert_eq!(estado_final, (Some("20/11/2026".into()), false));
    let escrita = pedidos
        .iter()
        .position(|p| p == "POST controle_prazo_definir")
        .unwrap();
    assert_eq!(pedidos.iter().filter(|p| p.starts_with("POST ")).count(), 1);
    assert!(
        pedidos[escrita + 1..]
            .iter()
            .any(|p| p == "GET controle_prazo_definir"),
        "a releitura independente precisa ocorrer depois do POST: {pedidos:?}"
    );
    assert_eq!(json["requisicoes"], pedidos.len());
}

#[test]
fn bdd_272_02_cli_real_confirma_dias_por_post_unico_e_releitura() {
    let (saida, pedidos, _estado_final) =
        executar_com_argumentos("sem", &["--dias", "15", "--confirmar"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["confirmado"], true);
    assert_eq!(json["solicitado"]["dias"], 15);
    assert_eq!(json["atual"]["tipo"], "dias");
    assert_eq!(json["atual"]["dias"], 15);
    let escrita = pedidos
        .iter()
        .position(|p| p == "POST controle_prazo_definir")
        .unwrap();
    assert_eq!(pedidos.iter().filter(|p| p.starts_with("POST ")).count(), 1);
    assert!(
        pedidos[escrita + 1..]
            .iter()
            .any(|p| p == "GET controle_prazo_definir"),
        "confirmar dias exige leitura nova: {pedidos:?}"
    );
    assert_eq!(json["requisicoes"], pedidos.len());
}

#[test]
fn bdd_272_02_post_aplicado_mas_recusado_retorna_efeito_incerto_sem_retry() {
    let (saida, pedidos, estado_final) =
        executar_com_resposta("sem", &["--ate", "20/11/2026", "--confirmar"], true);
    assert_eq!(
        saida.status.code(),
        Some(3),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["codigo"], "efeito_incerto");
    assert_eq!(json["retentavel"], false);
    assert_eq!(estado_final, (Some("20/11/2026".into()), false));
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p == &"POST controle_prazo_definir")
            .count(),
        1,
        "não repetir o envio inconclusivo: {pedidos:?}"
    );
}


#[test]
fn bdd_272_03_previa_de_exclusao_nao_altera_prazo() {
    let (saida, pedidos, estado_final) =
        executar_com_argumentos("data", &["--excluir"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["confirmado"], false);
    assert_eq!(estado_final, (Some("20/11/2026".into()), false));
    assert!(
        pedidos.iter().any(|p| p == "GET controle_prazo_listar"),
        "exclusão nasce da lista de prazos da unidade: {pedidos:?}"
    );
    assert!(pedidos.iter().all(|p| p.starts_with("GET ")));
}

#[test]
fn bdd_272_03_exclui_pela_lista_com_post_unico_e_releitura() {
    let (saida, pedidos, estado_final) =
        executar_com_argumentos("data", &["--excluir", "--confirmar"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["confirmado"], true);
    assert_eq!(json["atual"]["tipo"], "sem_prazo");
    assert_eq!(estado_final, (None, false));
    let escrita = pedidos
        .iter()
        .position(|p| p == "POST controle_prazo_excluir")
        .expect("exclusão da lista, não definição");
    assert_eq!(pedidos.iter().filter(|p| p.starts_with("POST ")).count(), 1);
    assert!(
        pedidos[escrita + 1..]
            .iter()
            .any(|p| p == "GET controle_prazo_listar"),
        "verificação da remoção é a lista relida: {pedidos:?}"
    );
}

#[test]
fn bdd_272_03_post_exclusao_persistido_sem_aceite_retorna_3_sem_retry() {
    let (saida, pedidos, estado_final) =
        executar_com_resposta("data", &["--excluir", "--confirmar"], true);
    assert_eq!(
        saida.status.code(),
        Some(3),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["codigo"], "efeito_incerto");
    assert_eq!(json["retentavel"], false);
    assert_eq!(estado_final, (None, false));
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p == &"POST controle_prazo_excluir")
            .count(),
        1,
        "não repetir exclusão inconclusiva: {pedidos:?}"
    );
}
