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

fn pedir_rota(endereco: std::net::SocketAddr, metodo: &str, acao: &str, caminho: &str) -> String {
    let mut tcp = TcpStream::connect(endereco).unwrap();
    write!(
        tcp,
        "{metodo} /sei/controlador.php?acao={acao}&{caminho} HTTP/1.1\r\nHost: sei.test\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    tcp.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    tcp.read_to_string(&mut resposta).unwrap();
    resposta
}

fn pedir(endereco: std::net::SocketAddr, metodo: &str, caminho: &str) -> String {
    pedir_rota(endereco, metodo, "procedimento_visualizar", caminho)
}

#[test]
fn http_documento_duplicado_nao_escolhe_id_protocolo_alternativo() {
    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let atendimento = thread::spawn(move || {
        let (banco, origem) = estado();
        for _ in 0..6 {
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

    // BDD-60-05: dois identificadores diferentes, cada qual selecionavel
    // isoladamente, nao podem escolher silenciosamente o primeiro.
    let conflitante = pedir(endereco, "GET", "id_documento=42&id_protocolo=43");
    assert!(
        conflitante.starts_with("HTTP/1.1 400 Bad Request\r\n"),
        "{conflitante}"
    );
    assert!(!conflitante.contains("0011519"));
    assert!(!conflitante.contains("0011520"));

    // Dois campos concordantes podem identificar o mesmo documento.
    let concordante = pedir(endereco, "GET", "id_documento=42&id_protocolo=42");
    assert!(
        concordante.starts_with("HTTP/1.1 200 OK\r\n"),
        "{concordante}"
    );
    assert!(concordante.contains("0011519"));
    assert!(!concordante.contains("0011520"));

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

fn verificar_selecao_mista_de_protocolo(endereco: std::net::SocketAddr) {
    // BDD-60-05: com PDF cadastrado, id_documento e id_protocolo
    // conflitantes não autorizam devolver a árvore nem conteúdo alternativo.
    let conflitante = pedir(
        endereco,
        "GET",
        "id_procedimento=7&id_documento=8000000000001&id_protocolo=42",
    );
    assert!(
        conflitante.starts_with("HTTP/1.1 400 Bad Request\r\n"),
        "{conflitante}"
    );
    assert!(
        !conflitante.contains("050000001"),
        "uma seleção conflitante não publica o anexo"
    );

    // Dois campos concordantes continuam aceitos, como na árvore individual.
    let concordante = pedir(
        endereco,
        "GET",
        "id_procedimento=7&id_documento=8000000000001&id_protocolo=8000000000001",
    );
    assert!(
        concordante.starts_with("HTTP/1.1 200 OK\r\n"),
        "{concordante}"
    );
    assert!(concordante.contains("050000001"));
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
        for _ in 0..7 {
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

    verificar_selecao_mista_de_protocolo(endereco);

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

#[test]
fn http_url_observada_antes_do_cancelamento_nao_serve_corpo_depois() {
    use iesei::cancelamento_documento::{self, Aplicacao, Situacao};

    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let atendimento = thread::spawn(move || {
        let (banco, origem) = estado();
        for indice in 0..4 {
            let (mut tcp, cliente) = servidor.accept().unwrap();
            assert!(cliente.ip().is_loopback());
            atender_http(&mut tcp, Some((&banco, &origem, CENARIO))).unwrap();
            if indice == 0 {
                // A URL foi observada antes do cancelamento: testa-se o replay
                // da mesma URL depois de uma transicao real no SQLite.
                assert_eq!(
                    cancelamento_documento::aplicar(
                        &banco,
                        &origem,
                        CENARIO,
                        "42",
                        "7",
                        "Cancelamento local"
                    )
                    .unwrap(),
                    Aplicacao::Aplicada
                );
            }
        }
        assert_eq!(
            cancelamento_documento::consultar(&banco, &origem, CENARIO, "42", "7").unwrap(),
            Some(Situacao::Cancelado {
                motivo: "Cancelamento local".into()
            })
        );
    });

    let alvo = "id_documento=42&id_procedimento=7";
    let antes = pedir_rota(endereco, "GET", "documento_visualizar", alvo);
    assert!(antes.starts_with("HTTP/1.1 200 OK\r\n"), "{antes}");
    assert!(antes.contains("Corpo do primeiro"));

    // A fonte de um link ja observado nao pode publicar novamente o corpo
    // cancelado; omitir src na arvore, sozinho, nao fecha essa transicao.
    let depois = pedir_rota(endereco, "GET", "documento_visualizar", alvo);
    assert!(depois.starts_with("HTTP/1.1 403 Forbidden\r\n"), "{depois}");
    assert!(!depois.contains("Corpo do primeiro"));

    let outro = pedir_rota(
        endereco,
        "GET",
        "documento_visualizar",
        "id_documento=43&id_procedimento=7",
    );
    assert!(outro.starts_with("HTTP/1.1 200 OK\r\n"), "{outro}");
    assert!(outro.contains("Corpo do segundo"));

    let metodo = pedir_rota(endereco, "POST", "documento_visualizar", alvo);
    assert!(
        metodo.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"),
        "{metodo}"
    );
    atendimento.join().unwrap();
}

#[test]
fn http_historico_e_conteudo_recusam_identidade_de_protocolo_conflitante() {
    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let atendimento = thread::spawn(move || {
        let (banco, origem) = estado();
        for _ in 0..7 {
            let (mut tcp, cliente) = servidor.accept().unwrap();
            assert!(cliente.ip().is_loopback());
            atender_http(&mut tcp, Some((&banco, &origem, CENARIO))).unwrap();
        }
        // A recusa de leitura nao modifica o mesmo estado SQLite.
        assert_eq!(
            documento_versoes::ler(&banco, &origem, CENARIO, "42")
                .unwrap()
                .unwrap()
                .numero,
            "0011519"
        );
    });

    // Os dois identificadores existem, mas designam documentos diferentes.
    // Nenhuma URL com identidade contraditoria pode servir o corpo ou
    // publicar links de versoes do primeiro como fallback.
    for acao in ["documento_versao_listar", "documento_visualizar"] {
        let resposta = pedir_rota(
            endereco,
            "GET",
            acao,
            "id_documento=42&id_protocolo=43&id_procedimento=7",
        );
        assert!(
            resposta.starts_with("HTTP/1.1 400 Bad Request\r\n"),
            "{acao}: {resposta}"
        );
        assert!(!resposta.contains("Corpo do primeiro"));
        assert!(!resposta.contains("Lista de Versões"));
    }

    let historico = pedir_rota(
        endereco,
        "GET",
        "documento_versao_listar",
        "id_documento=42&id_procedimento=7",
    );
    assert!(historico.starts_with("HTTP/1.1 200 OK\r\n"), "{historico}");
    assert!(historico.contains("Lista de Versões"));

    let conteudo = pedir_rota(
        endereco,
        "GET",
        "documento_visualizar",
        "id_documento=42&id_procedimento=7",
    );
    assert!(conteudo.starts_with("HTTP/1.1 200 OK\r\n"), "{conteudo}");
    assert!(conteudo.contains("Corpo do primeiro"));
    assert!(!conteudo.contains("Corpo do segundo"));

    let metodo = pedir_rota(
        endereco,
        "POST",
        "documento_versao_listar",
        "id_documento=42&id_procedimento=7",
    );
    assert!(
        metodo.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"),
        "{metodo}"
    );

    let repetido = pedir_rota(
        endereco,
        "GET",
        "documento_visualizar",
        "id_documento=42&id_procedimento=7&id_documento=42",
    );
    assert!(repetido.starts_with("HTTP/1.1 200 OK\r\n"), "{repetido}");
    assert!(repetido.contains("Corpo do primeiro"));

    let concordante = pedir_rota(
        endereco,
        "GET",
        "documento_visualizar",
        "id_documento=42&id_protocolo=42&id_procedimento=7",
    );
    assert!(
        concordante.starts_with("HTTP/1.1 200 OK\r\n"),
        "{concordante}"
    );
    assert!(concordante.contains("Corpo do primeiro"));
    atendimento.join().unwrap();
}

#[path = "aies_navegacao_identidade_http/parametros_repetidos.rs"]
mod parametros_repetidos;

#[path = "aies_navegacao_identidade_http/trabalhar_ambiguidade.rs"]
mod trabalhar_ambiguidade;

#[path = "aies_navegacao_identidade_http/trabalhar_protocolo_repetido.rs"]
mod trabalhar_protocolo_repetido;
