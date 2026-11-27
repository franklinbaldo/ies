//! Contenção SQLite determinística nas etapas de preparação e operação.
use super::*;

// Mantém a transação bloqueadora viva durante a chamada síncrona do
// índice. Não depende de sleeps, escalonamento ou alteração do timeout.
fn conferir_busy(indice: &IndiceLocal, pronto: bool) {
    assert!(!indice.ligado());
    assert_eq!(indice.pronto.get(), pronto);
    assert_eq!(
        serde_json::json!(indice.avisos()),
        serde_json::json!([{
            "falha": "banco",
            "causa": {"erro": "sqlite", "codigo_sqlite": 5}
        }])
    );
    indice.registrar_processo(&instalacao(), "N2", "20");
    assert!(indice.avisos().is_empty());
}

#[test]
fn busy_ao_preparar_banco_delete_com_leitor_ativo() {
    let caminho = pasta("busy-delete-leitor").join(NOME_DO_ARQUIVO);
    let indice = IndiceLocal::em(caminho.clone());
    indice.registrar_processo(&instalacao(), "N1", "10");
    assert!(indice.ligado());

    let mut bloqueador = rusqlite::Connection::open(&caminho).unwrap();
    let modo: String = bloqueador
        .pragma_update_and_check(None, "journal_mode", "DELETE", |l| l.get(0))
        .unwrap();
    assert_eq!(modo, "delete");
    let leitura = bloqueador.transaction().unwrap();
    let total: i64 = leitura
        .query_row("SELECT count(*) FROM processo", [], |l| l.get(0))
        .unwrap();
    assert_eq!(total, 1);

    let novo = IndiceLocal::em(caminho.clone());
    novo.registrar_processo(&instalacao(), "N2", "20");
    conferir_busy(&novo, false);

    leitura.rollback().unwrap();
    drop(bloqueador);
    let recuperado = IndiceLocal::em(caminho);
    assert_eq!(
        recuperado.id_por_nup(&instalacao(), "N1").as_deref(),
        Some("10")
    );
    assert_eq!(recuperado.id_por_nup(&instalacao(), "N2"), None);
    assert!(recuperado.ligado());
}

#[test]
fn busy_com_escritor_wal_distingue_preparacao_de_operacao() {
    let caminho = pasta("busy-wal-escritor").join(NOME_DO_ARQUIVO);
    let pronto = IndiceLocal::em(caminho.clone());
    pronto.registrar_processo(&instalacao(), "N1", "10");
    assert!(pronto.ligado());
    assert!(pronto.pronto.get());

    let mut bloqueador = rusqlite::Connection::open(&caminho).unwrap();
    let escrita = bloqueador
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .unwrap();
    escrita
        .execute("UPDATE processo SET observado_em = 0", [])
        .unwrap();

    // Controle independente: o WAL já vigente não precisa mudar; abrir
    // sem preparar também funciona mesmo com o escritor ativo.
    let controle = abertura::conectar(&caminho, false).unwrap();
    let modo: String = controle
        .pragma_update_and_check(None, "journal_mode", "WAL", |l| l.get(0))
        .unwrap();
    assert_eq!(modo, "wal");
    drop(controle);

    let novo = IndiceLocal::em(caminho.clone());
    novo.registrar_processo(&instalacao(), "N2", "20");
    conferir_busy(&novo, false);

    pronto.registrar_processo(&instalacao(), "N2", "20");
    conferir_busy(&pronto, true);

    escrita.rollback().unwrap();
    drop(bloqueador);
    let recuperado = IndiceLocal::em(caminho);
    assert_eq!(
        recuperado.id_por_nup(&instalacao(), "N1").as_deref(),
        Some("10")
    );
    assert_eq!(recuperado.id_por_nup(&instalacao(), "N2"), None);
    assert!(recuperado.ligado());
}
