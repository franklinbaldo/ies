//! #88 BDD-88-01/03: a arvore e as versoes consultam o mesmo documento do SQLite.
use anti_ies::{Estado, documento_arvore, documento_versoes};
use iesei::banco_cassettes::iniciar;
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::num::NonZeroU32;

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn documento() -> DocumentoSimulado {
    DocumentoSimulado {
        instalacao: origem(),
        id_documento: "42".into(),
        id_procedimento: "7".into(),
        numero_processo: "00001.000001/2026-01".into(),
        numero: "0011519".into(),
        tipo: "Ofício".into(),
        revisoes: vec![
            Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "Ana".into(),
                unidade: "PPI".into(),
                modificacao: "01/11/2026".into(),
                html_corpo: "<p>Primeira redação</p>".into(),
            },
            Revisao {
                versao: NonZeroU32::new(3).unwrap(),
                usuario: "Bia".into(),
                unidade: "PPI".into(),
                modificacao: "03/11/2026".into(),
                html_corpo: "<p>Terceira redação</p>".into(),
            },
        ],
    }
}

#[test]
fn arvore_publica_links_que_abrem_historico_e_conteudo_do_mesmo_documento() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(&mut banco, "ensaio", &documento()).unwrap();

    let arvore = documento_arvore::responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(arvore.estado, Estado::Ok);
    assert!(arvore.corpo.contains("Nos[0] = new infraArvoreNo"));
    assert!(arvore.corpo.contains("Nos[1] = new infraArvoreNo"));
    assert!(arvore.corpo.contains("documento_versao_listar"));
    assert!(arvore.corpo.contains("acao=documento_visualizar"));

    let tela = documento_arvore::responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_trabalhar&id_protocolo=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(tela.estado, Estado::Ok);
    assert!(tela.corpo.contains("id=\"ifrArvore\""));
    assert!(tela.corpo.contains("id_procedimento=7"));
    let historico = documento_versoes::responder(
        "GET",
        "/sei/controlador.php?acao=documento_versao_listar&id_documento=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(historico.estado, Estado::Ok);
    assert!(historico.corpo.contains("Lista de Versões (2 registros)"));
    assert!(historico.corpo.contains(&format!("infra_hash={}", "4".repeat(64))));
    let antiga = documento_versoes::responder(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=1",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(antiga.estado, Estado::Ok);
    assert!(antiga.corpo.contains("Primeira redação"));
    assert!(!antiga.corpo.contains("Terceira redação"));

    let outro = documento_arvore::responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=99&id_documento=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(outro.estado, Estado::NaoEncontrado);
}

#[test]
fn http_procedimento_repetido_recusa_sem_vazar_documento() {
    use std::io::{Read, Write};
    use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

    fn pedir(endereco: SocketAddr, metodo: &str, alvo: &str) -> String {
        let mut tcp = TcpStream::connect(endereco).unwrap();
        write!(
            tcp,
            "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        tcp.shutdown(Shutdown::Write).unwrap();
        let mut resposta = String::new();
        tcp.read_to_string(&mut resposta).unwrap();
        resposta
    }

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = listener.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        guardar(&mut banco, "ensaio", &documento()).unwrap();
        for _ in 0..5 {
            let (mut tcp, _) = listener.accept().unwrap();
            anti_ies::atender_http(&mut tcp, Some((&banco, &origem(), "ensaio"))).unwrap();
        }
    });
    for alvo in [
        "/sei/controlador.php?acao=procedimento_visualizar&id_documento=42&id_procedimento=7&id_procedimento=8",
        "/sei/controlador.php?acao=procedimento_visualizar&id_documento=42&id_procedimento=7&id_procedimento=7",
        "/sei/controlador.php?acao=procedimento_trabalhar&id_documento=42&id_procedimento=7&id_procedimento=8",
    ] {
        let resposta = pedir(endereco, "GET", alvo);
        assert!(resposta.starts_with("HTTP/1.1 400"), "{resposta}");
        assert!(!resposta.contains("0011519"), "{resposta}");
    }
    let valido =
        "/sei/controlador.php?acao=procedimento_visualizar&id_documento=42&id_procedimento=7";
    assert!(pedir(endereco, "POST", valido).starts_with("HTTP/1.1 405"));
    let resposta = pedir(endereco, "GET", valido);
    assert!(resposta.starts_with("HTTP/1.1 200"), "{resposta}");
    assert!(resposta.contains("0011519"));
    servidor.join().unwrap();
}
