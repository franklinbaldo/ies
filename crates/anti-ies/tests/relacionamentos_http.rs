//! BDD-258-01: a tela de relacionamentos nasce de estado SQLite observado.
//! Ensaio HTTP local; o teste do executável IES cobre a mesma prévia separadamente.
use anti_ies::atender_http;
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use iesei::processos::{self, ProcessoSimulado};
use iesei::relacionamentos;
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

#[test]
fn bdd_258_01_previa_relacionamentos_observados_nao_fabrica_vinculos() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
    let cenario = "bdd-258-01";
    for (id, numero) in [
        ("10", "00001.000001/2026-01"),
        ("22", "00001.000002/2026-01"),
    ] {
        processos::guardar(
            &mut banco,
            &origem,
            cenario,
            &ProcessoSimulado {
                id: id.into(),
                numero: numero.into(),
                unidade_origem: "TESTE".into(),
                aberto_em: vec!["TESTE".into()],
                anotacao_origem: None,
                destinos: vec![],
            },
        )
        .unwrap();
    }
    relacionamentos::observar(&banco, &origem, cenario, "10").unwrap();
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let (mut canal, _) = escuta.accept().unwrap();
        atender_http(&mut canal, Some((&banco, &origem, cenario))).unwrap();
    });
    let mut canal = TcpStream::connect(endereco).unwrap();
    canal.write_all(
        b"GET /sei/controlador.php?acao=procedimento_relacionar&id_procedimento=10 HTTP/1.1\r\nHost: sei.test\r\n\r\n",
    ).unwrap();
    canal.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    canal.read_to_string(&mut resposta).unwrap();
    servidor.join().unwrap();
    assert!(resposta.starts_with("HTTP/1.1 200 OK"), "{resposta}");
    assert!(resposta.contains("frmProcedimentoListarRelacionamentos"));
    assert!(resposta.contains("txtProtocolo"));
    assert!(resposta.contains("hdnIdProtocolo"));
    assert!(
        !resposta.contains(">00001.000002/2026-01</a>"),
        "prévia não inventa vínculo"
    );
}
