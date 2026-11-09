//! Regressões do formulário de relacionamentos; PEN1 e estado SQLite sintético.
use super::*;
use crate::rotas;
use iesei::banco_cassettes::iniciar;
use iesei::processos::{ProcessoSimulado, guardar};

fn cenario() -> (Connection, Instalacao) {
    let mut db = Connection::open_in_memory().unwrap();
    iniciar(&db).unwrap();
    let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
    for (id, numero) in [("10", "P-01"), ("22", "P-02")] {
        guardar(
            &mut db,
            &origem,
            "caso",
            &ProcessoSimulado {
                id: id.into(),
                numero: numero.into(),
                unidade_origem: "TESTE".into(),
                aberto_em: vec!["TESTE".into()],
                anotacao_origem: None,
                destinos: vec![],
            },
        )
        .unwrap();
        relacionamentos::observar(&db, &origem, "caso", id).unwrap();
    }
    (db, origem)
}

/// Apoio ao BDD-258-02: o POST sintético só é aceito quando uma
/// nova GET, inclusive pelo processo inverso, lê o vínculo no SQLite.
#[test]
fn adicionar_post_reler_bilateralmente_e_preservar_estado() {
    let (db, origem) = cenario();
    let url = "/sei/controlador.php?acao=procedimento_relacionar&id_procedimento=10";
    let inicial = rotas::responder("GET", url, b"", &db, &origem, "caso");
    assert_eq!(inicial.estado, Estado::Ok);
    // Cassette PEN1: a lista observada vazia oferece o formulário,
    // mas não publica a tabela nem o campo de remoção.
    assert!(inicial.corpo.contains("frmProcedimentoListarRelacionamentos"));
    assert!(inicial.corpo.contains("hdnIdProtocolo"));
    assert!(!inicial.corpo.contains("Lista de Processos Relacionados"));
    assert!(!inicial.corpo.contains("hdnInfraItemId"));
    assert!(!inicial.corpo.contains("acaoRemover"));
    assert!(!inicial.corpo.contains("P-02</a>"));

    let resposta = rotas::responder(
        "POST",
        url,
        b"hdnIdProtocolo=22&txtProtocolo=P-02&txtIdentificacaoProtocolo=&sbmAdicionar=Adicionar",
        &db,
        &origem,
        "caso",
    );
    assert_eq!(resposta.estado, Estado::Redirecionamento);
    assert!(
        resposta
            .localizacao
            .as_deref()
            .is_some_and(|s| s.contains("resultado=1"))
    );
    let depois = rotas::responder("GET", url, b"", &db, &origem, "caso");
    assert_eq!(depois.estado, Estado::Ok);
    assert!(depois.corpo.contains("Lista de Processos Relacionados"));
    assert!(depois.corpo.contains("hdnInfraItemId"));
    assert!(depois.corpo.contains("acaoRemover"));
    assert!(depois.corpo.contains("P-02</a>"));
    let inverso = rotas::responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_relacionar&id_procedimento=22",
        b"",
        &db,
        &origem,
        "caso",
    );
    assert!(inverso.corpo.contains("P-01</a>"));
    assert_eq!(
        relacionamentos::listar(&db, &origem, "caso", "10")
            .unwrap()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        relacionamentos::listar(&db, &origem, "caso", "22")
            .unwrap()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn adicionar_recusa_identidade_que_nao_corresponde_ao_nup() {
    let (db, origem) = cenario();
    let url = "/sei/controlador.php?acao=procedimento_relacionar&id_procedimento=10";
    let resposta = rotas::responder(
        "POST",
        url,
        b"hdnIdProtocolo=22&txtProtocolo=P-03&txtIdentificacaoProtocolo=&sbmAdicionar=Adicionar",
        &db,
        &origem,
        "caso",
    );
    assert_eq!(resposta.estado, Estado::RequisicaoInvalida);
    assert_eq!(
        relacionamentos::listar(&db, &origem, "caso", "10")
            .unwrap()
            .unwrap(),
        vec![]
    );
}

/// Apoio ao BDD-258-03: a remoção sintética só é confirmável pela
/// consulta posterior dos dois processos, sem apagar outros vínculos.
#[test]
fn remover_post_exige_releitura_bilateral_e_preserva_vizinho() {
    let (mut db, origem) = cenario();
    guardar(
        &mut db,
        &origem,
        "caso",
        &ProcessoSimulado {
            id: "33".into(),
            numero: "P-03".into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();
    relacionamentos::observar(&db, &origem, "caso", "33").unwrap();
    relacionamentos::registrar(&db, &origem, "caso", "10", "22").unwrap();
    relacionamentos::registrar(&db, &origem, "caso", "10", "33").unwrap();
    let tela = rotas::responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_relacionar&id_procedimento=10",
        b"",
        &db,
        &origem,
        "caso",
    );
    assert!(tela.corpo.contains("acaoRemover('22');"));
    assert!(tela.corpo.contains("procedimento_excluir_relacionamento"));

    let resposta = rotas::responder(
        "POST",
        "/sei/controlador.php?acao=procedimento_excluir_relacionamento&id_procedimento=10",
        b"hdnInfraItemId=22",
        &db,
        &origem,
        "caso",
    );
    assert_eq!(resposta.estado, Estado::Redirecionamento);
    assert!(
        resposta
            .localizacao
            .as_deref()
            .is_some_and(|l| l.contains("resultado=1"))
    );
    let origem_apos = relacionamentos::listar(&db, &origem, "caso", "10")
        .unwrap()
        .unwrap();
    assert_eq!(origem_apos.len(), 1);
    assert_eq!(origem_apos[0].id, "33");
    assert_eq!(
        relacionamentos::listar(&db, &origem, "caso", "22")
            .unwrap()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        relacionamentos::listar(&db, &origem, "caso", "33")
            .unwrap()
            .unwrap()[0]
            .id,
        "10"
    );
}

#[test]
fn remover_post_recusa_identidade_estranha_sem_alterar_vinculo() {
    let (db, origem) = cenario();
    relacionamentos::registrar(&db, &origem, "caso", "10", "22").unwrap();
    let resposta = rotas::responder(
        "POST",
        "/sei/controlador.php?acao=procedimento_excluir_relacionamento&id_procedimento=10",
        b"hdnInfraItemId=99",
        &db,
        &origem,
        "caso",
    );
    assert_eq!(resposta.estado, Estado::RequisicaoInvalida);
    assert_eq!(
        relacionamentos::listar(&db, &origem, "caso", "10")
            .unwrap()
            .unwrap()
            .len(),
        1
    );
}
