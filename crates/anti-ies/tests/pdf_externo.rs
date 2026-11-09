//! #332: POST de upload temporário, cadastro e releitura compartilham SQLite.
//! Cenários sintéticos; compatibilidade com SEI real ainda não verificada.
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use iesei::pdf_externo::{self as modelo, Processo, TipoExterno};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn semear(banco: &Connection) {
    modelo::semear(
        banco,
        &origem(),
        "ensaio",
        &Processo {
            id: "7".into(),
            nup: "00001.000001/2026-01".into(),
            unidade: "TESTE".into(),
        },
        &TipoExterno {
            id: "293".into(),
            nome: "Anexo".into(),
            limite_bytes: 200 * 1024 * 1024,
        },
    )
    .unwrap();
}

fn pedir(endereco: SocketAddr, metodo: &str, alvo: &str, corpo: &[u8]) -> String {
    let mut fluxo = TcpStream::connect(endereco).unwrap();
    write!(
        fluxo,
        "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n",
        corpo.len()
    )
    .unwrap();
    fluxo.write_all(corpo).unwrap();
    fluxo.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    fluxo.read_to_string(&mut resposta).unwrap();
    resposta
}

// Fluxo HTTP de leitura e recusas independente do BDD da inclusão.
fn conferir_download(endereco: SocketAddr, base: &str) {
    // HTTP local do arquivo cadastrado; recusas não alteram o documento.
    let download = format!("{base}documento_download_anexo&id_anexo=1");
    let original = pedir(endereco, "GET", &download, b"");
    assert!(original.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(original.contains("Content-Type: application/pdf\r\n"));
    assert!(original.ends_with("%PDF-1.7\nconteudo"));
    let metodo = pedir(endereco, "POST", &download, b"");
    assert!(metodo.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"));
    let duplicado = pedir(endereco, "GET", &format!("{download}&id_anexo=1"), b"");
    assert!(duplicado.starts_with("HTTP/1.1 400 Bad Request\r\n"));
    let ausente = pedir(
        endereco,
        "GET",
        &format!("{base}documento_download_anexo&id_anexo=999"),
        b"",
    );
    assert!(ausente.starts_with("HTTP/1.1 404 Not Found\r\n"));
    let novamente = pedir(endereco, "GET", &download, b"");
    assert!(novamente.ends_with("%PDF-1.7\nconteudo"));
}

#[test]
fn bdd_332_02_03_04_hypotese_sintetica_upload_e_cadastro_sao_estados_distintos() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        semear(&banco);
        let origem = origem();
        for _ in 0..13 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut fluxo, Some((&banco, &origem, "ensaio"))).unwrap();
        }
        modelo::documentos(&banco, &origem, "ensaio", "7").unwrap()
    });

    let base = "/sei/controlador.php?acao=";
    let lista = pedir(
        endereco,
        "GET",
        &format!("{base}documento_escolher_tipo&id_procedimento=7"),
        b"",
    );
    assert!(lista.contains("acao=documento_receber"));
    assert!(lista.contains(&format!("infra_hash={}", "4".repeat(64))));
    let cadastro = pedir(
        endereco,
        "GET",
        &format!("{base}documento_receber&id_procedimento=7"),
        b"",
    );
    assert!(cadastro.contains("id='selSerie'"));
    assert!(cadastro.contains(&format!("infra_hash={}", "4".repeat(64))));
    assert!(cadastro.contains("value='293'"));
    let recarregado = pedir(
        endereco,
        "POST",
        &format!("{base}documento_receber&id_procedimento=7"),
        b"hdnFlagDocumentoCadastro=1&selSerie=293",
    );
    assert!(recarregado.contains("selected='selected'"));
    assert!(recarregado.contains("documento_upload_anexo"));
    assert!(recarregado.contains(&format!("infra_hash={}", "4".repeat(64))));
    assert!(recarregado.contains("var arrExt = []"));

    let multipart = b"--B\r\nContent-Disposition: form-data; name=\"filArquivo\"; filename=\"prova.pdf\"\r\nContent-Type: application/pdf\r\n\r\n%PDF-1.7\nconteudo\r\n--B--\r\n";
    let recibo = pedir(
        endereco,
        "POST",
        &format!("{base}documento_upload_anexo&id_procedimento=7"),
        multipart,
    );
    assert!(recibo.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(recibo.contains("upload-1#prova.pdf#application/pdf#"));
    let antes = pedir(
        endereco,
        "GET",
        &format!("{base}arvore_visualizar&id_procedimento=7"),
        b"",
    );
    assert!(!antes.contains("documento_pdf"));
    let definitivo = pedir(
        endereco,
        "POST",
        &format!("{base}documento_receber&id_procedimento=7"),
        b"selSerie=293&hdnFlagDocumentoCadastro=2&hdnAnexos=upload-1%C2%B1prova.pdf",
    );
    assert!(definitivo.starts_with("HTTP/1.1 302 Found\r\n"));
    assert!(definitivo.contains(
        "Location: controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento=8000000000001"
    ));

    // A URL devolvida pelo cadastro deve selecionar o MESMO ID publico
    // da arvore que o leitor IES interpreta; seguir exatamente o Location.
    let location = definitivo
        .lines()
        .find_map(|linha| linha.strip_prefix("Location: "))
        .unwrap()
        .trim_end_matches('\r');
    let selecionado = pedir(endereco, "GET", &format!("/sei/{location}"), b"");
    assert!(selecionado.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(selecionado.contains("infraArvoreNo(\"DOCUMENTO\",\"8000000000001\""));
    assert!(selecionado.contains("Anexo 050000001"));

    let depois = pedir(
        endereco,
        "GET",
        &format!("{base}arvore_visualizar&id_procedimento=7"),
        b"",
    );
    assert!(depois.contains("documento_pdf"));
    // O mesmo ID publicado no Location aparece na arvore alternativa.
    assert!(depois.contains("data-id='8000000000001'"));
    assert!(depois.contains("prova.pdf"));
    assert!(depois.contains("050000001"));
    conferir_download(endereco, base);
    let documentos = servidor.join().unwrap();
    assert_eq!(documentos.len(), 1);
    assert_eq!(documentos[0].numero_sei, "050000001");
    assert_eq!(documentos[0].tipo, "Anexo");
}

#[test]
fn bdd_332_04_upload_rejeitado_nao_cadastra_e_cenario_isolado_nao_vaza() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    semear(&banco);
    let origem = origem();
    let vazio = modelo::documentos(&banco, &origem, "ensaio", "7").unwrap();
    assert_eq!(vazio, []);
    assert!(matches!(
        modelo::cadastrar(&banco, &origem, "ensaio", "7", "293", 999),
        Err(modelo::Falha::UploadAusente)
    ));
    assert!(matches!(
        modelo::receber(
            &banco,
            &origem,
            "ensaio",
            "7",
            "maior.pdf",
            201 * 1024 * 1024
        ),
        Err(modelo::Falha::LimiteExcedido)
    ));
    assert_eq!(
        modelo::documentos(&banco, &origem, "outro", "7")
            .unwrap()
            .len(),
        0
    );
    assert!(
        modelo::processo(&banco, &origem, "outro", "7")
            .unwrap()
            .is_none()
    );
    let outra = Instalacao::interpretar("https://outra.exemplo/sei").unwrap();
    assert!(
        modelo::processo(&banco, &outra, "ensaio", "7")
            .unwrap()
            .is_none()
    );
    assert_eq!(
        modelo::documentos(&banco, &origem, "ensaio", "7").unwrap(),
        []
    );
}

