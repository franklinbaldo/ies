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
    let alvo = "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10";
    let formulario = anti_ies::processo_envio::responder("GET", alvo, "", &banco, &origem, cenario);
    assert_eq!(formulario.estado, anti_ies::Estado::Ok);
    let tela = conferir_tela_de_envio(&formulario.corpo, id).unwrap();
    assert!(tela.remover_anotacoes_oferecida);
    let servida = TelaServida {
        url: "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10",
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
