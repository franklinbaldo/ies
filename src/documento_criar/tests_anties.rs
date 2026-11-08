//! Fronteira tipada IES ↔ anti-IES: a lista vem de uma resposta HTTP real.
//! Teste técnico da seleção BDD-246-01/03, não um teste da CLI completa.
use super::{ErroCriar, EscolhaDoDocumento, conferir_tipo, escolha_do_tipo, tipos_oferecidos};
use anti_ies::atender_http;
use iesei::banco_cassettes::iniciar;
use iesei::documentos_simulados::{Processo, Tipo, registrar_processo, registrar_tipo};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

#[test]
fn lista_renderizada_pelo_anti_ies_e_consumida_pelo_seletor_do_ies() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        registrar_processo(
            &banco,
            &origem,
            "bdd-246",
            &Processo {
                id: "10".into(),
                nup: "00001.000001/2026-01".into(),
                unidade: "TESTE".into(),
            },
        )
        .unwrap();
        registrar_tipo(
            &banco,
            &origem,
            "bdd-246",
            "10",
            &Tipo {
                id: "291".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
        let (mut fluxo, _) = escuta.accept().unwrap();
        atender_http(&mut fluxo, Some((&banco, &origem, "bdd-246"))).unwrap();
    });
    let mut cliente = TcpStream::connect(endereco).unwrap();
    write!(
        cliente,
        "GET /sei/controlador.php?acao=documento_escolher_tipo&id_procedimento=10 HTTP/1.1\r\nHost: localhost\r\n\r\n"
    ).unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    servidor.join().unwrap();
    assert!(resposta.starts_with("HTTP/1.1 200 OK"));
    let (_, html) = resposta.split_once("\r\n\r\n").unwrap();
    let tipos = tipos_oferecidos(html).unwrap();
    assert_eq!(tipos, ["Despacho"]);
    let instalacao =
        crate::instalacao::Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
    assert_eq!(
        escolha_do_tipo(&instalacao, html, "10", "Despacho"),
        Ok(EscolhaDoDocumento::Formulario("291".into())),
    );
    assert_eq!(conferir_tipo("Despacho", &tipos), Ok(()));
    assert_eq!(
        conferir_tipo("Parecer", &tipos),
        Err(ErroCriar::TipoAusente)
    );
    assert_eq!(
        conferir_tipo("Despacho", &["Despacho".into(), "Despacho".into()]),
        Err(ErroCriar::TipoAmbiguo)
    );
}

/// BDD-246-02: uma leitura da árvore, navegação sem efeito, único POST
/// criador e releitura independente. HTML real recebido no loopback é
/// interpretado pelos leitores da produção, sem confiar em markup fixo.
fn iniciar_anti_ies_com_documento() -> (std::net::SocketAddr, std::thread::JoinHandle<()>) {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        registrar_processo(
            &banco,
            &origem,
            "bdd-246",
            &Processo {
                id: "10".into(),
                nup: "00001.000001/2026-01".into(),
                unidade: "TESTE".into(),
            },
        )
        .unwrap();
        registrar_tipo(
            &banco,
            &origem,
            "bdd-246",
            "10",
            &Tipo {
                id: "291".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
        iesei::documentos_simulados::criar(&banco, &origem, "bdd-246", "10", "291", "Anterior")
            .unwrap();
        for _ in 0..8 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            atender_http(&mut fluxo, Some((&banco, &origem, "bdd-246"))).unwrap();
        }
    });
    (endereco, servidor)
}

#[test]
fn bdd_246_cadastro_e_arvore_antes_depois_consumidos_pelo_ies() {
    let (endereco, servidor) = iniciar_anti_ies_com_documento();
    let consultar = |metodo: &str, alvo: &str, corpo: &str| {
        let mut fluxo = TcpStream::connect(endereco).unwrap();
        write!(
            fluxo,
            "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{corpo}",
            corpo.len()
        )
        .unwrap();
        fluxo.shutdown(Shutdown::Write).unwrap();
        let mut resposta = String::new();
        fluxo.read_to_string(&mut resposta).unwrap();
        resposta
    };
    let raiz = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
    let escolha = "/sei/controlador.php?acao=documento_escolher_tipo&id_procedimento=10";
    let cadastro = "/sei/controlador.php?acao=documento_gerar&id_procedimento=10&id_serie=291";

    let trabalho = consultar(
        "GET",
        "/sei/controlador.php?acao=procedimento_trabalhar&id_procedimento=10",
        "",
    );
    assert!(trabalho.starts_with("HTTP/1.1 200 OK"));
    let (_, html_trabalho) = trabalho.split_once("\r\n\r\n").unwrap();
    let origem =
        crate::instalacao::Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
    let tela =
        crate::arvore_processo::ler_tela_do_processo(&origem, html_trabalho, "10", Some("TESTE"))
            .unwrap();
    assert!(tela.arvore.as_str().contains("procedimento_visualizar"));

    let resposta_antes = consultar("GET", raiz, "");
    assert!(resposta_antes.starts_with("HTTP/1.1 200 OK"));
    let (_, html_antes) = resposta_antes.split_once("\r\n\r\n").unwrap();
    let arvore_antes =
        crate::arvore_processo::ler_arvore(html_antes, "00001.000001/2026-01", "10").unwrap();
    assert_eq!(arvore_antes.documentos.len(), 1);

    let lista = consultar("GET", escolha, "");
    let (_, html_lista) = lista.split_once("\r\n\r\n").unwrap();
    assert_eq!(super::tipos_oferecidos(html_lista).unwrap(), ["Despacho"]);
    let navegacao = consultar("POST", escolha, "hdnIdSerie=291");
    assert!(navegacao.starts_with("HTTP/1.1 302 Found"));
    let formulario = consultar("GET", cadastro, "");
    assert!(formulario.starts_with("HTTP/1.1 200 OK"));
    let (_, html_cadastro) = formulario.split_once("\r\n\r\n").unwrap();
    super::conferir_cadastro(html_cadastro, "Despacho", Some("Novo"), Some("publico")).unwrap();
    assert!(super::guarda_script::cadastro_oferece_salvar(html_cadastro));

    let sem_criacao = consultar("GET", raiz, "");
    let (_, html_sem_criacao) = sem_criacao.split_once("\r\n\r\n").unwrap();
    assert_eq!(
        crate::arvore_processo::ler_arvore(html_sem_criacao, "00001.000001/2026-01", "10",)
            .unwrap()
            .documentos,
        arvore_antes.documentos
    );

    let resposta = consultar(
        "POST",
        cadastro,
        "hdnIdSerie=291&hdnFlagDocumentoCadastro=2&rdoNivelAcesso=0&txtDescricao=Novo",
    );
    assert!(resposta.starts_with("HTTP/1.1 302 Found"));
    assert!(resposta.contains("id_documento=2\r\n"));
    let resposta_depois = consultar("GET", raiz, "");
    let (_, html_depois) = resposta_depois.split_once("\r\n\r\n").unwrap();
    let arvore_depois =
        crate::arvore_processo::ler_arvore(html_depois, "00001.000001/2026-01", "10").unwrap();
    assert_eq!(
        super::conferir_criacao(
            &arvore_antes.documentos,
            &arvore_depois.documentos,
            "Despacho",
        ),
        Ok("1000002".to_owned())
    );
    servidor.join().unwrap();
}