#[test]
fn recibo_legado_sem_bytes_nao_publica_pdf_sem_download() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    semear(&banco);
    let origem = origem();
    let recibo = modelo::receber(&banco, &origem, "ensaio", "7", "legado.pdf", 42).unwrap();
    let resultado = modelo::cadastrar(&banco, &origem, "ensaio", "7", "293", recibo.id);
    assert!(matches!(
        resultado,
        Err(modelo::Falha::UploadSemConteudoRecuperavel)
    ));
    let cadastro = anti_ies::pdf_externo::atender(
        "POST",
        "/sei/controlador.php?acao=documento_receber&id_procedimento=7",
        b"selSerie=293&hdnFlagDocumentoCadastro=2&hdnAnexos=upload-1%C2%B1legado.pdf",
        &banco,
        &origem,
        "ensaio",
    )
    .unwrap();
    assert_eq!(cadastro.estado, anti_ies::Estado::RequisicaoInvalida);
    assert_eq!(
        modelo::documentos(&banco, &origem, "ensaio", "7").unwrap(),
        []
    );

    let arvore = anti_ies::pdf_externo::atender(
        "GET",
        "/sei/controlador.php?acao=arvore_visualizar&id_procedimento=7",
        b"",
        &banco,
        &origem,
        "ensaio",
    )
    .unwrap();
    assert!(!arvore.corpo.contains("legado.pdf"));
}

#[test]
fn bdd_332_03_multipart_com_campo_adicional_preserva_pdf_e_cadastro() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    semear(&banco);
    let origem = origem();
    let alvo = "/sei/controlador.php?acao=documento_upload_anexo&id_procedimento=7";
    let multipart = b"--B\r\nContent-Disposition: form-data; name=\"contexto\"\r\n\r\nensaio\r\n--B\r\nContent-Disposition: form-data; name=\"filArquivo\"; filename=\"prova.pdf\"\r\nContent-Type: application/pdf\r\n\r\n%PDF-1.7\nconteudo\r\n--B--\r\n";
    let resposta =
        anti_ies::pdf_externo::atender("POST", alvo, multipart, &banco, &origem, "ensaio").unwrap();
    assert_eq!(resposta.estado, anti_ies::Estado::Ok);
    assert!(
        resposta
            .corpo
            .contains("upload-1#prova.pdf#application/pdf#")
    );
    let temporarios: i64 = banco
        .query_row("SELECT COUNT(*) FROM externo_upload", [], |r| r.get(0))
        .unwrap();
    assert_eq!(temporarios, 1);
    let definitivo = anti_ies::pdf_externo::atender(
        "POST",
        "/sei/controlador.php?acao=documento_receber&id_procedimento=7",
        b"selSerie=293&hdnFlagDocumentoCadastro=2&hdnAnexos=upload-1%C2%B1prova.pdf",
        &banco,
        &origem,
        "ensaio",
    )
    .unwrap();
    assert_eq!(definitivo.estado, anti_ies::Estado::Redirecionamento);
    let docs = modelo::documentos(&banco, &origem, "ensaio", "7").unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].nome, "prova.pdf");
    assert_eq!(docs[0].tamanho, b"%PDF-1.7\nconteudo".len() as u64);
}

