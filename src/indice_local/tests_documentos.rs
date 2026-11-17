//! RED/GREEN da tabela `documento` e do aceite da entrada com documento
//! (Refs #60, fatia 2). Diretórios temporários, sem rede.
use super::tests::{instalacao, pasta};
use super::*;
use crate::instalacao::Instalacao;

fn indice(nome: &str) -> IndiceLocal {
    IndiceLocal::em(pasta(nome).join(NOME_DO_ARQUIVO))
}

fn guardado(processo: &str, documento: &str) -> DocumentoGuardado {
    DocumentoGuardado {
        id_procedimento: processo.to_owned(),
        id_documento: documento.to_owned(),
    }
}

fn linha(numero: &str, processo: &str, documento: &str) -> [String; 4] {
    [
        "https://sei.exemplo/sei".to_owned(),
        numero.to_owned(),
        processo.to_owned(),
        documento.to_owned(),
    ]
}

#[test]
fn numero_sei_opaco_aprendido_por_instalacao() {
    let indice = indice("doc-opaco");
    let inst = instalacao();
    indice.registrar_documentos(&inst, "10", &[("0000101", "21"), ("0000102", "22")]);
    assert_eq!(
        indice.documento_por_numero(&inst, "0000101"),
        Some(guardado("10", "21"))
    );
    // Zeros à esquerda contam: 101 não é 0000101.
    assert_eq!(indice.documento_por_numero(&inst, "101"), None);
    let outra = Instalacao::interpretar("https://outro.exemplo/sei").unwrap();
    indice.registrar_documentos(&outra, "90", &[("0000101", "91")]);
    assert_eq!(
        indice.documento_por_numero(&outra, "0000101"),
        Some(guardado("90", "91"))
    );
    assert_eq!(
        indice.documento_por_numero(&inst, "0000101"),
        Some(guardado("10", "21"))
    );
    assert_eq!(
        indice.documentos(),
        [
            [
                "https://outro.exemplo/sei".to_owned(),
                "0000101".to_owned(),
                "90".to_owned(),
                "91".to_owned(),
            ],
            linha("0000101", "10", "21"),
            linha("0000102", "10", "22"),
        ]
    );
}

#[test]
fn servidor_vence_numero_ou_documento_reaprendido_substitui() {
    let indice = indice("doc-conflito");
    let inst = instalacao();
    indice.registrar_documentos(&inst, "10", &[("0000101", "21"), ("0000102", "22")]);
    // O mesmo número noutro documento e o mesmo documento noutro processo.
    indice.registrar_documentos(&inst, "11", &[("0000101", "31"), ("0000109", "22")]);
    assert_eq!(
        indice.documentos(),
        [linha("0000101", "11", "31"), linha("0000109", "11", "22")]
    );
    indice.esquecer_documento(&inst, "0000101");
    assert_eq!(indice.documento_por_numero(&inst, "0000101"), None);
    assert_eq!(indice.documentos(), [linha("0000109", "11", "22")]);
}

#[test]
fn par_de_documento_contestado_pelo_servidor_nao_e_reutilizado() {
    let indice = indice("doc-contestado");
    let inst = instalacao();
    indice.registrar_documentos(&inst, "10", &[("0000101", "21")]);

    // O servidor respondeu com outro par para o mesmo número SEI.
    indice.registrar_documentos(&inst, "11", &[("0000101", "31")]);

    assert_eq!(
        indice.documento_por_numero(&inst, "0000101"),
        Some(guardado("11", "31"))
    );
    assert_eq!(indice.documentos(), [linha("0000101", "11", "31")]);
}

#[test]
fn repetidos_e_invalidos_da_mesma_arvore_sao_pulados() {
    let indice = indice("doc-repetido");
    let inst = instalacao();
    let documentos = [
        ("0000101", "21"),
        ("0000101", "22"),
        ("0000103", "23"),
        ("0000104", "23"),
        ("0000105", "2&x=1"),
        ("", "26"),
        ("00\n07", "27"),
        ("0000108", "28"),
    ];
    indice.registrar_documentos(&inst, "10", &documentos);
    assert_eq!(indice.documentos(), [linha("0000108", "10", "28")]);
    assert!(indice.avisos().is_empty());
    indice.registrar_documentos(&inst, "1&0", &[("0000109", "29")]);
    let avisos = indice.avisos();
    assert!(
        matches!(
            avisos.as_slice(),
            [FalhaDoIndice::RegistroRecusado(
                RegistroRecusado::IdProcedimentoInvalido
            )]
        ),
        "{avisos:?}"
    );
    assert_eq!(indice.documentos().len(), 1);
}

#[test]
fn nup_por_id_volta_o_par_guardado() {
    let indice = indice("doc-nup");
    let inst = instalacao();
    indice.registrar_processo(&inst, "N1", "10");
    assert_eq!(indice.nup_por_id(&inst, "10").as_deref(), Some("N1"));
    assert_eq!(indice.nup_por_id(&inst, "11"), None);
}

#[test]
fn aceite_da_forma_com_documento_e_contado_a_parte() {
    let indice = indice("doc-aceite");
    let inst = instalacao();
    indice.registrar_entrada_por_id(&inst, true);
    assert_eq!(indice.entrada_por_id_com_documento(&inst), None);
    indice.registrar_entrada_por_id_com_documento(&inst, false);
    assert_eq!(
        indice.entrada_por_id_com_documento(&inst),
        Some(AceiteDaEntrada::Recusada)
    );
    assert_eq!(indice.entrada_por_id(&inst), Some(AceiteDaEntrada::Aceita));
    indice.registrar_entrada_por_id_com_documento(&inst, true);
    assert_eq!(
        indice.entrada_por_id_com_documento(&inst),
        Some(AceiteDaEntrada::Aceita)
    );
}

/// Um banco da versão 1 (antes da fatia 2) migra sem perder linhas.
#[test]
fn banco_da_versao_1_migra_para_4_sem_perder_linhas() {
    let raiz = pasta("doc-migracao");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    IndiceLocal::em(caminho.clone()).registrar_processo(&instalacao(), "N1", "10");
    let conexao = rusqlite::Connection::open(&caminho).unwrap();
    conexao
        .execute_batch(
            "DROP TABLE controle_unidade;
             DROP TABLE unidade;
             DROP TABLE lista_unidade;
             DROP TABLE documento;
             ALTER TABLE instalacao DROP COLUMN entrada_com_documento;
             ALTER TABLE instalacao DROP COLUMN entrada_com_documento_testado_em;
             ALTER TABLE instalacao DROP COLUMN recusas_com_documento;
             PRAGMA user_version = 1;",
        )
        .unwrap();
    drop(conexao);
    let indice = IndiceLocal::em(caminho);
    assert_eq!(
        indice.id_por_nup(&instalacao(), "N1").as_deref(),
        Some("10")
    );
    indice.registrar_documentos(&instalacao(), "10", &[("0000101", "21")]);
    assert_eq!(indice.documentos(), [linha("0000101", "10", "21")]);
    assert!(indice.ligado());
}
