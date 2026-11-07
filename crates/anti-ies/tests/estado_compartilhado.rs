//! As duas operações já entregues devem consultar um único estado.
use anti_ies::{Estado, controlar, retorno_programado};
use iesei::banco_cassettes::{ControleResumido, Linha, guardar as guardar_controle, iniciar};
use iesei::instalacao::Instalacao;
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use iesei::retornos_cassettes::{Mes, Retorno, Situacao, guardar as guardar_retornos};
use rusqlite::Connection;

fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

#[test]
fn duas_operacoes_refletem_uma_mesma_base_reconstruivel() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = instalacao();
    let cenario = "bancada";

    let mut controle = ControleResumido {
        instalacao: origem.clone(),
        unidade: UnidadeExibida::nova("UNIDADE TESTE").unwrap(),
        recebidos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
    };
    guardar_controle(&mut banco, cenario, &controle).unwrap();
    let endereco_controle = "/sei/controlador.php?acao=procedimento_controlar";
    let primeira = controlar("GET", endereco_controle, &banco, &origem, cenario);
    assert_eq!(primeira.estado, Estado::Ok);
    assert!(primeira.corpo.contains("<caption>0 registros:</caption>"));

    let mut mes = Mes {
        instalacao: origem.clone(),
        data_inicial: "01".into(),
        data_final: "31".into(),
        itens: vec![Retorno {
            situacao: Situacao::Aguardando,
            numero: "NUP-1".into(),
            unidade: "UNIDADE TESTE".into(),
            data_programada: "15/10/2026".into(),
            retornado_em: None,
            prazo_restante: None,
        }],
    };
    guardar_retornos(&mut banco, cenario, &mes).unwrap();
    let endereco_retornos =
        "/sei/controlador.php?acao=retorno_programado_listar&data_inicial=01&data_final=31";
    let retorno = retorno_programado("GET", endereco_retornos, &banco, &origem, cenario);
    assert_eq!(retorno.estado, Estado::Ok);
    assert!(retorno.corpo.contains("NUP-1"));
    assert_eq!(controlar("GET", endereco_controle, &banco, &origem, cenario), primeira);

    // Alterações sintéticas na fonte derivada tornam-se visíveis sem editar
    // os geradores HTML. Isto não simula nem comprova um POST real do SEI.
    controle.recebidos = TabelaObservada::nova(
        Some(1),
        FaixaObservada::nova(1, 1),
        vec![Linha {
            id: "42".into(),
            numero: "PROCESSO-42".into(),
            visualizado: true,
            sigiloso: false,
        }],
    ).unwrap();
    guardar_controle(&mut banco, cenario, &controle).unwrap();
    let atualizado = controlar("GET", endereco_controle, &banco, &origem, cenario);
    assert_eq!(atualizado.estado, Estado::Ok);
    assert!(atualizado.corpo.contains("PROCESSO-42"));
    assert!(!primeira.corpo.contains("PROCESSO-42"));

    mes.itens[0].numero = "NUP-2".into();
    guardar_retornos(&mut banco, cenario, &mes).unwrap();
    let atualizado = retorno_programado("GET", endereco_retornos, &banco, &origem, cenario);
    assert_eq!(atualizado.estado, Estado::Ok);
    assert!(atualizado.corpo.contains("NUP-2"));
    assert!(!atualizado.corpo.contains("NUP-1"));

    // Outro cenário não herda estado nem cria vazio fictício.
    assert_eq!(
        controlar("GET", endereco_controle, &banco, &origem, "outro").estado,
        Estado::DadosInsuficientes
    );
    assert_eq!(
        retorno_programado("GET", endereco_retornos, &banco, &origem, "outro").estado,
        Estado::DadosInsuficientes
    );

    let nova_base = Connection::open_in_memory().unwrap();
    iniciar(&nova_base).unwrap();
    assert_eq!(
        controlar("GET", endereco_controle, &nova_base, &origem, cenario).estado,
        Estado::DadosInsuficientes
    );
}
