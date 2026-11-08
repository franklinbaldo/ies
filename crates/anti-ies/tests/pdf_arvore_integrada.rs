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
            &banco,
            &origem,
            "caso",
            "7",
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

#[test]
fn bdd_332_03_selecionar_pdf_cadastrado_nao_oculta_arvore_do_processo() {
    // Apoio tecnico ao BDD: a releitura com o id do PDF selecionado
    // tambem precisa apresentar o documento externo ja cadastrado.
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        externo::semear(
            &banco,
            &origem,
            "selecao",
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
        let temporario =
            externo::receber(&banco, &origem, "selecao", "7", "prova.pdf", 42).unwrap();
        externo::cadastrar(&banco, &origem, "selecao", "7", "293", temporario.id).unwrap();
        let (mut conexao, _) = escuta.accept().unwrap();
        anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "selecao"))).unwrap();
    });
    let resposta = pedir(
        endereco,
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento=8000000000001",
    );
    assert!(resposta.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(resposta.contains("Nos[1] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(resposta.contains("Anexo 050000001"));
    servidor.join().unwrap();
}

#[test]
fn bdd_332_03_pdf_cadastrado_preserva_documento_versionado_e_interno() {
    use iesei::documento_versoes::{self as versoes, DocumentoSimulado, Revisao};
    use std::num::NonZeroU32;

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        externo::semear(
            &banco,
            &origem,
            "triplo",
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
            &banco,
            &origem,
            "triplo",
            "7",
            &Tipo {
                id: "291".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
        interno::criar(&banco, &origem, "triplo", "7", "291", "Interno").unwrap();
        versoes::guardar(
            &mut banco,
            "triplo",
            &DocumentoSimulado {
                instalacao: origem.clone(),
                id_documento: "300".into(),
                id_procedimento: "7".into(),
                numero_processo: "00001.000001/2026-01".into(),
                numero: "2000003".into(),
                tipo: "Oficio".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "AUTOR".into(),
                    unidade: "TESTE".into(),
                    modificacao: "08/11/2026".into(),
                    html_corpo: "<p>Versao inicial</p>".into(),
                }],
            },
        )
        .unwrap();
        let temporario = externo::receber(&banco, &origem, "triplo", "7", "anexo.pdf", 42).unwrap();
        externo::cadastrar(&banco, &origem, "triplo", "7", "293", temporario.id).unwrap();
        let (mut conexao, _) = escuta.accept().unwrap();
        anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "triplo"))).unwrap();
    });
    let resposta = pedir(
        endereco,
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7",
    );
    assert!(resposta.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(resposta.contains("Oficio 2000003"));
    assert!(resposta.contains("documento_versao_listar"));
    assert!(resposta.contains("Despacho 1000001"));
    assert!(resposta.contains("Anexo 050000001"));
    assert_eq!(
        resposta.matches("new infraArvoreNo(\"DOCUMENTO\"").count(),
        3
    );
    servidor.join().unwrap();
}

#[test]
fn bdd_332_03_pdf_selecionado_preserva_acoes_do_processo() {
    use iesei::processos::{self, ProcessoSimulado};

    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
    let processo = Processo {
        id: "7".into(),
        nup: "00001.000001/2026-01".into(),
        unidade: "TESTE".into(),
    };
    externo::semear(
        &banco,
        &origem,
        "barra",
        &processo,
        &TipoExterno {
            id: "293".into(),
            nome: "Anexo".into(),
            limite_bytes: 100_000,
        },
    )
    .unwrap();
    processos::guardar(
        &mut banco,
        &origem,
        "barra",
        &ProcessoSimulado {
            id: processo.id.clone(),
            numero: processo.nup.clone(),
            unidade_origem: processo.unidade.clone(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: Some("Conferir".into()),
            destinos: vec![],
        },
    )
    .unwrap();
    let temp = externo::receber(&banco, &origem, "barra", "7", "prova.pdf", 42).unwrap();
    let doc = externo::cadastrar(&banco, &origem, "barra", "7", "293", temp.id).unwrap();
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let (mut conexao, _) = escuta.accept().unwrap();
        anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "barra"))).unwrap();
    });
    let alvo = format!(
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento={}",
        doc.id_publico()
    );
    let resposta = pedir(endereco, &alvo);
    assert!(resposta.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(resposta.contains("Anexo 050000001"));
    assert!(resposta.contains("Nos[0].acoes = '"));
    assert!(resposta.contains("procedimento_enviar"));
    assert!(resposta.contains("anotacao_registrar"));
    servidor.join().unwrap();
}
