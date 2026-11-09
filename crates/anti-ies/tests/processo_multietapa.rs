//! Regressão técnica BDD-333-01/02: duas tramitações sucessivas pelo HTTP local.
//! O mesmo SQLite sustenta Controle R, barra, formulário, POST e releitura.
use anti_ies::atender_http;
use iesei::banco_cassettes::{ControleResumido, guardar as guardar_controle, iniciar};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::processos::{Destino, ProcessoSimulado, guardar, ler};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

const CENARIO: &str = "bdd-333-duas-tramitacoes";
const NUP: &str = "00001.000001/2026-01";
const ARVORE: &str = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
const ENVIO: &str = "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10";

fn controlar(banco: &mut Connection, instalacao: &Instalacao, unidade: &str) {
    guardar_controle(
        banco,
        CENARIO,
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova(unidade).unwrap(),
            recebidos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
}

fn requisitar(endereco: SocketAddr, metodo: &str, alvo: &str, corpo: &str) -> String {
    let mut cliente = TcpStream::connect(endereco).unwrap();
    write!(
        cliente,
        "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{corpo}",
        corpo.len()
    )
    .unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    resposta
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "duas tramitações e uma tentativa recusada em dez pedidos HTTP no mesmo SQLite"
)]
fn duas_tramitacoes_sucessivas_trocam_a_unidade_da_sessao_sem_duplicar_post() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let instalacao = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        guardar(
            &mut banco,
            &instalacao,
            CENARIO,
            &ProcessoSimulado {
                id: "10".into(),
                numero: NUP.into(),
                unidade_origem: "ORIGEM".into(),
                aberto_em: vec!["ORIGEM".into()],
                anotacao_origem: Some("Nota da origem".into()),
                destinos: vec![
                    Destino {
                        id: "2".into(),
                        sigla: "DESTINO".into(),
                        descricao: "DESTINO - Unidade de testes".into(),
                    },
                    Destino {
                        id: "3".into(),
                        sigla: "ARQUIVO".into(),
                        descricao: "ARQUIVO - Unidade de testes".into(),
                    },
                ],
            },
        )
        .unwrap();
        controlar(&mut banco, &instalacao, "ORIGEM");
        for requisicao in 1..=10 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            atender_http(&mut fluxo, Some((&banco, &instalacao, CENARIO))).unwrap();
            if requisicao == 3 {
                assert_eq!(
                    ler(&banco, &instalacao, CENARIO, "10")
                        .unwrap()
                        .unwrap()
                        .aberto_em,
                    ["DESTINO"]
                );
                controlar(&mut banco, &instalacao, "DESTINO");
            }
            if requisicao == 7 {
                let estado = ler(&banco, &instalacao, CENARIO, "10").unwrap().unwrap();
                assert_eq!(estado.aberto_em, ["DESTINO"]);
                assert_eq!(estado.anotacao_origem.as_deref(), Some("Nota da origem"));
            }
            if requisicao == 8 {
                assert_eq!(
                    ler(&banco, &instalacao, CENARIO, "10")
                        .unwrap()
                        .unwrap()
                        .aberto_em,
                    ["ARQUIVO"]
                );
                controlar(&mut banco, &instalacao, "ARQUIVO");
            }
        }
        ler(&banco, &instalacao, CENARIO, "10").unwrap().unwrap()
    });

    let caixa_origem = requisitar(
        endereco,
        "GET",
        "/sei/controlador.php?acao=procedimento_controlar",
        "",
    );
    assert!(caixa_origem.contains(NUP));
    let arvore_origem = requisitar(endereco, "GET", ARVORE, "");
    assert!(arvore_origem.contains("Enviar"));
    assert!(arvore_origem.contains("Incluir Documento"));

    let primeiro = requisitar(
        endereco,
        "POST",
        ENVIO,
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO",
    );
    assert!(primeiro.starts_with("HTTP/1.1 302 Found"));

    let trabalho = requisitar(
        endereco,
        "GET",
        "/sei/controlador.php?acao=procedimento_trabalhar&id_procedimento=10",
        "",
    );
    assert!(trabalho.contains("id='lnkInfraUnidade'>DESTINO</a>"));
    assert!(!trabalho.contains("Incluir Documento"));
    let arvore_destino = requisitar(endereco, "GET", ARVORE, "");
    assert!(arvore_destino.contains("Enviar"));
    assert!(!arvore_destino.contains("Incluir Documento"));
    assert!(!arvore_destino.contains("Anotações"));
    let formulario = requisitar(endereco, "GET", ENVIO, "");
    assert!(formulario.contains("frmAtividadeListar"));
    assert!(
        !formulario.contains("chkSinRemoverAnotacoes"),
        "a anotacao da ORIGEM fechada nao pode ser oferecida para remocao em DESTINO"
    );

    // Um POST antigo tentando limpar a anotação da ORIGEM na unidade
    // DESTINO não pode alterar nenhuma das duas transições do processo.
    let indevido = requisitar(
        endereco,
        "POST",
        ENVIO,
        "hdnIdProtocolos=10&hdnUnidades=3%B1ARQUIVO&chkSinRemoverAnotacoes=on",
    );
    assert!(indevido.starts_with("HTTP/1.1 400 Bad Request"));

    let segundo = requisitar(
        endereco,
        "POST",
        ENVIO,
        "hdnIdProtocolos=10&hdnUnidades=3%B1ARQUIVO",
    );
    assert!(segundo.starts_with("HTTP/1.1 302 Found"));

    let caixa_arquivo = requisitar(
        endereco,
        "GET",
        "/sei/controlador.php?acao=procedimento_controlar",
        "",
    );
    assert!(caixa_arquivo.contains(NUP));
    let arvore_final = requisitar(endereco, "GET", ARVORE, "");
    assert!(arvore_final.contains("Processo aberto somente na unidade"));
    let depois = servidor.join().unwrap();
    assert_eq!(depois.aberto_em, ["ARQUIVO"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some("Nota da origem"));
}
