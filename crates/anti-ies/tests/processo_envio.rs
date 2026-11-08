//! BDD-333-03/04: transições de um processo único no SQLite efêmero.
//! Provas sintéticas; não implicam equivalência com o SEI real.
use anti_ies::{Estado, processo_envio::responder};
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use iesei::processos::{Destino, ProcessoSimulado, guardar, ler};
use rusqlite::Connection;

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn preparar() -> Connection {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(
        &mut banco,
        &origem(),
        "cenario-a",
        &ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: Some("Anotação temporária".into()),
            destinos: vec![Destino {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Unidade de testes".into(),
            }],
        },
    )
    .unwrap();
    banco
}

const ENDERECO: &str = "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10";

#[test]
fn previa_e_confirmacao_reconciliam_o_mesmo_estado_sqlite() {
    let mut banco = preparar();
    let antes = ler(&banco, &origem(), "cenario-a", "10").unwrap().unwrap();
    let previa = responder("GET", ENDERECO, "", &banco, &origem(), "cenario-a");
    assert_eq!(previa.estado, Estado::Ok);
    assert!(previa.corpo.contains("frmAtividadeListar"));
    assert!(previa.corpo.contains("chkSinRemoverAnotacoes"));
    assert_eq!(
        ler(&banco, &origem(), "cenario-a", "10").unwrap().unwrap(),
        antes
    );

    let envio = responder(
        "POST",
        ENDERECO,
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO&chkSinManterAberto=on&chkSinRemoverAnotacoes=on",
        &banco,
        &origem(),
        "cenario-a",
    );
    assert_eq!(envio.estado, Estado::Redirecionado);
    assert_eq!(
        envio.localizacao.as_deref(),
        Some(
            "controlador.php?acao=arvore_visualizar&acao_origem=procedimento_enviar&id_procedimento=10"
        )
    );
    let depois = ler(&banco, &origem(), "cenario-a", "10").unwrap().unwrap();
    assert_eq!(depois.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some(""));
    assert_eq!(depois.destinos, antes.destinos);
}

#[test]
fn envio_sem_manter_aberto_fecha_origem_e_preserva_anotacao() {
    let mut banco = preparar();
    let r = responder(
        "POST",
        ENDERECO,
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO",
        &banco,
        &origem(),
        "cenario-a",
    );
    assert_eq!(r.estado, Estado::Redirecionado);
    let depois = ler(&banco, &origem(), "cenario-a", "10").unwrap().unwrap();
    assert_eq!(depois.aberto_em, ["DESTINO"]);
    assert_eq!(
        depois.anotacao_origem.as_deref(),
        Some("Anotação temporária")
    );
}

#[test]
fn destino_invalido_e_duplicado_nao_modificam_o_banco() {
    let mut banco = preparar();
    let antes = ler(&banco, &origem(), "cenario-a", "10").unwrap();
    for corpo in [
        "hdnIdProtocolos=10&hdnUnidades=999%B1FALSO",
        "hdnIdProtocolos=99&hdnUnidades=2%B1DESTINO",
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO&hdnUnidades=2%B1DESTINO",
    ] {
        let r = responder("POST", ENDERECO, corpo, &banco, &origem(), "cenario-a");
        assert_eq!(r.estado, Estado::RequisicaoInvalida, "{corpo}");
        assert_eq!(ler(&banco, &origem(), "cenario-a", "10").unwrap(), antes);
    }
}

#[test]
fn mesmo_id_em_outro_cenario_nao_acessa_ou_muta_estado() {
    let mut banco = preparar();
    let r = responder(
        "POST",
        ENDERECO,
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO",
        &banco,
        &origem(),
        "outro-cenario",
    );
    assert_eq!(r.estado, Estado::DadosInsuficientes);
    assert_eq!(
        ler(&banco, &origem(), "cenario-a", "10")
            .unwrap()
            .unwrap()
            .aberto_em,
        ["ORIGEM"]
    );
}

#[test]
fn bdd_333_03_http_loopback_persiste_envio_e_releitura_independente() {
    use anti_ies::atender_http;
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};

    fn requisitar(endereco: std::net::SocketAddr, metodo: &str, alvo: &str, corpo: &str) -> String {
        let mut cliente = TcpStream::connect(endereco).unwrap();
        write!(
            cliente,
            "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{corpo}",
            corpo.len()
        )
        .unwrap();
        cliente.shutdown(Shutdown::Write).unwrap();
        let mut resposta = String::new();
        cliente.read_to_string(&mut resposta).unwrap();
        resposta
    }

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let banco = preparar();
    let servidor = std::thread::spawn(move || {
        for _ in 0..3 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            atender_http(&mut fluxo, Some((&banco, &origem(), "cenario-a"))).unwrap();
        }
        ler(&banco, &origem(), "cenario-a", "10").unwrap().unwrap()
    });
    let anotacao = "/sei/controlador.php?acao=anotacao_registrar&id_procedimento=10";
    let primeira = requisitar(endereco, "GET", anotacao, "");
    assert!(primeira.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(primeira.contains("Anotação temporária"));

    let envio = requisitar(
        endereco,
        "POST",
        ENDERECO,
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO&chkSinRemoverAnotacoes=on",
    );
    assert!(envio.starts_with("HTTP/1.1 302 Found\r\n"));
    assert!(envio.contains("Location: controlador.php?acao=arvore_visualizar"));

    let segunda = requisitar(endereco, "GET", anotacao, "");
    assert!(segunda.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(!segunda.contains("Anotação temporária"));
    assert!(segunda.contains("txaDescricao"));
    let depois = servidor.join().unwrap();
    assert_eq!(depois.aberto_em, ["DESTINO"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some(""));
}
