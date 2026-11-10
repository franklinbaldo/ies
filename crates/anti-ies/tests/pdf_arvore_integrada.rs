//! #332: árvore pública deve incluir documentos internos e PDFs definitivos.
use iesei::banco_cassettes::iniciar;
use iesei::documentos_simulados::{self as interno, Tipo};
use iesei::instalacao::Instalacao;
use iesei::pdf_externo::{self as externo, Processo, TipoExterno};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

// Evita bloqueio quando uma asserção impede o segundo GET do teste.
fn aceitar(escuta: &TcpListener) -> TcpStream {
    escuta.set_nonblocking(true).unwrap();
    let prazo = Instant::now() + Duration::from_secs(5);
    loop {
        match escuta.accept() {
            Ok((fluxo, _)) => {
                fluxo
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                fluxo
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return fluxo;
            }
            Err(erro)
                if erro.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < prazo =>
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(erro) => panic!("servidor HTTP sintético não recebeu GET: {erro}"),
        }
    }
}

fn pedir_metodo(endereco: SocketAddr, metodo: &str, alvo: &str) -> String {
    let mut fluxo = TcpStream::connect_timeout(&endereco, Duration::from_secs(5)).unwrap();
    fluxo
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    fluxo
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(fluxo, "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
    fluxo.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    fluxo.read_to_string(&mut resposta).unwrap();
    resposta
}

fn pedir(endereco: SocketAddr, alvo: &str) -> String {
    pedir_metodo(endereco, "GET", alvo)
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

        {
            let mut conexao = aceitar(&escuta);
            anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "caso"))).unwrap();
        }
        let temporario = externo::receber_conteudo(
            &banco,
            &origem,
            "caso",
            "7",
            "prova.pdf",
            b"%PDF-1.7\nconteudo",
        )
        .unwrap();
        externo::cadastrar(&banco, &origem, "caso", "7", "293", temporario.id).unwrap();
        let mut conexao = aceitar(&escuta);
        anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "caso"))).unwrap();
    });
    let alvo = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento=";
    let antes = pedir(endereco, alvo);
    // Ambas as requisições ocorrem antes de validar o primeiro resultado.
    let depois = pedir(endereco, alvo);
    servidor.join().unwrap();
    assert!(antes.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(antes.contains("Nos[1] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(antes.contains("Despacho"));
    assert!(!antes.contains("050000001"));
    assert!(depois.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(depois.contains("Despacho"));
    assert!(depois.contains("Nos[2] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(depois.contains("Anexo (050000001)"));
    // A árvore do PDF usa o mesmo token emulado da barra e do Controle R.
    assert!(depois.contains(&format!("infra_hash={}", "4".repeat(64))));
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
        let temporario = externo::receber_conteudo(
            &banco,
            &origem,
            "selecao",
            "7",
            "prova.pdf",
            b"%PDF-1.7\nconteudo",
        )
        .unwrap();
        externo::cadastrar(&banco, &origem, "selecao", "7", "293", temporario.id).unwrap();
        let mut conexao = aceitar(&escuta);
        anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "selecao"))).unwrap();
    });
    let resposta = pedir(
        endereco,
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento=8000000000001",
    );
    assert!(resposta.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(resposta.contains("Nos[1] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(resposta.contains("Anexo (050000001)"));
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
        let temporario = externo::receber_conteudo(
            &banco,
            &origem,
            "triplo",
            "7",
            "anexo.pdf",
            b"%PDF-1.7\nconteudo",
        )
        .unwrap();
        externo::cadastrar(&banco, &origem, "triplo", "7", "293", temporario.id).unwrap();
        let mut conexao = aceitar(&escuta);
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
    assert!(resposta.contains("Anexo (050000001)"));
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
    let temp = externo::receber_conteudo(
        &banco,
        &origem,
        "barra",
        "7",
        "prova.pdf",
        b"%PDF-1.7\nconteudo",
    )
    .unwrap();
    let doc = externo::cadastrar(&banco, &origem, "barra", "7", "293", temp.id).unwrap();
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut conexao = aceitar(&escuta);
        anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "barra"))).unwrap();
    });
    let alvo = format!(
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento={}",
        doc.id_publico()
    );
    let resposta = pedir(endereco, &alvo);
    assert!(resposta.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(resposta.contains("Anexo (050000001)"));
    assert!(resposta.contains("Nos[0].acoes = '"));
    assert!(resposta.contains("procedimento_enviar"));
    assert!(resposta.contains("anotacao_registrar"));
    servidor.join().unwrap();
}

/// BDD-60-05: identidade de PDF em outra árvore conhecida não autoriza
/// fallback para o processo pedido. A recusa é hipótese sintética:
/// a cassette UnB registra navegação, não esta seleção cruzada.
#[test]
fn bdd_60_05_pdf_conhecido_em_outro_processo_recusa_sem_trocar_arvore() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
    let tipo = TipoExterno {
        id: "293".into(),
        nome: "Anexo".into(),
        limite_bytes: 100_000,
    };
    let mut registrados = Vec::new();
    for (id, nup, nome) in [
        ("7", "00001.000001/2026-01", "primeiro.pdf"),
        ("8", "00001.000002/2026-02", "segundo.pdf"),
    ] {
        externo::semear(
            &banco,
            &origem,
            "identidades",
            &Processo {
                id: id.into(),
                nup: nup.into(),
                unidade: "TESTE".into(),
            },
            &tipo,
        )
        .unwrap();
        let temporario = externo::receber_conteudo(
            &banco,
            &origem,
            "identidades",
            id,
            nome,
            b"%PDF-1.7\nconteudo",
        )
        .unwrap();
        registrados.push(
            externo::cadastrar(&banco, &origem, "identidades", id, "293", temporario.id).unwrap(),
        );
    }
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        for _ in 0..6 {
            let mut conexao = aceitar(&escuta);
            anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "identidades"))).unwrap();
        }
        for processo in ["7", "8"] {
            assert_eq!(
                externo::documentos(&banco, &origem, "identidades", processo)
                    .unwrap()
                    .len(),
                1,
                "GET e recusa não alteram o SQLite"
            );
        }
    });

    let url_primeiro = format!(
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento={}",
        registrados[0].id_publico()
    );
    let url_cruzado = format!(
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento={}",
        registrados[1].id_publico()
    );
    let url_segundo = format!(
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=8&id_documento={}",
        registrados[1].id_publico()
    );
    let primeiro = pedir(endereco, &url_primeiro);
    let cruzado = pedir(endereco, &url_cruzado);
    let segundo = pedir(endereco, &url_segundo);
    let desconhecido = pedir(
        endereco,
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento=8000000000999",
    );
    let metodo = pedir_metodo(endereco, "POST", &url_primeiro);
    let novamente = pedir(endereco, &url_primeiro);
    servidor.join().unwrap();

    assert!(primeiro.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(primeiro.contains(&registrados[0].numero_sei));
    assert!(segundo.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(segundo.contains(&registrados[1].numero_sei));
    assert!(
        cruzado.starts_with("HTTP/1.1 404 Not Found\r\n"),
        "PDF de outro processo conhecido deve recusar, sem resposta ambígua: {cruzado}"
    );
    assert!(!cruzado.contains(&registrados[1].numero_sei));
    assert!(
        desconhecido.starts_with("HTTP/1.1 503"),
        "ausência de observação de PDF não prova inexistência: {desconhecido}"
    );
    assert!(metodo.starts_with("HTTP/1.1 405"));
    assert_eq!(primeiro, novamente, "recusa não modifica a árvore original");
}
