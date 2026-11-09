use super::*;
use iesei::banco_cassettes::iniciar;
use iesei::cancelamento_documento::{Aplicacao, aplicar};
use iesei::documento_versoes::Revisao;
use std::num::NonZeroU32;

/// Um documento recém-criado mantém um nó único quando ganha revisão.
/// A presença de uma identidade no SQLite não equivale a uma revisão.
#[test]
fn criar_depois_versionar_mantem_um_no_sem_revisao_inventada() {
    use iesei::documentos_simulados::{self, Processo, Tipo};

    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    let cenario = "criar-depois-versionar";
    documentos_simulados::registrar_processo(
        &banco,
        &origem,
        cenario,
        &Processo {
            id: "10".into(),
            nup: "00001.000001/2026-01".into(),
            unidade: "ORIGEM".into(),
        },
    )
    .unwrap();
    documentos_simulados::registrar_tipo(
        &banco,
        &origem,
        cenario,
        "10",
        &Tipo {
            id: "291".into(),
            nome: "Despacho".into(),
        },
    )
    .unwrap();
    let criado = documentos_simulados::criar(&banco, &origem, cenario, "10", "291", "").unwrap();
    let sem_revisao = nos_do_processo(&banco, &origem, cenario, "10").unwrap();
    assert_eq!(sem_revisao.matches("new infraArvoreNo").count(), 1);
    assert_eq!(sem_revisao.matches("Versões").count(), 0);
    assert!(sem_revisao.contains("Cancelar Documento"));

    versoes::guardar(
        &mut banco,
        cenario,
        &DocumentoSimulado {
            instalacao: origem.clone(),
            id_documento: criado.id.to_string(),
            id_procedimento: "10".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: criado.numero_sei,
            tipo: criado.tipo,
            revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "TESTE".into(),
                unidade: "ORIGEM".into(),
                modificacao: "09/11/2026".into(),
                html_corpo: "<p>Conteúdo sintético</p>".into(),
            }],
        },
    )
    .unwrap();
    let com_revisao = nos_do_processo(&banco, &origem, cenario, "10").unwrap();
    assert_eq!(com_revisao.matches("new infraArvoreNo").count(), 1);
    assert_eq!(com_revisao.matches("Versões").count(), 1);
    assert_eq!(com_revisao.matches("Cancelar Documento").count(), 1);
}

/// Regressao tecnica do BDD-273-02: as duas arvores leem o mesmo
/// cancelamento persistido, sem perder os outros documentos.
#[test]
fn cancelamento_altera_icone_na_arvore_mista_sem_apagar_outros_documentos() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    let cenario = "bdd-273";
    for (id, numero) in [("42", "0011519"), ("43", "0011520")] {
        versoes::guardar(
            &mut banco,
            cenario,
            &DocumentoSimulado {
                instalacao: origem.clone(),
                id_documento: id.into(),
                id_procedimento: "10".into(),
                numero_processo: "00001.000001/2026-01".into(),
                numero: numero.into(),
                tipo: "Despacho".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "Teste".into(),
                    unidade: "ORIGEM".into(),
                    modificacao: "08/11/2026".into(),
                    html_corpo: "<p>Original</p>".into(),
                }],
            },
        )
        .unwrap();
    }
    let antes = nos_do_processo(&banco, &origem, cenario, "10").unwrap();
    assert_eq!(antes.matches("documento_cancelado.svg").count(), 0);
    assert_eq!(
        aplicar(
            &banco,
            &origem,
            cenario,
            "42",
            "10",
            "Motivo do cancelamento"
        )
        .unwrap(),
        Aplicacao::Aplicada
    );
    let depois = nos_do_processo(&banco, &origem, cenario, "10").unwrap();
    assert_eq!(depois.matches("documento_cancelado.svg").count(), 3);
    assert_eq!(depois.matches("documento_interno.svg").count(), 3);
    assert!(depois.contains("Motivo do cancelamento"));
    assert!(depois.contains("0011519"));
    assert!(depois.contains("0011520"));
    let individual = responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=42",
        &banco,
        &origem,
        cenario,
    );
    assert!(individual.corpo.contains("documento_cancelado.svg"));
}
