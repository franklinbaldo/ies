//! Integração sintética: formulário e dados são produzidos pelo anti-IES;
//! o consumidor da submissão, do autocomplete e da árvore é o IES real.
use super::destino::{escolher_destino, itens_do_autocompletar};
use super::tela::{
    AUTOCOMPLETAR_UNIDADE, ENVIAR_PROCESSO, campos_da_consulta, campos_do_envio,
    conferir_tela_de_envio, redirecionamento_esperado,
};
use crate::arvore_processo::{AbertoEm, ler_arvore};
use crate::formulario_web::{
    CodificacaoDaTela, TelaServida, derivar_consulta_ajax, derivar_submissao,
};
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao as OrigemSimulada;
use iesei::processos::{Destino, ProcessoSimulado, guardar, ler};
use rusqlite::Connection;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "um único ciclo SQLite, leitura da árvore, POST e releitura usando os consumidores reais"
)]
fn formulario_real_do_ies_tramita_e_confere_estado_do_anti_ies() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = OrigemSimulada::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let instalacao =
        crate::instalacao::Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let cenario = "bdd-333-envio";
    let id = "10";
    let numero = "00001.000001/2026-01";
    guardar(
        &mut banco,
        &origem,
        cenario,
        &ProcessoSimulado {
            id: id.into(),
            numero: numero.into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: Some("Anotação temporária".into()),
            destinos: vec![Destino {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Unidade de testes".into(),
            }],
        },
    )
    .unwrap();
    let antes = ler(&banco, &origem, cenario, id).unwrap().unwrap();
    // BDD-333-03/05: o fluxo real só descobre envio e anotação pela
    // árvore servida, não por caminhos montados pelo teste.
    let arvore_antes = anti_ies::processo_envio::navegacao(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10",
        &banco,
        &origem,
        cenario,
    )
    .unwrap()
    .corpo;
    let envio_publicado =
        crate::arvore_processo::ler_enviar_da_barra(&instalacao, &arvore_antes, id)
            .unwrap()
            .expect("o anti-IES deve publicar Enviar na barra");
    assert!(
        envio_publicado
            .as_str()
            .contains("acao=procedimento_enviar")
    );
    let anotacao_publicada =
        crate::arvore_processo::ler_link_das_anotacoes(&instalacao, &arvore_antes, id)
            .unwrap()
            .expect("anotação simulada deve oferecer consulta/edição");
    assert!(anotacao_publicada.contains("acao=anotacao_registrar"));
    let alvo = format!(
        "/sei/{}",
        envio_publicado.as_str().split_once("/sei/").unwrap().1
    );
    let caminho_anotacao = format!("/sei/{}", anotacao_publicada.split_once("/sei/").unwrap().1);
    let anotacao_antes =
        anti_ies::processo_envio::responder("GET", &caminho_anotacao, "", &banco, &origem, cenario);
    assert!(super::anotacoes::anotacao_presente(&anotacao_antes.corpo, id).unwrap());
    let formulario =
        anti_ies::processo_envio::responder("GET", &alvo, "", &banco, &origem, cenario);
    assert_eq!(formulario.estado, anti_ies::Estado::Ok);
    let tela = conferir_tela_de_envio(&formulario.corpo, id).unwrap();
    assert!(tela.remover_anotacoes_oferecida);
    let servida = TelaServida {
        url: envio_publicado.as_str(),
        html: &formulario.corpo,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let consulta = derivar_consulta_ajax(
        &AUTOCOMPLETAR_UNIDADE,
        &instalacao,
        &servida,
        &tela.autocompletar,
        &campos_da_consulta(&tela, "DESTINO"),
    )
    .unwrap();
    let caminho_ajax = format!("/sei/{}", consulta.url().split_once("/sei/").unwrap().1);
    let corpo_ajax = String::from_utf8(consulta.corpo().to_vec()).unwrap();
    let resposta_ajax = anti_ies::processo_envio::autocompletar(
        "POST",
        &caminho_ajax,
        &corpo_ajax,
        &banco,
        &origem,
        cenario,
    );
    let itens = itens_do_autocompletar(&resposta_ajax.corpo).unwrap();
    let escolhido = escolher_destino("DESTINO", &itens).unwrap();
    let campos = campos_do_envio(&tela, &escolhido, true, true);
    let pares: Vec<_> = campos.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let submissao = derivar_submissao(&ENVIAR_PROCESSO, &instalacao, &servida, "", &pares).unwrap();
    let caminho = format!("/sei/{}", submissao.url().split_once("/sei/").unwrap().1);
    assert_eq!(ler(&banco, &origem, cenario, id).unwrap().unwrap(), antes);
    let corpo = String::from_utf8(submissao.corpo().to_vec()).unwrap();
    let post =
        anti_ies::processo_envio::responder("POST", &caminho, &corpo, &banco, &origem, cenario);
    assert_eq!(post.estado, anti_ies::Estado::Redirecionamento);
    assert!(redirecionamento_esperado(
        post.localizacao.as_deref(),
        submissao.url(),
        id
    ));
    let depois = ler(&banco, &origem, cenario, id).unwrap().unwrap();
    assert_eq!(depois.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some(""));
    let anotacao_depois =
        anti_ies::processo_envio::responder("GET", &caminho_anotacao, "", &banco, &origem, cenario);
    assert!(!super::anotacoes::anotacao_presente(&anotacao_depois.corpo, id).unwrap());
    let html_arvore = anti_ies::processo_envio::navegacao(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10",
        &banco,
        &origem,
        cenario,
    )
    .unwrap()
    .corpo;
    let arvore = ler_arvore(&html_arvore, numero, id).unwrap();
    let AbertoEm::Unidades(unidades) = arvore.aberto_em else {
        panic!("arvore nao refletiu as unidades persistidas");
    };
    assert_eq!(
        unidades.into_iter().map(|u| u.sigla).collect::<Vec<_>>(),
        ["ORIGEM", "DESTINO"]
    );
}

