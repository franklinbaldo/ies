//! Regressão HTTP técnica da navegação #60: identidade ambígua não seleciona fallback.
//! São pedidos locais, não um BDD novo; a CLI completa é testada em outros arquivos.
#![cfg(unix)]

use anti_ies::atender_http;
use iesei::banco_cassettes;
use iesei::documento_versoes::{self, DocumentoSimulado, Revisao};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::num::NonZeroU32;
use std::thread;

const CENARIO: &str = "identidade-http";
const ORIGEM: &str = "https://sei.test/sei";

fn estado() -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    for (id, numero, texto) in [
        ("42", "0011519", "Corpo do primeiro"),
        ("43", "0011520", "Corpo do segundo"),
    ] {
        documento_versoes::guardar(
            &mut banco,
            CENARIO,
            &DocumentoSimulado {
                instalacao: origem.clone(),
                id_documento: id.into(),
                id_procedimento: "7".into(),
                numero_processo: "00001.000001/2026-01".into(),
                numero: numero.into(),
                tipo: "Despacho".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "TESTE".into(),
                    unidade: "TESTE".into(),
                    modificacao: "01/11/2026".into(),
                    html_corpo: format!("<p>{texto}</p>"),
                }],
            },
        )
        .unwrap();
    }
    (banco, origem)
}

fn pedir(endereco: std::net::SocketAddr, metodo: &str, caminho: &str) -> String {
    let mut tcp = TcpStream::connect(endereco).unwrap();
    write!(
        tcp,
        "{metodo} /sei/controlador.php?acao=procedimento_visualizar&{caminho} HTTP/1.1\r\nHost: sei.test\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    tcp.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    tcp.read_to_string(&mut resposta).unwrap();
    resposta
}

#[test]
fn http_documento_duplicado_nao_escolhe_id_protocolo_alternativo() {
    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let atendimento = thread::spawn(move || {
        let (banco, origem) = estado();
        for _ in 0..4 {
            let (mut tcp, cliente) = servidor.accept().unwrap();
            assert!(cliente.ip().is_loopback());
            atender_http(&mut tcp, Some((&banco, &origem, CENARIO))).unwrap();
        }
    });

    // Antes da correção, a duplicação fazia o leitor ignorar id_documento
    // e escolher silenciosamente o id_protocolo de outro documento.
    let ambiguo = pedir(endereco, "GET", "id_documento=42&id_documento=42&id_protocolo=43");
    assert!(ambiguo.starts_with("HTTP/1.1 400 Bad Request\r\n"), "{ambiguo}");
    assert!(!ambiguo.contains("0011520"), "não entregar documento alternativo");

    let primeiro = pedir(endereco, "GET", "id_documento=42");
    assert!(primeiro.starts_with("HTTP/1.1 200 OK\r\n"), "{primeiro}");
    assert!(primeiro.contains("0011519"), "árvore do primeiro documento");
    assert!(!primeiro.contains("0011520"), "identidade não se mistura");

    let segundo = pedir(endereco, "GET", "id_documento=43");
    assert!(segundo.starts_with("HTTP/1.1 200 OK\r\n"), "{segundo}");
    assert!(segundo.contains("0011520"), "outro documento segue acessível");

    let metodo = pedir(endereco, "POST", "id_documento=42");
    assert!(metodo.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"), "{metodo}");
    atendimento.join().unwrap();
}
