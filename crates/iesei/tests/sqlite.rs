use iesei::instalacao::Instalacao;
use iesei::observacao::{UnidadeExibida, VisaoDoControle};
use iesei::sqlite::{MIGRACAO_CONTROLE, consultar_unidade, registrar_unidade};
use rusqlite::Connection;

#[test]
fn guarda_fato_unico_e_isola_geracoes_e_instalacoes() {
    let mut banco = Connection::open_in_memory().unwrap();
    banco.execute_batch(MIGRACAO_CONTROLE).unwrap();
    let origem = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    let outra = Instalacao::interpretar("https://outro.exemplo/sei").unwrap();
    let nome = UnidadeExibida::nova("TESTE").unwrap();

    assert!(consultar_unidade(&banco, &origem, "geracao-a").unwrap().is_none());
    {
        let tx = banco.transaction().unwrap();
        registrar_unidade(&tx, &origem, "geracao-a", &nome, VisaoDoControle::Resumida, 1.0)
            .unwrap();
        registrar_unidade(&tx, &origem, "geracao-a", &nome, VisaoDoControle::Detalhada, 2.0)
            .unwrap();
        tx.commit().unwrap();
    }
    let guardada = consultar_unidade(&banco, &origem, "geracao-a").unwrap().unwrap();
    assert_eq!(guardada.unidade.as_str(), "TESTE");
    assert_eq!(guardada.visao, VisaoDoControle::Detalhada);
    assert_eq!(guardada.observada_em, 2.0);
    let linhas: i64 = banco
        .query_row("SELECT count(*) FROM controle_unidade", [], |r| r.get(0))
        .unwrap();
    assert_eq!(linhas, 1);
    assert!(consultar_unidade(&banco, &origem, "geracao-b").unwrap().is_none());
    assert!(consultar_unidade(&banco, &outra, "geracao-a").unwrap().is_none());
}
