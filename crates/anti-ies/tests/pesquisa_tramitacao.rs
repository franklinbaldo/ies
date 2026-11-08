//! BDD-333-01/02: a pesquisa por NUP deve abrir o processo da tramitação,
//! inclusive após mudar a unidade ativa. HTTP local + SQLite compartilhado.
//! Hipótese sintética: não comprova compatibilidade com o SEI real.
use anti_ies::atender_http;
use iesei::banco_cassettes::{ControleResumido, guardar as guardar_controle, iniciar};
use iesei::documentos_simulados::{Processo, registrar_processo};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::processos::{Destino, ProcessoSimulado, guardar, ler};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

const CENARIO: &str = "bdd-333-pesquisa";
const NUP: &str = "00001.000001/2026-01";

fn origem() -> Instalacao {
    Instalacao::interpretar("https://simulado.invalid/sei").unwrap()
}

fn controle(banco: &mut Connection, unidade: &str) {
    guardar_controle(
        banco,
        CENARIO,
        &ControleResumido {
            instalacao: origem(),
            unidade: UnidadeExibida::nova(unidade).unwrap(),
            recebidos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
}

fn preparar() -> Connection {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    controle(&mut banco, "ORIGEM");
    // A identidade também existe no cadastro documental: as duas capacidades
    // não podem disputar a navegação do mesmo id_protocolo.
    registrar_processo(
        &banco,
        &origem(),
        CENARIO,
        &Processo {
            id: "10".into(),
            nup: NUP.into(),
            unidade: "ORIGEM".into(),
        },
    )
    .unwrap();
    guardar(
        &mut banco,
        &origem(),
        CENARIO,
        &ProcessoSimulado {
            id: "10".into(),
            numero: NUP.into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: Some("anotacao".into()),
            destinos: vec![Destino {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Unidade de testes".into(),
            }],
        },
    )
    .unwrap();
    banco
}

/// Requisição HTTP real, sem rede externa. A conexão SQLite volta da thread
/// para que o teste compare o estado antes/depois de cada operação.
fn pedir(banco: Connection, metodo: &str, alvo: &str, corpo: &str) -> (Connection, String) {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let (mut fluxo, _) = escuta.accept().unwrap();
        let instalacao = origem();
        atender_http(&mut fluxo, Some((&banco, &instalacao, CENARIO))).unwrap();
        banco
    });
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
    (servidor.join().unwrap(), resposta)
}

#[test]
#[expect(clippy::too_many_lines, reason = "um ciclo HTTP e SQLite de ponta a ponta para tramitação")]
fn pesquisa_abre_a_unidade_ativa_e_preserva_o_estado_ate_o_post_unico() {
    let mut banco = preparar();
    let antes = ler(&banco, &origem(), CENARIO, "10").unwrap().unwrap();
    let (novo, pesquisa) = pedir(
        banco,
        "POST",
        "/sei/controlador.php?acao=protocolo_pesquisa_rapida",
        "txtPesquisaRapida=00001.000001%2F2026-01",
    );
    banco = novo;
    assert!(pesquisa.starts_with("HTTP/1.1 302 Found"));
    let location = pesquisa
        .lines()
        .find_map(|linha| linha.strip_prefix("Location: "))
        .unwrap()
        .trim_end_matches('\r');
    assert!(location.contains("id_protocolo=10"));
    let trabalho = format!("/sei/{location}");
    let (novo, pagina) = pedir(banco, "GET", &trabalho, "");
    banco = novo;
    assert!(pagina.starts_with("HTTP/1.1 200 OK"));
    assert!(pagina.contains("<a id='lnkInfraUnidade'>ORIGEM</a>"));
    assert!(pagina.contains("id='ifrVisualizacao'"));
    let (novo, arvore) = pedir(
        banco,
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10",
        "",
    );
    banco = novo;
    assert!(arvore.contains("acao=procedimento_enviar"));
    assert_eq!(
        ler(&banco, &origem(), CENARIO, "10").unwrap().unwrap(),
        antes
    );

    // Exatamente um POST mutador; a consulta seguinte é independente.
    let (novo, envio) = pedir(
        banco,
        "POST",
        "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10",
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO",
    );
    banco = novo;
    assert!(envio.starts_with("HTTP/1.1 302 Found"));
    assert_eq!(
        ler(&banco, &origem(), CENARIO, "10")
            .unwrap()
            .unwrap()
            .aberto_em,
        ["DESTINO"]
    );

    controle(&mut banco, "DESTINO");
    let (novo, releitura) = pedir(banco, "GET", &trabalho, "");
    banco = novo;
    assert!(releitura.starts_with("HTTP/1.1 200 OK"));
    assert!(
        releitura.contains("<a id='lnkInfraUnidade'>DESTINO</a>"),
        "pesquisa reabriu tela documental com unidade de origem obsoleta"
    );
    assert_eq!(
        ler(&banco, &origem(), CENARIO, "10")
            .unwrap()
            .unwrap()
            .aberto_em,
        ["DESTINO"]
    );
}