#[test]
fn barra_da_origem_fechada_nao_oferece_novo_envio_ou_anotacao_nao_observada() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = OrigemSimulada::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let instalacao =
        crate::instalacao::Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    guardar(
        &mut banco,
        &origem,
        "processo-fechado",
        &ProcessoSimulado {
            id: "20".into(),
            numero: "00001.000002/2026-01".into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["DESTINO".into()],
            anotacao_origem: None,
            destinos: vec![Destino {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Unidade de testes".into(),
            }],
        },
    )
    .unwrap();
    let arvore = anti_ies::processo_envio::navegacao(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=20",
        &banco,
        &origem,
        "processo-fechado",
    )
    .unwrap()
    .corpo;
    assert!(
        crate::arvore_processo::ler_enviar_da_barra(&instalacao, &arvore, "20")
            .unwrap()
            .is_none()
    );
    assert!(
        crate::arvore_processo::ler_link_das_anotacoes(&instalacao, &arvore, "20")
            .unwrap()
            .is_none()
    );
}

#[test]
fn leitor_real_da_caixa_reflete_tramitacao_no_controle_r_sintetico() {
    use iesei::banco_cassettes::{ControleResumido, guardar as guardar_controle};
    use iesei::observacao::{TabelaObservada, UnidadeExibida};

    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = OrigemSimulada::interpretar("https://simulado.invalid/sei").unwrap();
    let cenario = "bdd-333-controle-ies";
    let controle = |unidade: &str| ControleResumido {
        instalacao: origem.clone(),
        unidade: UnidadeExibida::nova(unidade).unwrap(),
        recebidos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
    };
    guardar_controle(&mut banco, cenario, &controle("ORIGEM")).unwrap();
    guardar(
        &mut banco,
        &origem,
        cenario,
        &ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: None,
            destinos: vec![Destino {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Unidade de testes".into(),
            }],
        },
    )
    .unwrap();

    let consultar = |banco: &Connection| {
        let resposta = anti_ies::controlar(
            "GET",
            "/sei/controlador.php?acao=procedimento_controlar",
            banco,
            &origem,
            cenario,
        );
        assert_eq!(resposta.estado, anti_ies::Estado::Ok);
        crate::caixa::parse_html(&resposta.corpo).unwrap()
    };
    let antes = consultar(&banco);
    assert_eq!(antes.recebidos.processos.len(), 1);
    assert_eq!(antes.recebidos.processos[0].numero, "00001.000001/2026-01");

    // POST mutador único; o próximo GET tem de refletir o estado persistido.
    let post = anti_ies::processo_envio::responder(
        "POST",
        "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10",
        "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO",
        &banco,
        &origem,
        cenario,
    );
    assert_eq!(post.estado, anti_ies::Estado::Redirecionamento);
    assert_eq!(consultar(&banco).recebidos.processos, Vec::new());

    guardar_controle(&mut banco, cenario, &controle("DESTINO")).unwrap();
    let destino = consultar(&banco);
    assert_eq!(destino.recebidos.processos.len(), 1);
    assert_eq!(
        destino.recebidos.processos[0].numero,
        "00001.000001/2026-01"
    );
}

