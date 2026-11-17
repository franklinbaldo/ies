//! #88: a pesquisa rapida liga a CLI ao mesmo documento do SQLite.
//! Ensaio HTTP sintetico sem SEI externo nem credenciais.
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::documento_versoes::{self, DocumentoSimulado, Revisao};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::num::NonZeroU32;

const BASE: &str = "/sei/controlador.php";

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn guardar_cenario(banco: &mut Connection, cenario: &str) {
    let instalacao = origem();
    banco_cassettes::guardar(
        banco,
        cenario,
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("PPI").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    documento_versoes::guardar(
        banco,
        cenario,
        &DocumentoSimulado {
            instalacao,
            id_documento: "42".into(),
            id_procedimento: "7".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: "0011519".into(),
            tipo: "Ofício".into(),
            revisoes: vec![
                Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "Ana".into(),
                    unidade: "PPI".into(),
                    modificacao: "01/11/2026".into(),
                    html_corpo: "<p>Texto histórico</p>".into(),
                },
                Revisao {
                    versao: NonZeroU32::new(3).unwrap(),
                    usuario: "Bia".into(),
                    unidade: "PPI".into(),
                    modificacao: "03/11/2026".into(),
                    html_corpo: "<p>Texto atual</p>".into(),
                },
            ],
        },
    )
    .unwrap();
}

fn pedir(endereco: SocketAddr, metodo: &str, alvo: &str, corpo: &str) -> String {
    let mut fluxo = TcpStream::connect(endereco).unwrap();
    write!(
        fluxo,
        "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{corpo}",
        corpo.len()
    )
    .unwrap();
    fluxo.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    fluxo.read_to_string(&mut resposta).unwrap();
    resposta
}

#[test]
fn bdd_88_01_02_controle_pesquisa_arvore_historico_e_conteudo_em_loopback() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        guardar_cenario(&mut banco, "documento-88");
        let origem = origem();
        for _ in 0..8 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            atender_http(&mut fluxo, Some((&banco, &origem, "documento-88"))).unwrap();
        }
    });
    let controle = pedir(
        endereco,
        "GET",
        &format!("{BASE}?acao=procedimento_controlar"),
        "",
    );
    assert!(controle.starts_with("HTTP/1.1 200 OK"), "{controle}");
    assert!(controle.contains("id=\"frmProtocoloPesquisaRapida\""));
    assert!(controle.contains("name=\"txtPesquisaRapida\""));
    let pesquisa = pedir(
        endereco,
        "POST",
        &format!("{BASE}?acao=protocolo_pesquisa_rapida"),
        "txtPesquisaRapida=0011519",
    );
    assert!(pesquisa.starts_with("HTTP/1.1 302 Found"), "{pesquisa}");
    // Cassettes documento-numero-e-busca-* (sete instalações): o 302 de
    // número SEI endereça id_documento, diferentemente do NUP.
    assert!(pesquisa.contains("id_documento=42"), "{pesquisa}");
    assert!(!pesquisa.contains("id_protocolo=42"), "{pesquisa}");
    let tela = pedir(
        endereco,
        "GET",
        &format!("{BASE}?acao=procedimento_trabalhar&id_documento=42"),
        "",
    );
    assert!(tela.contains("id=\"ifrArvore\""), "{tela}");
    let arvore = pedir(
        endereco,
        "GET",
        &format!("{BASE}?acao=procedimento_visualizar&id_procedimento=7&id_documento=42"),
        "",
    );
    assert!(arvore.contains("documento_versao_listar"), "{arvore}");
    let lista = pedir(
        endereco,
        "GET",
        &format!("{BASE}?acao=documento_versao_listar&id_documento=42"),
        "",
    );
    assert!(lista.contains("Lista de Versões (2 registros)"), "{lista}");
    let antiga = pedir(
        endereco,
        "GET",
        &format!("{BASE}?acao=documento_visualizar&id_documento=42&versao=1"),
        "",
    );
    assert!(antiga.contains("Texto histórico"));
    assert!(!antiga.contains("Texto atual"));
    let ausente = pedir(
        endereco,
        "POST",
        &format!("{BASE}?acao=protocolo_pesquisa_rapida"),
        "txtPesquisaRapida=0099999",
    );
    assert!(
        ausente.starts_with("HTTP/1.1 503 Service Unavailable"),
        "{ausente}"
    );
    let repetido = pedir(
        endereco,
        "POST",
        &format!("{BASE}?acao=protocolo_pesquisa_rapida"),
        "txtPesquisaRapida=0011519&txtPesquisaRapida=0011519",
    );
    assert!(
        repetido.starts_with("HTTP/1.1 400 Bad Request"),
        "{repetido}"
    );
    servidor.join().unwrap();
}

#[test]
fn bdd_88_03_pesquisa_nao_finge_sucesso_sem_snapshot_do_documento() {
    let banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let alvo = format!("{BASE}?acao=protocolo_pesquisa_rapida");
    assert_eq!(
        anti_ies::pesquisa_documento::responder(
            "POST",
            &alvo,
            "txtPesquisaRapida=0011519",
            &banco,
            &origem(),
            "vazio"
        )
        .estado,
        anti_ies::Estado::DadosInsuficientes
    );
}
