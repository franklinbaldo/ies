//! BDD-333-03/04: transições de um processo único no SQLite efêmero.
//! Provas sintéticas; não implicam equivalência com o SEI real.
use anti_ies::{Estado, processo_envio::responder};
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use iesei::processos::{Destino, ProcessoSimulado, guardar, ler};
use rusqlite::Connection;

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn preparar() -> Connection {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(
        &mut banco,
        &origem(),
        "cenario-a",
        &ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: Some("Anotação temporária".into()),
            destinos: vec![Destino {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Unidade de testes".into(),
            }],
        },
    ).unwrap();
    banco
}

const ENDERECO: &str = "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10";

#[test]
fn previa_e_confirmacao_reconciliam_o_mesmo_estado_sqlite() {
    let mut banco = preparar();
    let antes = ler(&banco, &origem(), "cenario-a", "10").unwrap().unwrap();
    let previa = responder("GET", ENDERECO, "", &mut banco, &origem(), "cenario-a");
    assert_eq!(previa.estado, Estado::Ok);
    assert!(previa.corpo.contains("frmAtividadeListar"));
    assert!(previa.corpo.contains("chkSinRemoverAnotacoes"));
    assert_eq!(ler(&banco, &origem(), "cenario-a", "10").unwrap().unwrap(), antes);

    let envio = responder(
        "POST", ENDERECO,
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO&chkSinManterAberto=on&chkSinRemoverAnotacoes=on",
        &mut banco, &origem(), "cenario-a",
    );
    assert_eq!(envio.estado, Estado::Redirecionado);
    assert_eq!(
        envio.localizacao.as_deref(),
        Some("controlador.php?acao=arvore_visualizar&acao_origem=procedimento_enviar&id_procedimento=10")
    );
    let depois = ler(&banco, &origem(), "cenario-a", "10").unwrap().unwrap();
    assert_eq!(depois.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some(""));
    assert_eq!(depois.destinos, antes.destinos);
}

#[test]
fn envio_sem_manter_aberto_fecha_origem_e_preserva_anotacao() {
    let mut banco = preparar();
    let r = responder(
        "POST", ENDERECO, "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO",
        &mut banco, &origem(), "cenario-a",
    );
    assert_eq!(r.estado, Estado::Redirecionado);
    let depois = ler(&banco, &origem(), "cenario-a", "10").unwrap().unwrap();
    assert_eq!(depois.aberto_em, ["DESTINO"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some("Anotação temporária"));
}

#[test]
fn destino_invalido_e_duplicado_nao_modificam_o_banco() {
    let mut banco = preparar();
    let antes = ler(&banco, &origem(), "cenario-a", "10").unwrap();
    for corpo in [
        "hdnIdProtocolos=10&hdnUnidades=999%B1FALSO",
        "hdnIdProtocolos=99&hdnUnidades=2%B1DESTINO",
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO&hdnUnidades=2%B1DESTINO",
    ] {
        let r = responder("POST", ENDERECO, corpo, &mut banco, &origem(), "cenario-a");
        assert_eq!(r.estado, Estado::RequisicaoInvalida, "{corpo}");
        assert_eq!(ler(&banco, &origem(), "cenario-a", "10").unwrap(), antes);
    }
}

#[test]
fn mesmo_id_em_outro_cenario_nao_acessa_ou_muta_estado() {
    let mut banco = preparar();
    let r = responder(
        "POST", ENDERECO, "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO",
        &mut banco, &origem(), "outro-cenario",
    );
    assert_eq!(r.estado, Estado::DadosInsuficientes);
    assert_eq!(
        ler(&banco, &origem(), "cenario-a", "10").unwrap().unwrap().aberto_em,
        ["ORIGEM"]
    );
}
