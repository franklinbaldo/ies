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

#[test]
fn bdd_332_02_03_04_hypotese_sintetica_upload_e_cadastro_sao_estados_distintos() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        semear(&banco);
        let origem = origem();
        for _ in 0..7 {
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
    let cadastro = pedir(
        endereco,
        "GET",
        &format!("{base}documento_receber&id_procedimento=7"),
        b"",
    );
    assert!(cadastro.contains("id='selSerie'"));
    assert!(cadastro.contains("value='293'"));
    let recarregado = pedir(
        endereco,
        "POST",
        &format!("{base}documento_receber&id_procedimento=7"),
        b"hdnFlagDocumentoCadastro=1&selSerie=293",
    );
    assert!(recarregado.contains("selected='selected'"));
    assert!(recarregado.contains("documento_upload_anexo"));
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
    assert!(definitivo.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(definitivo.contains("Documento 050000001"));

    let depois = pedir(
        endereco,
        "GET",
        &format!("{base}arvore_visualizar&id_procedimento=7"),
        b"",
    );
    assert!(depois.contains("documento_pdf"));
    assert!(depois.contains("prova.pdf"));
    assert!(depois.contains("050000001"));
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
