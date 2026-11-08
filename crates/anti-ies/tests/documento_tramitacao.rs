//! Integração HTTP: criar um documento, enviar o processo e recusar nova criação.
//! BDD-246-01/02/03 com estado SQLite compartilhado, ainda sintético.
use anti_ies::atender_http;
use iesei::banco_cassettes::iniciar;
use iesei::documentos_simulados::{registrar_tipo, Tipo};
use iesei::instalacao::Instalacao;
use iesei::processos::{guardar, Destino, ProcessoSimulado};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

fn servidor(pedidos: usize) -> (SocketAddr, std::thread::JoinHandle<()>) {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let tarefa = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        guardar(&mut banco, &origem, "bdd-246", &ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![Destino {
                id: "20".into(),
                sigla: "DESTINO".into(),
                descricao: "Unidade de destino".into(),
            }],
        }).unwrap();
        registrar_tipo(&banco, &origem, "bdd-246", "10", &Tipo {
            id: "291".into(), nome: "Despacho".into()
        }).unwrap();
        for _ in 0..pedidos {
            let (mut fluxo, _) = escuta.accept().unwrap();
            atender_http(&mut fluxo, Some((&banco, &origem, "bdd-246"))).unwrap();
        }
        assert_eq!(
            iesei::documentos_simulados::documentos(&banco, &origem, "bdd-246", "10")
                .unwrap().len(), 1,
            "a segunda escrita não pode ocorrer após a tramitação"
        );
    });
    (endereco, tarefa)
}

fn pedir(endereco: SocketAddr, metodo: &str, alvo: &str, corpo: &str) -> String {
    let mut fluxo = TcpStream::connect(endereco).unwrap();
    write!(
        fluxo,
        "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{corpo}",
        corpo.len()
    ).unwrap();
    fluxo.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    fluxo.read_to_string(&mut resposta).unwrap();
    resposta
}

#[test]
fn documento_criar_perde_oferta_apos_enviar_sem_manter_aberto() {
    let (endereco, tarefa) = servidor(7);
    let arvore = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
    let cadastro = "/sei/controlador.php?acao=documento_gerar&id_procedimento=10&id_serie=291";
    let escrever = "hdnIdSerie=291&hdnFlagDocumentoCadastro=2&rdoNivelAcesso=0&txtDescricao=Novo";

    let inicial = pedir(endereco, "GET", arvore, "");
    assert!(inicial.starts_with("HTTP/1.1 200 OK"));
    assert!(inicial.contains("acao=documento_escolher_tipo"), "a CLI precisa encontrar Incluir Documento");

    let criado = pedir(endereco, "POST", cadastro, escrever);
    assert!(criado.starts_with("HTTP/1.1 302 Found"));
    let antes = pedir(endereco, "GET", arvore, "");
    assert!(antes.contains("Despacho 1000001"));

    let enviado = pedir(
        endereco, "POST",
        "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10",
        "hdnIdProtocolos=10&hdnUnidades=20%B1DESTINO",
    );
    assert!(enviado.starts_with("HTTP/1.1 302 Found"));
    let depois = pedir(endereco, "GET", arvore, "");
    assert!(depois.contains("DESTINO"));
    assert!(!depois.contains("acao=documento_escolher_tipo"), "processo não está mais aberto em TESTE");

    let recusado = pedir(endereco, "POST", cadastro, escrever);
    assert!(recusado.starts_with("HTTP/1.1 400 Bad Request"), "não pode criar fora da unidade: {recusado}");
    let final_arvore = pedir(endereco, "GET", arvore, "");
    assert!(final_arvore.contains("Despacho 1000001"));
    assert!(!final_arvore.contains("Despacho 1000002"));
    tarefa.join().unwrap();
}
