//! #271 BDD-271-02: o mesmo usuário não pode registrar segunda ciência.
//! Resposta de recusa observada no oráculo Todos: validação Infra HTTP 200.
//! Teste local; não representa paridade de sessão ou ensaio no SEI externo.
use anti_ies::atender_http;
use iesei::banco_cassettes::iniciar;
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::num::NonZeroU32;

fn pedir(endereco: SocketAddr, alvo: &str) -> String {
    let mut tcp = TcpStream::connect(endereco).unwrap();
    write!(
        tcp,
        "GET {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    tcp.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    tcp.read_to_string(&mut resposta).unwrap();
    resposta
}

#[test]
fn segunda_ciencia_devolve_validacao_infra_200_sem_nova_atividade() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = listener.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        guardar(
            &mut banco,
            "bdd-271",
            &DocumentoSimulado {
                instalacao: origem.clone(),
                id_documento: "42".into(),
                id_procedimento: "7".into(),
                numero_processo: "00001.000001/2026-01".into(),
                numero: "0011519".into(),
                tipo: "Despacho".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "Autora da revisão".into(),
                    unidade: "TESTE".into(),
                    modificacao: "09/11/2026".into(),
                    html_corpo: "<p>Ensaio local</p>".into(),
                }],
            },
        )
        .unwrap();
        for _ in 0..3 {
            let (mut tcp, _) = listener.accept().unwrap();
            atender_http(&mut tcp, Some((&banco, &origem, "bdd-271"))).unwrap();
        }
        let quantidade: i64 = banco
            .query_row("SELECT COUNT(*) FROM ciencia_documental", [], |r| r.get(0))
            .unwrap();
        assert_eq!(quantidade, 1, "segunda requisição não cria atividade");
    });

    let acao = "/sei/controlador.php?acao=documento_ciencia&id_documento=42";
    let aceite = pedir(endereco, acao);
    assert!(aceite.starts_with("HTTP/1.1 200 OK"), "{aceite}");
    assert!(aceite.contains("id_atividade=42457"), "{aceite}");
    let segunda = pedir(endereco, acao);
    assert!(segunda.starts_with("HTTP/1.1 200 OK"), "{segunda}");
    assert!(segunda.contains("id='txaInfraValidacao'"), "{segunda}");
    assert!(
        segunda.contains("Usuário já deu ciência neste documento."),
        "{segunda}"
    );
    assert!(!segunda.contains("atualizarArvore"), "{segunda}");
    let lista = pedir(
        endereco,
        "/sei/controlador.php?acao=protocolo_ciencia_listar&id_procedimento=7&id_documento=42",
    );
    assert!(lista.starts_with("HTTP/1.1 200 OK"), "{lista}");
    assert!(lista.contains("value='42457'"), "{lista}");
    servidor.join().unwrap();
}
