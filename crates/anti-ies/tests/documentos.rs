//! Navegação HTTP sintética com um único SQLite entre várias requisições.
//! Teste técnico auxiliar da #246. Não representa compatibilidade SEI real.
use anti_ies::atender_http;
use iesei::banco_cassettes::iniciar;
use iesei::documentos_simulados::{self as sim, Processo, Tipo};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

const BASE: &str = "/sei/controlador.php";
const NUMERO_PEDIDOS: usize = 15;

fn consultar(servidor: std::net::SocketAddr, metodo: &str, alvo: &str, corpo: &str) -> String {
    let mut fluxo = TcpStream::connect(servidor).expect("conectar loopback");
    write!(
        fluxo,
        "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{corpo}",
        corpo.len()
    )
    .expect("enviar requisição");
    fluxo.shutdown(Shutdown::Write).expect("encerrar envio");
    let mut resposta = String::new();
    fluxo
        .read_to_string(&mut resposta)
        .expect("receber resposta");
    resposta
}

fn verificar_navegacao_da_serie(endereco: std::net::SocketAddr, selecao: &str) {
    // Escolha por POST e apenas navegacao; criar exige outro POST no cadastro.
    let escolhido = consultar(endereco, "POST", selecao, "hdnIdSerie=291");
    assert!(escolhido.starts_with("HTTP/1.1 302 Found"));
    assert!(escolhido.contains(
        "Location: controlador.php?acao=documento_gerar&id_procedimento=10&id_serie=291\r\n"
    ));
    let ausente = consultar(endereco, "POST", selecao, "hdnIdSerie=999");
    assert!(ausente.starts_with("HTTP/1.1 404 Not Found"));
    let repetido = consultar(endereco, "POST", selecao, "hdnIdSerie=291&hdnIdSerie=291");
    assert!(repetido.starts_with("HTTP/1.1 400 Bad Request"));
}

#[test]
fn bdd_246_navegacao_post_unico_e_releitura_mantem_estado_compartilhado() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        let cenario = "bdd-246";
        sim::registrar_processo(
            &banco,
            &origem,
            cenario,
            &Processo {
                id: "10".into(),
                nup: "00001.000001/2026-01".into(),
                unidade: "UNIDADE TESTE".into(),
            },
        )
        .unwrap();
        sim::registrar_tipo(
            &banco,
            &origem,
            cenario,
            "10",
            &Tipo {
                id: "291".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
        let anterior = sim::criar(&banco, &origem, cenario, "10", "291", "Anterior").unwrap();
        assert_eq!(anterior.numero_sei, "1000001");
        for _ in 0..NUMERO_PEDIDOS {
            let (mut fluxo, _) = escuta.accept().expect("aceitar requisição");
            atender_http(&mut fluxo, Some((&banco, &origem, cenario))).unwrap();
        }
        let docs = sim::documentos(&banco, &origem, cenario, "10").unwrap();
        assert_eq!(docs.len(), 2, "apenas o POST válido cria um documento");
        assert_eq!(docs[0].descricao, "Anterior");
        assert_eq!(docs[1].descricao, "Ensaio PEN1");
    });
    let arvore = format!("{BASE}?acao=procedimento_trabalhar&id_procedimento=10");
    let selecao = format!("{BASE}?acao=documento_escolher_tipo&id_procedimento=10");
    let cadastro = format!("{BASE}?acao=documento_gerar&id_procedimento=10&id_serie=291");
    let render1 = consultar(endereco, "GET", &arvore, "");
    assert!(render1.starts_with("HTTP/1.1 200 OK"));
    assert!(render1.contains("Despacho 1000001"));
    let arvore_antes = consultar(endereco, "GET", &format!("{BASE}?acao=procedimento_visualizar&id_procedimento=10"), "");
    assert!(arvore_antes.starts_with("HTTP/1.1 200 OK"));
    assert!(arvore_antes.contains("Nos[0] = new infraArvoreNo"));
    assert!(arvore_antes.contains("Despacho 1000001"));
    assert!(!arvore_antes.contains("Despacho 1000002"));
    assert!(!render1.contains("Despacho 1000002"));
    let lista = consultar(endereco, "GET", &selecao, "");
    assert!(lista.starts_with("HTTP/1.1 200 OK"));
    assert!(lista.contains("tblSeries"));
    assert!(lista.contains("frmDocumentoEscolherTipo"));
    assert!(lista.contains("onclick='escolher(291)'"));
    verificar_navegacao_da_serie(endereco, &selecao);
    assert!(lista.contains(">Despacho</a>") || lista.contains("Despacho</a>"));
    let form = consultar(endereco, "GET", &cadastro, "");
    assert!(form.starts_with("HTTP/1.1 200 OK"));
    assert!(form.contains("frmDocumentoCadastro"));
    // O HTML servido deve permitir o mesmo POST que o anti-IES aceita.
    assert!(form.contains("name='hdnFlagDocumentoCadastro' value='1'"));
    assert!(form.contains("id='btnSalvar' name='btnSalvar'"));
    assert!(form.contains("onclick='confirmarDados()'"));
    let invalido = consultar(
        endereco,
        "POST",
        &cadastro,
        "hdnIdSerie=999&hdnFlagDocumentoCadastro=2&rdoNivelAcesso=0&txtDescricao=Errado",
    );
    assert!(invalido.starts_with("HTTP/1.1 400 Bad Request"));
    let apos_recusa = consultar(endereco, "GET", &arvore, "");
    assert!(!apos_recusa.contains("Despacho 1000002"));
    let duplicado = consultar(
        endereco,
        "POST",
        &cadastro,
        "hdnIdSerie=291&hdnIdSerie=291&hdnFlagDocumentoCadastro=2&rdoNivelAcesso=0&txtDescricao=Errado",
    );
    assert!(duplicado.starts_with("HTTP/1.1 400 Bad Request"));
    let corpo =
        "hdnIdSerie=291&hdnFlagDocumentoCadastro=2&rdoNivelAcesso=0&txtDescricao=Ensaio+PEN1";
    let confirmacao = consultar(endereco, "POST", &cadastro, corpo);
    assert!(confirmacao.starts_with("HTTP/1.1 302 Found"));
    assert!(confirmacao.contains(
        "Location: controlador.php?acao=arvore_visualizar&id_procedimento=10&id_documento=2\r\n"
    ));
    let apos = consultar(endereco, "GET", &arvore, "");
    assert!(apos.starts_with("HTTP/1.1 200 OK"));
    assert!(apos.contains("Despacho 1000001"));
    assert!(apos.contains("Despacho 1000002"));
    let arvore_depois = consultar(endereco, "GET", &format!("{BASE}?acao=procedimento_visualizar&id_procedimento=10"), "");
    assert!(arvore_depois.starts_with("HTTP/1.1 200 OK"));
    assert!(arvore_depois.contains("Despacho 1000001"));
    assert!(arvore_depois.contains("Despacho 1000002"));
    let metodo_errado = consultar(endereco, "PUT", &selecao, "");
    assert!(metodo_errado.starts_with("HTTP/1.1 405 Method Not Allowed"));
    let sem_nup = consultar(
        endereco,
        "GET",
        &format!("{BASE}?acao=procedimento_trabalhar&id_procedimento=999"),
        "",
    );
    assert!(sem_nup.starts_with("HTTP/1.1 404 Not Found"));
    servidor.join().unwrap();
}
