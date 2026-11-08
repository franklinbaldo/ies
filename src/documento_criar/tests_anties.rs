//! Fronteira tipada IES ↔ anti-IES: a lista vem de uma resposta HTTP real.
//! Teste técnico da seleção BDD-246-01/03, não um teste da CLI completa.
use super::{ErroCriar, conferir_tipo, tipos_oferecidos};
use anti_ies::atender_http;
use iesei::banco_cassettes::iniciar;
use iesei::documentos_simulados::{Processo, Tipo, registrar_processo, registrar_tipo};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

#[test]
fn lista_renderizada_pelo_anti_ies_e_consumida_pelo_seletor_do_ies() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        registrar_processo(
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
        registrar_tipo(
            &banco,
            &origem,
            "bdd-246",
            "10",
            &Tipo {
                id: "291".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
        let (mut fluxo, _) = escuta.accept().unwrap();
        atender_http(&mut fluxo, Some((&banco, &origem, "bdd-246"))).unwrap();
    });
    let mut cliente = TcpStream::connect(endereco).unwrap();
    write!(
        cliente,
        "GET /sei/controlador.php?acao=documento_escolher_tipo&id_procedimento=10 HTTP/1.1\r\nHost: localhost\r\n\r\n"
    ).unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    servidor.join().unwrap();
    assert!(resposta.starts_with("HTTP/1.1 200 OK"));
    let (_, html) = resposta.split_once("\r\n\r\n").unwrap();
    let tipos = tipos_oferecidos(html).unwrap();
    assert_eq!(tipos, ["Despacho"]);
    assert_eq!(conferir_tipo("Despacho", &tipos), Ok(()));
    assert_eq!(
        conferir_tipo("Parecer", &tipos),
        Err(ErroCriar::TipoAusente)
    );
    assert_eq!(
        conferir_tipo("Despacho", &["Despacho".into(), "Despacho".into()]),
        Err(ErroCriar::TipoAmbiguo)
    );
}
