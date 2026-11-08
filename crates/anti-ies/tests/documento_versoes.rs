//! BDD-88-01/03: versões são estado do anti-IES, não HTML fixo.
use anti_ies::Estado;
use anti_ies::documento_versoes::responder;
use iesei::banco_cassettes::iniciar;
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar, ler};
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
        numero: "0011519".into(),
        tipo: "Ofício".into(),
        revisoes: vec![
            Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "Ana".into(),
                unidade: "PPI".into(),
                modificacao: "01/11/2026".into(),
                html_corpo: "<p>Texto histórico</p>".into(),
            },
            Revisao {
                versao: NonZeroU32::new(3).unwrap(),
                usuario: "Bia".into(),
                unidade: "PPI".into(),
                modificacao: "03/11/2026".into(),
                html_corpo: "<p>Texto atual</p>".into(),
            },
        ],
    }
}

#[test]
fn bdd_88_01_versao_nao_contigua_tem_conteudo_proprio() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(&mut banco, "ensaio", &documento()).unwrap();
    let lista = responder(
        "GET",
        "/sei/controlador.php?acao=documento_versao_listar&id_documento=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(lista.estado, Estado::Ok);
    assert!(lista.corpo.contains("Lista de Versões (2 registros)"));
    let antiga = responder(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=1",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(antiga.estado, Estado::Ok);
    assert!(antiga.corpo.contains("Texto histórico"));
    assert!(!antiga.corpo.contains("Texto atual"));
    assert!(antiga.corpo.contains("versão 1 por"));
    let atual = responder(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert!(atual.corpo.contains("Texto atual"));
    assert!(!atual.corpo.contains("Texto histórico"));
    assert!(atual.corpo.contains("versão 3 por"));
    assert_eq!(
        responder(
            "GET",
            "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=2",
            &banco,
            &origem(),
            "ensaio"
        )
        .estado,
        Estado::NaoEncontrado
    );
}

#[test]
fn bdd_88_03_ausencia_nao_e_versao_zero_nem_lista_vazia() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let alvo = "/sei/controlador.php?acao=documento_versao_listar&id_documento=42";
    assert_eq!(
        responder("GET", alvo, &banco, &origem(), "ensaio").estado,
        Estado::DadosInsuficientes
    );
    assert_eq!(
        responder("POST", alvo, &banco, &origem(), "ensaio").estado,
        Estado::MetodoNaoPermitido
    );
    assert_eq!(
        responder(
            "GET",
            "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=0",
            &banco,
            &origem(),
            "ensaio"
        )
        .estado,
        Estado::RequisicaoInvalida
    );
}

#[test]
fn estado_sqlite_compartilhado_idempotente_e_isolado() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let original = documento();
    guardar(&mut banco, "ensaio", &original).unwrap();
    guardar(&mut banco, "ensaio", &original).unwrap();
    assert_eq!(
        ler(&banco, &origem(), "ensaio", "42").unwrap(),
        Some(original.clone())
    );
    assert!(ler(&banco, &origem(), "outro", "42").unwrap().is_none());
    let outra = Instalacao::interpretar("https://outra.exemplo/sei").unwrap();
    assert!(ler(&banco, &outra, "ensaio", "42").unwrap().is_none());
    let count: i64 = banco
        .query_row("SELECT count(*) FROM documento_revisao", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 2);
    let mut modificado = original;
    modificado.revisoes.pop();
    guardar(&mut banco, "ensaio", &modificado).unwrap();
    assert_eq!(
        ler(&banco, &origem(), "ensaio", "42").unwrap(),
        Some(modificado)
    );
    let count: i64 = banco
        .query_row("SELECT count(*) FROM documento_revisao", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    let vazio = Connection::open_in_memory().unwrap();
    iniciar(&vazio).unwrap();
    assert!(ler(&vazio, &origem(), "ensaio", "42").unwrap().is_none());
}

#[test]
fn estado_invalido_nao_grava_meia_revisao() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let mut invalido = documento();
    invalido.revisoes[1].versao = invalido.revisoes[0].versao;
    assert!(guardar(&mut banco, "ensaio", &invalido).is_err());
    assert!(ler(&banco, &origem(), "ensaio", "42").unwrap().is_none());
}
