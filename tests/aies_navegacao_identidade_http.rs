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
        "{metodo} /sei/controlador.php?acao=procedimento_visualizar&{caminho} HTTP/1.1\r\nHost: sei.test\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
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
    let ambiguo = pedir(
        endereco,
        "GET",
        "id_documento=42&id_documento=42&id_protocolo=43",
    );
    assert!(
        ambiguo.starts_with("HTTP/1.1 400 Bad Request\r\n"),
        "{ambiguo}"
    );
    assert!(
        !ambiguo.contains("0011520"),
        "não entregar documento alternativo"
    );

    let primeiro = pedir(endereco, "GET", "id_documento=42");
    assert!(primeiro.starts_with("HTTP/1.1 200 OK\r\n"), "{primeiro}");
    assert!(primeiro.contains("0011519"), "árvore do primeiro documento");
    assert!(!primeiro.contains("0011520"), "identidade não se mistura");

    let segundo = pedir(endereco, "GET", "id_documento=43");
    assert!(segundo.starts_with("HTTP/1.1 200 OK\r\n"), "{segundo}");
    assert!(
        segundo.contains("0011520"),
        "outro documento segue acessível"
    );

    let metodo = pedir(endereco, "POST", "id_documento=42");
    assert!(
        metodo.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"),
        "{metodo}"
    );
    atendimento.join().unwrap();
}

#[test]
fn http_arvore_mista_recusa_id_documento_repetido_sem_vazar_pdf() {
    use iesei::pdf_externo::{self, Processo, TipoExterno};

    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let atendimento = thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar(ORIGEM).unwrap();
        pdf_externo::semear(
            &banco,
            &origem,
            CENARIO,
            &Processo {
                id: "7".into(),
                nup: "00001.000001/2026-01".into(),
                unidade: "TESTE".into(),
            },
            &TipoExterno {
                id: "293".into(),
                nome: "Anexo".into(),
                limite_bytes: 1024,
            },
        )
        .unwrap();
        let upload = pdf_externo::receber_conteudo(
            &banco,
            &origem,
            CENARIO,
            "7",
            "prova.pdf",
            b"%PDF-1.4\n%%EOF\n",
        )
        .unwrap();
        let documento =
            pdf_externo::cadastrar(&banco, &origem, CENARIO, "7", "293", upload.id).unwrap();
        let id = documento.id_publico();
        let numero = documento.numero_sei;
        for _ in 0..5 {
            let (mut tcp, cliente) = servidor.accept().unwrap();
            assert!(cliente.ip().is_loopback());
            atender_http(&mut tcp, Some((&banco, &origem, CENARIO))).unwrap();
        }
        (id, numero)
    });

    // O PDF efetivamente existe no SQLite do processo. A rota de
    // árvore mista deve aplicar a mesma recusa que a árvore de versões.
    let valido = pedir(
        endereco,
        "GET",
        "id_procedimento=7&id_documento=8000000000001",
    );
    assert!(valido.starts_with("HTTP/1.1 200 OK\r\n"), "{valido}");

    let ambiguo = pedir(
        endereco,
        "GET",
        "id_procedimento=7&id_documento=8000000000001&id_documento=8000000000001",
    );
    assert!(
        ambiguo.starts_with("HTTP/1.1 400 Bad Request\r\n"),
        "{ambiguo}"
    );
    assert!(
        !ambiguo.contains("050000001"),
        "recusa não publica outro documento"
    );

    // Com id_protocolo repetido, a seleção é ambígua mesmo que os
    // dois valores sejam iguais. Não responder com a árvore inteira.
    let protocolo_ambiguo = pedir(
        endereco,
        "GET",
        "id_procedimento=7&id_protocolo=8000000000001&id_protocolo=8000000000001",
    );
    assert!(
        protocolo_ambiguo.starts_with("HTTP/1.1 400 Bad Request\r\n"),
        "{protocolo_ambiguo}"
    );
    assert!(!protocolo_ambiguo.contains("050000001"));

    let metodo = pedir(
        endereco,
        "POST",
        "id_procedimento=7&id_documento=8000000000001",
    );
    assert!(
        metodo.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"),
        "{metodo}"
    );

    let depois = pedir(
        endereco,
        "GET",
        "id_procedimento=7&id_documento=8000000000001",
    );
    assert!(depois.starts_with("HTTP/1.1 200 OK\r\n"), "{depois}");
    assert!(depois.contains("050000001"), "a recusa não altera o SQLite");
    let (id, numero) = atendimento.join().unwrap();
    assert_eq!(id, "8000000000001");
    assert_eq!(numero, "050000001");
}
