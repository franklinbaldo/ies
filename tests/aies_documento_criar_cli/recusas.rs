//! Casos BDD-246-03 da CLI completa sobre tipos oferecidos pelo anti-IES.
use super::{criar, executar, executar_com_cena, json};

#[test]
fn bdd_246_03_cli_recusa_tipo_ausente_sem_escrita() {
    let mut previa = criar(false);
    previa[6] = "Parecer";
    let mut confirmacao = criar(true);
    confirmacao[6] = "Parecer";
    let (saidas, trilha, quantidade) = executar(&[&previa, &confirmacao], false);
    for saida in &saidas {
        assert_eq!(
            saida.status.code(),
            Some(1),
            "stdout={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stdout)
        );
        let resposta = json(saida);
        assert_eq!(resposta["subcodigo"], "tipo_ausente");
        assert_eq!(resposta["retentavel"], false);
    }
    assert_eq!(quantidade, 1);
    assert!(
        !trilha
            .iter()
            .any(|pedido| pedido.starts_with("POST documento_gerar:")),
        "a lista sem o tipo não permite criar documento: {trilha:?}"
    );
}

#[test]
fn bdd_246_03_cli_expande_lista_e_recusa_tipo_ambiguo_sem_escrita() {
    // A lista recente publica apenas a primeira série. A segunda série
    // de nome igual está disponível somente após "Exibir todos".
    let previa = criar(false);
    let confirmacao = criar(true);
    let (saidas, trilha, quantidade) = executar_com_cena(&[&previa, &confirmacao], false, true);
    for saida in &saidas {
        assert_eq!(
            saida.status.code(),
            Some(1),
            "stdout={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stdout)
        );
        let resposta = json(saida);
        assert_eq!(resposta["subcodigo"], "tipo_ambiguo");
        assert_eq!(resposta["retentavel"], false);
    }
    assert_eq!(quantidade, 1, "a lista ambígua não deve criar documento");
    assert!(
        trilha
            .iter()
            .any(|pedido| pedido.starts_with("POST documento_escolher_tipo:")),
        "a lista completa precisa ser consultada para aferir unicidade: {trilha:?}"
    );
    assert!(
        !trilha
            .iter()
            .any(|pedido| pedido.starts_with("POST documento_gerar:")),
        "não enviar escrita ao encontrar tipo ambíguo: {trilha:?}"
    );
}
