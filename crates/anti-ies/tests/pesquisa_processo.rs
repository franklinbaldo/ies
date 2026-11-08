//! Busca por NUP no anti-IES por HTTP loopback, com estado SQLite único.
//! Teste técnico da navegação necessária ao BDD-246-01/02.
use anti_ies::atender_http;
use iesei::banco_cassettes::iniciar;
use iesei::documentos_simulados::{self as sim, Processo};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

fn requisitar(endereco: std::net::SocketAddr, metodo: &str, alvo: &str, corpo: &str) -> String {
    let mut fluxo = TcpStream::connect(endereco).unwrap();
    write!(
        fluxo,
        "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{corpo}",
        corpo.len()
    )
    .unwrap();
    fluxo.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    fluxo.read_to_string(&mut resposta).unwrap();
    resposta
}

#[test]
fn pesquisa_do_nup_serve_a_tela_e_a_arvore_sem_criar_documentos() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        sim::registrar_processo(
            &banco,
            &origem,
            "bdd-246",
            &Processo {
                id: "10".into(),
                nup: "00001.000001/2026-01".into(),
                unidade: "TESTE".into(),
            },
        )
        .unwrap();
        for _ in 0..5 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            atender_http(&mut fluxo, Some((&banco, &origem, "bdd-246"))).unwrap();
        }
        assert!(sim::documentos(&banco, &origem, "bdd-246", "10").unwrap().is_empty());
    });
    let busca = "/sei/controlador.php?acao=protocolo_pesquisa_rapida";
    let resultado = requisitar(
        endereco,
        "POST",
        busca,
        "txtPesquisaRapida=00001.000001%2F2026-01",
    );
    assert!(resultado.starts_with("HTTP/1.1 302 Found"));
    assert!(resultado.contains("acao_origem=protocolo_pesquisa_rapida"));
    assert!(resultado.contains("id_protocolo=10"));
    let trabalho = requisitar(
        endereco,
        "GET",
        "/sei/controlador.php?acao=procedimento_trabalhar&id_protocolo=10",
        "",
    );
    assert!(trabalho.starts_with("HTTP/1.1 200 OK"));
    assert!(trabalho.contains("id='ifrArvore'"));
    assert!(trabalho.contains("00001.000001/2026-01"));
    let arvore = requisitar(
        endereco,
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10",
        "",
    );
    assert!(arvore.contains("Nos[0] = new infraArvoreNo"));
    assert!(!arvore.contains("Nos[1] = new infraArvoreNo"));
    let ausente = requisitar(
        endereco,
        "POST",
        busca,
        "txtPesquisaRapida=00001.999999%2F2026-01",
    );
    assert!(ausente.starts_with("HTTP/1.1 404 Not Found"));
    let repetida = requisitar(
        endereco,
        "POST",
        busca,
        "txtPesquisaRapida=00001.000001%2F2026-01&txtPesquisaRapida=00001.000001%2F2026-01",
    );
    assert!(repetida.starts_with("HTTP/1.1 400 Bad Request"));
    servidor.join().unwrap();
}
