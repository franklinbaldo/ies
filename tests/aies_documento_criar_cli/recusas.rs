//! Casos BDD-246-03 da CLI completa sobre tipos oferecidos pelo anti-IES.
use super::{criar, executar, executar_com_cena, falha_por};

#[test]
fn bdd_246_03_cli_recusa_tipo_ausente_sem_escrita() {
    let mut previa = criar(false);
    previa[6] = "Parecer";
    let mut confirmacao = criar(true);
    confirmacao[6] = "Parecer";
    let (saidas, trilha, quantidade) = executar(&[&previa, &confirmacao], false);
    for saida in &saidas {
        falha_por(saida, "TentativaDeCriacao", "ListaDeTipos");
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
        falha_por(saida, "TentativaDeCriacao", "ListaDeTipos");
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

#[test]
fn texto_padrao_nao_oferecido_recusa_previa_sem_criar() {
    let mut previa = criar(false);
    previa.extend(["--texto-padrao", "Ofício"]);
    let (saidas, trilha, quantidade) = executar(&[&previa], false);
    falha_por(&saidas[0], "TentativaDeCriacao", "TextoPadrao");
    assert_eq!(quantidade, 1);
    assert!(
        trilha
            .iter()
            .any(|pedido| pedido.starts_with("GET documento_gerar: 200")),
        "a recusa precisa vir da tela de cadastro: {trilha:?}"
    );
    assert!(
        !trilha
            .iter()
            .any(|pedido| pedido.starts_with("POST documento_gerar:"))
    );
}

#[test]
fn texto_padrao_com_confirmacao_recusa_antes_da_sessao() {
    let mut confirmacao = criar(true);
    confirmacao.extend(["--texto-padrao", "Ofício"]);
    let (saidas, trilha, quantidade) = executar(&[&confirmacao], false);
    assert_eq!(saidas[0].status.code(), Some(2));
    assert!(trilha.is_empty());
    assert_eq!(quantidade, 1);
}