#[test]
fn multipart_com_dois_arquivos_nao_pode_gerar_recibo_ambiguo() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    semear(&banco);
    let origem = origem();
    let multipart = b"--B\r\nContent-Disposition: form-data; name=\"filArquivo\"; filename=\"a.pdf\"\r\n\r\n%PDF-1.7\nA\r\n--B\r\nContent-Disposition: form-data; name=\"filArquivo\"; filename=\"b.pdf\"\r\n\r\n%PDF-1.7\nB\r\n--B--\r\n";
    let resposta = anti_ies::pdf_externo::atender(
        "POST",
        "/sei/controlador.php?acao=documento_upload_anexo&id_procedimento=7",
        multipart,
        &banco,
        &origem,
        "ensaio",
    )
    .unwrap();
    assert_eq!(resposta.estado, anti_ies::Estado::RequisicaoInvalida);
    let temporarios: i64 = banco
        .query_row("SELECT COUNT(*) FROM externo_upload", [], |r| r.get(0))
        .unwrap();
    assert_eq!(temporarios, 0);
}

#[test]
fn bdd_332_04_releitura_sqlite_indisponivel_nao_vira_processo_inexistente() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    semear(&banco);
    let origem = origem();
    let recibo = modelo::receber_conteudo(
        &banco,
        &origem,
        "ensaio",
        "7",
        "prova.pdf",
        b"%PDF-1.7\nconteudo",
    )
    .unwrap();
    modelo::cadastrar(&banco, &origem, "ensaio", "7", "293", recibo.id).unwrap();
    banco.execute_batch("DROP TABLE sim_processo").unwrap();
    let resposta = anti_ies::pdf_externo::atender(
        "GET",
        "/sei/controlador.php?acao=arvore_visualizar&id_procedimento=7",
        b"",
        &banco,
        &origem,
        "ensaio",
    )
    .unwrap();
    assert_eq!(resposta.estado, anti_ies::Estado::DadosInsuficientes);
    assert!(!resposta.corpo.contains("050000001"));
}

#[test]
fn bdd_332_03_arvore_compartilhada_mostra_documentos_interno_e_externo() {
    use iesei::documentos_simulados as interno;
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        semear(&banco);
        let origem = origem();
        interno::registrar_tipo(
            &banco,
            &origem,
            "ensaio",
            "7",
            &interno::Tipo {
                id: "291".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
        interno::criar(&banco, &origem, "ensaio", "7", "291", "Interno").unwrap();
        let temporario = modelo::receber_conteudo(
            &banco,
            &origem,
            "ensaio",
            "7",
            "anexo.pdf",
            b"%PDF-1.7\nconteudo",
        )
        .unwrap();
        modelo::cadastrar(&banco, &origem, "ensaio", "7", "293", temporario.id).unwrap();
        for _ in 0..2 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut fluxo, Some((&banco, &origem, "ensaio"))).unwrap();
        }
    });
    let escolha = pedir(
        endereco,
        "GET",
        "/sei/controlador.php?acao=documento_escolher_tipo&id_procedimento=7",
        b"",
    );
    assert!(escolha.contains("frmDocumentoEscolherTipo"));
    assert!(escolha.contains("incluirExterno"));
    let arvore = pedir(
        endereco,
        "GET",
        "/sei/controlador.php?acao=arvore_visualizar&id_procedimento=7",
        b"",
    );
    assert!(arvore.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(arvore.contains("Despacho"));
    assert!(arvore.contains("1000001"));
    assert!(arvore.contains("Anexo"));
    assert!(arvore.contains("050000001"));
    assert!(arvore.contains("anexo.pdf"));
    servidor.join().unwrap();
}

#[test]
fn falha_ao_persistir_os_bytes_nao_deixa_upload_parcial() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    semear(&banco);
    // Injeta uma falha de persistência no segundo BLOB, em INSERT ou UPDATE.
    banco
        .execute_batch("CREATE UNIQUE INDEX falha_no_blob ON externo_upload(conteudo)")
        .unwrap();
    let origem = origem();
    let bytes = b"%PDF-1.7";
    modelo::receber_conteudo(&banco, &origem, "ensaio", "7", "primeiro.pdf", bytes).unwrap();
    assert!(matches!(
        modelo::receber_conteudo(&banco, &origem, "ensaio", "7", "segundo.pdf", bytes),
        Err(modelo::Falha::Sqlite(_))
    ));
    let uploads: i64 = banco
        .query_row("SELECT count(*) FROM externo_upload", [], |r| r.get(0))
        .unwrap();
    assert_eq!(uploads, 1);
}
