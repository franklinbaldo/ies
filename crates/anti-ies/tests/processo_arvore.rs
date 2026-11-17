//! Releitura de processo após envio: o anti-IES deriva a árvore do SQLite.
//! Hipótese sintética de BDD-333-02/04, sem equivalência com SEI externo.
use anti_ies::{Estado, atender_http};
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use iesei::processos::{Destino, ProcessoSimulado, guardar, ler};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn banco() -> Connection {
    let mut db = Connection::open_in_memory().unwrap();
    iniciar(&db).unwrap();
    guardar(
        &mut db,
        &origem(),
        "bdd-333",
        &ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: Some("Observação".into()),
            destinos: vec![Destino {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Teste".into(),
            }],
        },
    )
    .unwrap();
    db
}

fn pedir(endereco: SocketAddr, metodo: &str, caminho: &str, corpo: &str) -> String {
    let mut cliente = TcpStream::connect(endereco).unwrap();
    write!(
        cliente,
        "{metodo} {caminho} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{corpo}",
        corpo.len()
    )
    .unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    resposta
}

#[test]
fn bdd_333_04_arvore_relida_mostra_unidades_e_anotacao_depois_do_post_unico() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let db = banco();
    let servidor = std::thread::spawn(move || {
        for _ in 0..4 {
            let (mut conexao, _) = escuta.accept().unwrap();
            atender_http(&mut conexao, Some((&db, &origem(), "bdd-333"))).unwrap();
        }
        ler(&db, &origem(), "bdd-333", "10").unwrap().unwrap()
    });
    let arvore = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
    let antes = pedir(endereco, "GET", arvore, "");
    assert!(antes.starts_with("HTTP/1.1 200 OK\r\n"), "{antes}");
    assert!(antes.contains("Nos[0].html ="));
    assert!(antes.contains("ORIGEM"));
    assert!(!antes.contains("DESTINO"));

    let envio = pedir(
        endereco,
        "POST",
        "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10",
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO&chkSinManterAberto=on&chkSinRemoverAnotacoes=on",
    );
    assert!(envio.starts_with("HTTP/1.1 302 Found\r\n"), "{envio}");

    let depois = pedir(endereco, "GET", arvore, "");
    assert!(depois.starts_with("HTTP/1.1 200 OK\r\n"), "{depois}");
    assert!(depois.contains("Processo aberto nas unidades:"));
    assert!(depois.contains("ORIGEM"));
    assert!(depois.contains("DESTINO"));

    let anotacao = pedir(
        endereco,
        "GET",
        "/sei/controlador.php?acao=anotacao_registrar&id_procedimento=10",
        "",
    );
    assert!(anotacao.contains("txaDescricao"));
    assert!(!anotacao.contains("Observação"));
    let estado = servidor.join().unwrap();
    assert_eq!(estado.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(estado.anotacao_origem.as_deref(), Some(""));
}

#[test]
fn processar_navegacao_nao_muta_estado_e_outro_cenario_nao_vaza_dados() {
    let db = banco();
    let antes = ler(&db, &origem(), "bdd-333", "10").unwrap();
    let rota = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
    let navegacao = anti_ies::processo_envio::navegacao("GET", rota, &db, &origem(), "bdd-333");
    assert_eq!(navegacao.unwrap().estado, Estado::Ok);
    assert_eq!(ler(&db, &origem(), "bdd-333", "10").unwrap(), antes);
    let desconhecido = anti_ies::processo_envio::navegacao("GET", rota, &db, &origem(), "outro");
    assert!(desconhecido.is_none());
}

/// O cadastro de documentos e as transições leem a mesma identidade canônica.
#[test]
fn identidade_do_processo_compartilhada_entre_documentos_e_tramitacao() {
    use iesei::documentos_simulados::{Processo, ler_processo, registrar_processo};
    let db = banco();
    let base = ler_processo(&db, &origem(), "bdd-333", "10")
        .unwrap()
        .expect("cadastro único ao guardar a trilha de envio");
    assert_eq!(base.nup, "00001.000001/2026-01");
    assert_eq!(base.unidade, "ORIGEM");

    registrar_processo(
        &db,
        &origem(),
        "bdd-333",
        &Processo {
            id: "10".into(),
            nup: "00001.000002/2026-01".into(),
            unidade: "ORIGEM".into(),
        },
    )
    .unwrap();
    let processo = ler(&db, &origem(), "bdd-333", "10").unwrap().unwrap();
    assert_eq!(processo.numero, "00001.000002/2026-01");
    assert_eq!(processo.aberto_em, ["ORIGEM"]);
}

