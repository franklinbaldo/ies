//! BDD-333-01/02: Controle R precisa refletir o SQLite após tramitação.
//! Hipótese sintética revisável; não comprova compatibilidade externa.
use anti_ies::{Estado, controlar, processo_envio::responder};
use iesei::banco_cassettes::{ControleResumido, guardar as guardar_controle, iniciar};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::processos::{Destino, ProcessoSimulado, guardar, ler};
use rusqlite::Connection;

const CENARIO: &str = "bdd-333-controle";
const NUP: &str = "00001.000001/2026-01";
const CONTROLE: &str = "/sei/controlador.php?acao=procedimento_controlar";
const ENVIO: &str = "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10";

fn origem() -> Instalacao {
    Instalacao::interpretar("https://simulado.invalid/sei").unwrap()
}

fn observar_unidade(banco: &mut Connection, unidade: &str) {
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
    observar_unidade(&mut banco, "ORIGEM");
    guardar(
        &mut banco,
        &origem(),
        CENARIO,
        &ProcessoSimulado {
            id: "10".into(),
            numero: NUP.into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: Some("Anotação original".into()),
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

fn tela(banco: &Connection) -> String {
    let resposta = controlar("GET", CONTROLE, banco, &origem(), CENARIO);
    assert_eq!(resposta.estado, Estado::Ok);
    resposta.corpo
}

#[test]
fn controle_r_reflete_abertura_e_fechamento_apos_post_unico() {
    let mut banco = preparar();
    let antes = ler(&banco, &origem(), CENARIO, "10").unwrap().unwrap();
    assert_eq!(antes.aberto_em, ["ORIGEM"]);
    assert!(
        tela(&banco).contains(NUP),
        "controle inicial ocultou processo aberto"
    );
    // A prévia não escreve no banco nem muda o Controle R.
    let previa = responder("GET", ENVIO, "", &banco, &origem(), CENARIO);
    assert_eq!(previa.estado, Estado::Ok);
    assert_eq!(
        ler(&banco, &origem(), CENARIO, "10").unwrap().unwrap(),
        antes
    );
    assert!(tela(&banco).contains(NUP));

    let envio = responder(
        "POST",
        ENVIO,
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO",
        &banco,
        &origem(),
        CENARIO,
    );
    assert_eq!(envio.estado, Estado::Redirecionamento);
    assert_eq!(
        ler(&banco, &origem(), CENARIO, "10")
            .unwrap()
            .unwrap()
            .aberto_em,
        ["DESTINO"]
    );
    assert!(
        !tela(&banco).contains(NUP),
        "origem fechada continua no Controle R"
    );
    // Trocar a unidade ativa lê o MESMO processo, sem copiá-lo para outra base.
    observar_unidade(&mut banco, "DESTINO");
    assert!(
        tela(&banco).contains(NUP),
        "destino aberto não aparece no Controle R"
    );
}

#[test]
fn manter_aberto_preserva_o_processo_no_controle_de_ambas_as_unidades() {
    let mut banco = preparar();
    let envio = responder(
        "POST",
        ENVIO,
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO&chkSinManterAberto=on",
        &banco,
        &origem(),
        CENARIO,
    );
    assert_eq!(envio.estado, Estado::Redirecionamento);
    assert!(tela(&banco).contains(NUP));
    observar_unidade(&mut banco, "DESTINO");
    assert!(tela(&banco).contains(NUP));
    assert_eq!(
        ler(&banco, &origem(), CENARIO, "10")
            .unwrap()
            .unwrap()
            .aberto_em,
        ["ORIGEM", "DESTINO"]
    );
}
