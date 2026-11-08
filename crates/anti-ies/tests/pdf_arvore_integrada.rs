//! #332: árvore pública deve incluir documentos internos e PDFs definitivos.
use iesei::banco_cassettes::iniciar;
use iesei::documentos_simulados::{self as interno, Tipo};
use iesei::instalacao::Instalacao;
use iesei::pdf_externo::{self as externo, Processo, TipoExterno};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

fn pedir(endereco: SocketAddr, alvo: &str) -> String {
    let mut fluxo = TcpStream::connect(endereco).unwrap();
    write!(fluxo, "GET {alvo} HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
    fluxo.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    fluxo.read_to_string(&mut resposta).unwrap();
    resposta
}

#[test]
fn bdd_332_03_pdf_so_aparece_na_arvore_js_apos_cadastro() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        externo::semear(
            &banco,
            &origem,
            "caso",
            &Processo {
                id: "7".into(),
                nup: "00001.000001/2026-01".into(),
                unidade: "TESTE".into(),
            },
            &TipoExterno {
                id: "293".into(),
                nome: "Anexo".into(),
                limite_bytes: 100_000,
            },
        )
        .unwrap();
        interno::registrar_tipo(
            &banco, &origem, "caso", "7",
            &Tipo {
                id: "291".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
        interno::criar(&banco, &origem, "caso", "7", "291", "Primeiro").unwrap();

        let (mut conexao, _) = escuta.accept().unwrap();
        anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "caso"))).unwrap();
        let temporario = externo::receber(&banco, &origem, "caso", "7", "prova.pdf", 42).unwrap();
        externo::cadastrar(&banco, &origem, "caso", "7", "293", temporario.id).unwrap();
        let (mut conexao, _) = escuta.accept().unwrap();
        anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "caso"))).unwrap();
    });
    let alvo = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento=";
    let antes = pedir(endereco, alvo);
    assert!(antes.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(antes.contains("Nos[1] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(antes.contains("Despacho"));
    assert!(!antes.contains("050000001"));
    let depois = pedir(endereco, alvo);
    assert!(depois.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(depois.contains("Despacho"));
    assert!(depois.contains("Nos[2] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(depois.contains("Anexo 050000001"));
    servidor.join().unwrap();
}
