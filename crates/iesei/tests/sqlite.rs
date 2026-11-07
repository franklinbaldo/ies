use iesei::instalacao::Instalacao;
use iesei::observacao::{
    Cobertura, FormaDoControle, ObservacaoDoControle, Proveniencia, TabelaObservada, UnidadeExibida,
};
use iesei::sqlite::{MIGRACAO_CONTROLE, consultar, registrar};
use rusqlite::Connection;

fn unidade_sintetica() -> ObservacaoDoControle<()> {
    ObservacaoDoControle {
        instalacao: Instalacao::interpretar("https://sei.exemplo/sei").unwrap(),
        unidade: UnidadeExibida::nova("PPI").unwrap(),
        forma: FormaDoControle::Resumida {
            recebidos: Cobertura::Observada(TabelaObservada::nova(Some(0), None, vec![]).unwrap()),
            gerados: Cobertura::Observada(TabelaObservada::nova(Some(0), None, vec![]).unwrap()),
        },
        proveniencia: Proveniencia::Cassette {
            cartao: "sintetica".to_owned(),
        },
    }
}

#[test]
fn roundtrip_do_controle_preserva_identidades_e_cobertura() {
    let mut banco = Connection::open_in_memory().unwrap();
    banco.execute_batch(MIGRACAO_CONTROLE).unwrap();
    let observacao = unidade_sintetica();
    let transacao = banco.transaction().unwrap();
    registrar(&transacao, "perfil-sintetico", &observacao).unwrap();
    transacao.commit().unwrap();

    let recarregada = consultar(
        &banco,
        "https://sei.exemplo/sei",
        "perfil-sintetico",
    )
    .unwrap()
    .unwrap();
    assert_eq!(recarregada, observacao);
    assert!(recarregada.suficiente_para_servir());
    assert!(consultar(&banco, "https://outro.exemplo", "perfil-sintetico")
        .unwrap()
        .is_none());
}

#[test]
fn observacao_identica_nao_escreve_de_novo() {
    let mut banco = Connection::open_in_memory().unwrap();
    banco.execute_batch(MIGRACAO_CONTROLE).unwrap();
    let observacao = unidade_sintetica();
    {
        let transacao = banco.transaction().unwrap();
        registrar(&transacao, "perfil-sintetico", &observacao).unwrap();
        transacao.commit().unwrap();
    }
    let antes: i64 = banco
        .query_row("SELECT total_changes()", [], |linha| linha.get(0))
        .unwrap();
    {
        let transacao = banco.transaction().unwrap();
        registrar(&transacao, "perfil-sintetico", &observacao).unwrap();
        transacao.commit().unwrap();
    }
    let depois: i64 = banco
        .query_row("SELECT total_changes()", [], |linha| linha.get(0))
        .unwrap();
    assert_eq!(antes, depois);
}

#[test]
fn ausencia_de_lista_nao_e_convertida_em_zero() {
    let mut banco = Connection::open_in_memory().unwrap();
    banco.execute_batch(MIGRACAO_CONTROLE).unwrap();
    let mut observacao = unidade_sintetica();
    if let FormaDoControle::Resumida { gerados, .. } = &mut observacao.forma {
        *gerados = Cobertura::NaoObservada;
    }
    let transacao = banco.transaction().unwrap();
    registrar(&transacao, "perfil-sintetico", &observacao).unwrap();
    transacao.commit().unwrap();
    let lida = consultar(
        &banco,
        "https://sei.exemplo/sei",
        "perfil-sintetico",
    )
    .unwrap()
    .unwrap();
    assert!(!lida.suficiente_para_servir());
    assert_eq!(lida, observacao);
}
