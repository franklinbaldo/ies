//! Casos BDD-246-03 da CLI completa sobre tipos oferecidos pelo anti-IES.
use super::{criar, executar, executar_com_cena, json};

#[test]
fn bdd_246_03_cli_recusa_tipo_ausente_sem_escrita() {
    let mut args = criar(true);
    args[6] = "Parecer";
    let (saidas, trilha, quantidade) = executar(&[&args], false);
    assert_eq!(
        saidas[0].status.code(),
        Some(1),
        "stdout={} trilha={trilha:?}",
        String::from_utf8_lossy(&saidas[0].stdout)
    );
    let resposta = json(&saidas[0]);
    assert_eq!(resposta["subcodigo"], "tipo_de_documento_nao_oferecido");
    assert_eq!(resposta["retentavel"], false);
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
    let args = criar(true);
    let (saidas, trilha, quantidade) = executar_com_cena(&[&args], false, true);
    assert_eq!(
        saidas[0].status.code(),
        Some(1),
        "stdout={} trilha={trilha:?}",
        String::from_utf8_lossy(&saidas[0].stdout)
    );
    let resposta = json(&saidas[0]);
    assert_eq!(resposta["subcodigo"], "tipo_de_documento_ambiguo");
    assert_eq!(resposta["retentavel"], false);
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
