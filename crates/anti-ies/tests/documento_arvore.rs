//! #88 BDD-88-01/03: a arvore e as versoes consultam o mesmo documento do SQLite.
use anti_ies::{Estado, documento_versoes::responder};
use iesei::banco_cassettes::iniciar;
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::num::NonZeroU32;

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn documento() -> DocumentoSimulado {
    DocumentoSimulado {
        instalacao: origem(),
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
                html_corpo: "<p>Primeira redação</p>".into(),
            },
            Revisao {
                versao: NonZeroU32::new(3).unwrap(),
                usuario: "Bia".into(),
                unidade: "PPI".into(),
                modificacao: "03/11/2026".into(),
                html_corpo: "<p>Terceira redação</p>".into(),
            },
        ],
    }
}

#[test]
fn arvore_publica_links_que_abrem_historico_e_conteudo_do_mesmo_documento() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(&mut banco, "ensaio", &documento()).unwrap();

    let arvore = responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7&id_documento=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(arvore.estado, Estado::Ok);
    assert!(arvore.corpo.contains("Nos[0] = new infraArvoreNo"));
    assert!(arvore.corpo.contains("Nos[1] = new infraArvoreNo"));
    assert!(arvore.corpo.contains("documento_versao_listar"));
    assert!(arvore.corpo.contains("acao=documento_visualizar"));

    let tela = responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_trabalhar&id_protocolo=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(tela.estado, Estado::Ok);
    assert!(tela.corpo.contains("id=\"ifrArvore\""));
    assert!(tela.corpo.contains("id_procedimento=7"));
    let historico = responder(
        "GET",
        "/sei/controlador.php?acao=documento_versao_listar&id_documento=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(historico.estado, Estado::Ok);
    assert!(historico.corpo.contains("Lista de Versões (2 registros)"));
    let antiga = responder(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=1",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(antiga.estado, Estado::Ok);
    assert!(antiga.corpo.contains("Primeira redação"));
    assert!(!antiga.corpo.contains("Terceira redação"));

    let outro = responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=99&id_documento=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(outro.estado, Estado::NaoEncontrado);
}
