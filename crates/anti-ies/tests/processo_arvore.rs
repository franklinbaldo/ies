//! Releitura de processo após envio: o anti-IES deriva a árvore do SQLite.
//! Hipótese sintética de BDD-333-02/04, sem equivalência com SEI externo.
use anti_ies::{Estado, atender_http};
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use iesei::processos::{Destino, ProcessoSimulado, guardar, ler};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn banco() -> Connection {
    let mut db = Connection::open_in_memory().unwrap();
    iniciar(&db).unwrap();
    guardar(
        &mut db,
        &origem(),
        "bdd-333",
        &ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: Some("Observação".into()),
            destinos: vec![Destino {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Teste".into(),
            }],
        },
    )
    .unwrap();
    db
}

fn pedir(endereco: SocketAddr, metodo: &str, caminho: &str, corpo: &str) -> String {
    let mut cliente = TcpStream::connect(endereco).unwrap();
    write!(
        cliente,
        "{metodo} {caminho} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{corpo}",
        corpo.len()
    )
    .unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    resposta
}

#[test]
fn bdd_333_04_arvore_relida_mostra_unidades_e_anotacao_depois_do_post_unico() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let db = banco();
    let servidor = std::thread::spawn(move || {
        for _ in 0..4 {
            let (mut conexao, _) = escuta.accept().unwrap();
            atender_http(&mut conexao, Some((&db, &origem(), "bdd-333"))).unwrap();
        }
        ler(&db, &origem(), "bdd-333", "10").unwrap().unwrap()
    });
    let arvore = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
    let antes = pedir(endereco, "GET", arvore, "");
    assert!(antes.starts_with("HTTP/1.1 200 OK\r\n"), "{antes}");
    assert!(antes.contains("Nos[0].html ="));
    assert!(antes.contains("ORIGEM"));
    assert!(!antes.contains("DESTINO"));

    let envio = pedir(
        endereco,
        "POST",
        "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10",
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO&chkSinManterAberto=on&chkSinRemoverAnotacoes=on",
    );
    assert!(envio.starts_with("HTTP/1.1 302 Found\r\n"), "{envio}");

    let depois = pedir(endereco, "GET", arvore, "");
    assert!(depois.starts_with("HTTP/1.1 200 OK\r\n"), "{depois}");
    assert!(depois.contains("Processo aberto nas unidades:"));
    assert!(depois.contains("ORIGEM"));
    assert!(depois.contains("DESTINO"));

    let anotacao = pedir(
        endereco,
        "GET",
        "/sei/controlador.php?acao=anotacao_registrar&id_procedimento=10",
        "",
    );
    assert!(anotacao.contains("txaDescricao"));
    assert!(!anotacao.contains("Observação"));
    let estado = servidor.join().unwrap();
    assert_eq!(estado.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(estado.anotacao_origem.as_deref(), Some(""));
}

#[test]
fn processar_navegacao_nao_muta_estado_e_outro_cenario_nao_vaza_dados() {
    let db = banco();
    let antes = ler(&db, &origem(), "bdd-333", "10").unwrap();
    let rota = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
    let navegacao = anti_ies::processo_envio::navegacao("GET", rota, &db, &origem(), "bdd-333");
    assert_eq!(navegacao.unwrap().estado, Estado::Ok);
    assert_eq!(ler(&db, &origem(), "bdd-333", "10").unwrap(), antes);
    let desconhecido = anti_ies::processo_envio::navegacao("GET", rota, &db, &origem(), "outro");
    assert!(desconhecido.is_none());
}