#[test]
fn leitor_ies_descobre_nova_tramitacao_depois_de_trocar_a_unidade_ativa() {
    use iesei::banco_cassettes::{ControleResumido, guardar as guardar_controle};
    use iesei::observacao::{TabelaObservada, UnidadeExibida};

    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = OrigemSimulada::interpretar("https://simulado.invalid/sei").unwrap();
    let instalacao =
        crate::instalacao::Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
    let cenario = "bdd-333-segunda-unidade-ies";
    let nup = "00001.000001/2026-01";
    let controle = |unidade: &str| ControleResumido {
        instalacao: origem.clone(),
        unidade: UnidadeExibida::nova(unidade).unwrap(),
        recebidos: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
        gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
    };
    guardar_controle(&mut banco, cenario, &controle("ORIGEM")).unwrap();
    guardar(
        &mut banco,
        &origem,
        cenario,
        &ProcessoSimulado {
            id: "10".into(),
            numero: nup.into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: Some("Nota restrita à origem".into()),
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
    let caminho_arvore = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10";
    let caminho_envio = "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10";
    let arvore = |banco: &Connection| {
        anti_ies::processo_envio::navegacao(
            "GET", caminho_arvore, banco, &origem, cenario,
        )
        .unwrap()
        .corpo
    };
    assert!(
        crate::arvore_processo::ler_enviar_da_barra(&instalacao, &arvore(&banco), "10")
            .unwrap()
            .is_some()
    );
    let primeiro = anti_ies::processo_envio::responder(
        "POST", caminho_envio, "hdnIdProtocolos=10&hdnUnidades=2%B1DESTINO",
        &banco, &origem, cenario,
    );
    assert_eq!(primeiro.estado, anti_ies::Estado::Redirecionamento);
    guardar_controle(&mut banco, cenario, &controle("DESTINO")).unwrap();

    let segunda_arvore = arvore(&banco);
    let situacao = ler_arvore(&segunda_arvore, nup, "10").unwrap();
    let AbertoEm::Unidades(unidades) = situacao.aberto_em else {
        panic!("releitura do IES não reconheceu a nova unidade");
    };
    assert_eq!(unidades[0].sigla, "DESTINO");
    assert_eq!(unidades.len(), 1);
    assert!(
        crate::arvore_processo::ler_enviar_da_barra(&instalacao, &segunda_arvore, "10")
            .unwrap()
            .is_some()
    );
    assert!(
        crate::arvore_processo::ler_link_das_anotacoes(&instalacao, &segunda_arvore, "10")
            .unwrap()
            .is_none()
    );
    let segundo = anti_ies::processo_envio::responder(
        "POST", caminho_envio, "hdnIdProtocolos=10&hdnUnidades=3%B1ARQUIVO",
        &banco, &origem, cenario,
    );
    assert_eq!(segundo.estado, anti_ies::Estado::Redirecionamento);
    assert_eq!(
        ler(&banco, &origem, cenario, "10").unwrap().unwrap().aberto_em,
        ["ARQUIVO"]
    );
}
