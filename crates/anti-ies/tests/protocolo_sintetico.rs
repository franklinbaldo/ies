//! Regressão de apresentação da CLI: URLs do mesmo cenário usam uma única
//! assinatura sintética; nunca é uma credencial da instalação real.
use anti_ies::{Estado, controlar, processo_envio};
use iesei::banco_cassettes::{ControleResumido, guardar as guardar_controle, iniciar};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::processos::{ProcessoSimulado, guardar};
use rusqlite::Connection;

#[test]
fn controle_e_barra_do_processo_usam_mesma_assinatura_sintetica() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let instalacao = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
    let cenario = "assinatura-compartilhada";
    guardar_controle(
        &mut banco,
        cenario,
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("ORIGEM").unwrap(),
            recebidos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    guardar(
        &mut banco,
        &instalacao,
        cenario,
        &ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();

    let controle = controlar(
        "GET",
        "/sei/controlador.php?acao=procedimento_controlar",
        &banco,
        &instalacao,
        cenario,
    );
    let arvore = processo_envio::navegacao(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10",
        &banco,
        &instalacao,
        cenario,
    )
    .unwrap();
    assert_eq!(controle.estado, Estado::Ok);
    assert_eq!(arvore.estado, Estado::Ok);
    let marcador = format!("infra_hash={}", "4".repeat(64));
    assert!(controle.corpo.contains(&marcador), "formulário de pesquisa");
    assert!(arvore.corpo.contains(&marcador), "barra do processo");
}
