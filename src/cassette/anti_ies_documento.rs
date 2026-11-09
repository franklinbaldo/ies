//! Contraste parcial com conteúdo original da cassette `UnB` (#60).
//! O leitor do IES estabelece identidade/texto; o anti-IES projeta um
//! documento do SQLite e os mesmos leitores conferem a resposta HTTP.
//! A versão, o ID e o autor do simulador são sintéticos, não observações.
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use iesei::banco_cassettes::iniciar;
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::num::NonZeroU32;

fn escapar(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

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

#[test]
#[ignore = "ZIP original UnB SHA-verificado no IES_ACERVO_CACHE; somente offline"]
fn cassette_unb_conteudo_documento_compara_leitores_por_http() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join("cassettes/unb/liberacoes-e-versoes-leitura-unb--a781b0fbca8f.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("lote-9497057d512230ff.zip")).unwrap();
    let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
    let instalacao = gravacao::instalacao(&captura.json).unwrap();
    let originais: Vec<_> = gravacao::respostas(&captura.json)
        .into_iter()
        .filter(|r| r.operacao == "documento_visualizar" && r.metodo == "GET" && r.status == 200)
        .collect();
    assert_eq!(
        originais.len(),
        1,
        "não inventar resposta ausente na cassette"
    );
    let html_original = html_da_resposta(originais[0].item).unwrap();
    let identidade = crate::documento_web::identidade_do_titulo(&html_original).unwrap();
    let texto_original = crate::documento_web::texto_do_corpo(&html_original).unwrap();
    assert!(!texto_original.contains("infra_hash"));

    let documento = DocumentoSimulado {
        instalacao: instalacao.clone(),
        id_documento: "42".to_owned(),
        id_procedimento: "7".to_owned(),
        numero_processo: "00001.000001/2026-01".to_owned(),
        numero: identidade.numero.clone(),
        tipo: identidade.tipo.clone(),
        revisoes: vec![Revisao {
            versao: NonZeroU32::new(1).unwrap(),
            usuario: "SIMULADO".to_owned(),
            unidade: "SIMULADO".to_owned(),
            modificacao: "09/11/2026".to_owned(),
            html_corpo: format!("<p>{}</p>", escapar(&texto_original)),
        }],
    };
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let tarefa = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        guardar(&mut banco, "cassette-conteudo", &documento).unwrap();
        for _ in 0..3 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut fluxo, Some((&banco, &instalacao, "cassette-conteudo")))
                .unwrap();
        }
        assert_eq!(
            banco
                .query_row("SELECT count(*) FROM documento_revisao", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1,
            "GET e POST recusado não modificam o SQLite"
        );
    });
    let url =
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=1&id_procedimento=7";
    let resposta = pedir(endereco, "GET", url);
    let (cabecalho, html_anti) = resposta.split_once("\r\n\r\n").unwrap();
    assert!(cabecalho.starts_with("HTTP/1.1 200 OK"), "{cabecalho}");
    let identidade_anti = crate::documento_web::identidade_do_titulo(html_anti).unwrap();
    assert_eq!(
        identidade_anti, identidade,
        "número/tipo vêm da fonte original"
    );
    let texto_anti = crate::documento_web::texto_do_corpo(html_anti).unwrap();
    assert!(
        texto_anti.starts_with(&texto_original),
        "o texto original deve sobreviver ao transporte e renderização"
    );
    let cruzado = pedir(
        endereco,
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=1&id_procedimento=8",
    );
    assert!(
        cruzado.starts_with("HTTP/1.1 404"),
        "identidade de outro processo"
    );
    assert!(pedir(endereco, "POST", url).starts_with("HTTP/1.1 405"));
    tarefa.join().unwrap();
}
