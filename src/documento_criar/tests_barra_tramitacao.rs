//! Teste técnico da fronteira IES/anti-IES para BDD-246-01/03.
//! O mesmo processo muda de estado; o parser IES lê a barra sem inventar link.
use crate::arvore_processo::ler_link_de_incluir_documento;
use anti_ies::processo_envio::navegacao;
use iesei::banco_cassettes::iniciar;
use iesei::processos::{Destino, PedidoEnvio, ProcessoSimulado, enviar, guardar};
use rusqlite::Connection;

#[test]
fn leitor_ies_respeita_oferta_dinamica_apos_tramitacao_simulada() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = iesei::instalacao::Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
    let instalacao = crate::instalacao::Instalacao::interpretar("https://simulado.invalid/sei")
        .unwrap();
    guardar(
        &mut banco,
        &origem,
        "bdd-246",
        &ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![Destino {
                id: "20".into(),
                sigla: "DESTINO".into(),
                descricao: "Unidade de destino".into(),
            }],
        },
    )
    .unwrap();
    let alvo = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
    let arvore = |banco: &Connection| {
        navegacao("GET", alvo, banco, &origem, "bdd-246")
            .unwrap()
            .corpo
    };
    let antes = arvore(&banco);
    assert!(
        ler_link_de_incluir_documento(&instalacao, &antes, "10")
            .unwrap()
            .is_some()
    );

    enviar(
        &banco,
        &origem,
        "bdd-246",
        PedidoEnvio {
            id: "10",
            id_destino: "20",
            manter_aberto: false,
            remover_anotacoes: false,
        },
    )
    .unwrap();
    let depois = arvore(&banco);
    assert!(
        ler_link_de_incluir_documento(&instalacao, &depois, "10")
            .unwrap()
            .is_none(),
        "a CLI não pode oferecer criação na unidade já fechada"
    );
}