/// Prova técnica do BDD-333-04: a transição não oculta documentos internos
/// nem o histórico do mesmo processo e banco simulado.
#[test]
fn envio_preserva_documentos_internos_e_versoes_na_arvore() {
    use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar as guardar_versao};
    use iesei::documentos_simulados::{Tipo, criar, documentos, registrar_tipo};
    use std::num::NonZeroU32;

    let mut db = banco();
    registrar_tipo(
        &db,
        &origem(),
        "bdd-333",
        "10",
        &Tipo {
            id: "291".into(),
            nome: "Despacho".into(),
        },
    )
    .unwrap();
    let interno = criar(&db, &origem(), "bdd-333", "10", "291", "Interno").unwrap();
    guardar_versao(
        &mut db,
        "bdd-333",
        &DocumentoSimulado {
            instalacao: origem(),
            id_documento: "42".into(),
            id_procedimento: "10".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: "0011519".into(),
            tipo: "Ofício".into(),
            revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "Teste".into(),
                unidade: "ORIGEM".into(),
                modificacao: "08/11/2026".into(),
                html_corpo: "<p>Versão preservada</p>".into(),
            }],
        },
    )
    .unwrap();
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        for _ in 0..5 {
            let (mut conexao, _) = escuta.accept().unwrap();
            atender_http(&mut conexao, Some((&db, &origem(), "bdd-333"))).unwrap();
        }
        assert_eq!(
            documentos(&db, &origem(), "bdd-333", "10").unwrap(),
            [interno]
        );
        ler(&db, &origem(), "bdd-333", "10").unwrap().unwrap()
    });
    let raiz = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
    let antes = pedir(endereco, "GET", raiz, "");
    assert!(antes.contains("Nos[1] = new infraArvoreNo"), "{antes}");
    assert!(antes.contains("Nos[2] = new infraArvoreNo"), "{antes}");
    assert!(antes.contains("documento_versao_listar"));
    let envio = pedir(
        endereco,
        "POST",
        "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10",
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO&chkSinManterAberto=on&chkSinRemoverAnotacoes=on",
    );
    assert!(envio.starts_with("HTTP/1.1 302 Found"), "{envio}");
    let depois = pedir(endereco, "GET", raiz, "");
    assert!(depois.contains("Nos[1] = new infraArvoreNo"), "{depois}");
    assert!(depois.contains("Nos[2] = new infraArvoreNo"), "{depois}");
    assert!(depois.contains("DESTINO"));
    assert!(depois.contains("documento_versao_listar"));
    let arvore_documento = pedir(
        endereco,
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=42",
        "",
    );
    assert!(arvore_documento.contains("Nos[1] = new infraArvoreNo"));
    let historico = pedir(
        endereco,
        "GET",
        "/sei/controlador.php?acao=documento_versao_listar&id_documento=42",
        "",
    );
    assert!(
        historico.contains("Lista de Versões (1 registros)"),
        "{historico}"
    );
    let processo = servidor.join().unwrap();
    assert_eq!(processo.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(processo.anotacao_origem.as_deref(), Some(""));
}

#[test]
fn selecao_documental_repetida_na_arvore_de_tramitacao_recusa_sem_apagar_estado() {
    use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar as guardar_versao};
    use std::num::NonZeroU32;

    let mut db = banco();
    guardar_versao(
        &mut db,
        "bdd-333",
        &DocumentoSimulado {
            instalacao: origem(),
            id_documento: "42".into(),
            id_procedimento: "10".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: "0011519".into(),
            tipo: "Despacho".into(),
            revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "TESTE".into(),
                unidade: "ORIGEM".into(),
                modificacao: "08/11/2026".into(),
                html_corpo: "<p>Conteúdo do documento legítimo</p>".into(),
            }],
        },
    )
    .unwrap();
    let antes = ler(&db, &origem(), "bdd-333", "10").unwrap();
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        for _ in 0..6 {
            let (mut fluxo, cliente) = escuta.accept().unwrap();
            assert!(cliente.ip().is_loopback());
            atender_http(&mut fluxo, Some((&db, &origem(), "bdd-333"))).unwrap();
        }
        ler(&db, &origem(), "bdd-333", "10").unwrap()
    });
    let raiz = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
    let valido = pedir(endereco, "GET", raiz, "");
    assert!(valido.starts_with("HTTP/1.1 200 OK\r\n"), "{valido}");
    assert!(valido.contains("0011519"), "documento presente no SQLite");

    // Sem PDF no processo, a rota de tramitacao nao deve ignorar a
    // duplicacao e servir a arvore como se nenhuma selecao existisse.
    let ambiguo = pedir(
        endereco,
        "GET",
        &format!("{raiz}&id_documento=42&id_documento=42"),
        "",
    );
    assert!(
        ambiguo.starts_with("HTTP/1.1 400 Bad Request\r\n"),
        "{ambiguo}"
    );
    assert!(!ambiguo.contains("0011519"), "recusa nao publica documento");

    // A tela de trabalho tem a mesma fronteira de identidade da arvore.
    let trabalhar = pedir(
        endereco,
        "GET",
        "/sei/controlador.php?acao=procedimento_trabalhar&id_procedimento=10&id_documento=42&id_documento=42",
        "",
    );
    assert!(
        trabalhar.starts_with("HTTP/1.1 400 Bad Request\r\n"),
        "{trabalhar}"
    );

    let vazio = pedir(endereco, "GET", &format!("{raiz}&id_documento="), "");
    assert!(vazio.starts_with("HTTP/1.1 200 OK\r\n"), "{vazio}");
    assert!(
        vazio.contains("0011519"),
        "selecao vazia unica abre a arvore"
    );

    let post = pedir(endereco, "POST", raiz, "");
    assert!(
        post.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"),
        "{post}"
    );

    let depois = pedir(endereco, "GET", raiz, "");
    assert!(depois.starts_with("HTTP/1.1 200 OK\r\n"), "{depois}");
    assert!(depois.contains("0011519"));
    assert_eq!(
        servidor.join().unwrap(),
        antes,
        "leitura e recusa sao imutaveis"
    );
}
