//! Casos BDD-246-03 da CLI completa sobre tipos oferecidos pelo anti-IES.
use super::{criar, executar, executar_com_cena, falha_por, json};

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
    assert_eq!(trilha, [] as [String; 0]);
    assert_eq!(quantidade, 1);
}

#[test]
fn previa_texto_padrao_cli_usa_catalogo_da_unidade_sem_escrita() {
    let mut previa = criar(false);
    previa.extend(["--texto-padrao", "Minuta Padrão & B"]);
    let (saidas, trilha, quantidade) = executar(&[&previa], false);
    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    let resultado = json(&saidas[0]);
    assert_eq!(resultado["texto_padrao"]["id"], "007");
    assert_eq!(resultado["texto_padrao"]["nome"], "Minuta Padrão & B");
    assert_eq!(resultado["confirmado"], false);
    assert_eq!(quantidade, 1);
    assert!(
        trilha
            .iter()
            .any(|pedido| pedido.starts_with("POST texto_padrao_auto_completar: 200"))
    );
    assert!(
        !trilha
            .iter()
            .any(|pedido| pedido.starts_with("POST documento_gerar:"))
    );
}

#[test]
fn texto_padrao_fora_do_treinamento_recusa_antes_de_ler_sessao() {
    for origem in [
        "https://sei.exemplo.gov.br/sei",
        "https://sei.test.exemplo/sei",
    ] {
        let pasta = super::Pasta::nova();
        let arquivo = pasta.privado("origem", origem);
        let mut previa = criar(false);
        previa.extend(["--texto-padrao", "Ofício"]);
        let saida = std::process::Command::new(env!("CARGO_BIN_EXE_ies"))
            .args(&previa)
            .current_dir(&pasta.0)
            .env("IES_ORIGEM_FILE", arquivo)
            .env("IES_URL_CONTROLE_FILE", pasta.0.join("url-ausente"))
            .env("IES_COOKIE_FILE", pasta.0.join("cookie-ausente"))
            .env("HOME", &pasta.0)
            .env_remove("IES_CONFIG_FILE")
            .env_remove("IES_CREDENCIAIS_FILE")
            .output()
            .unwrap();
        super::falha_cli::falha(&saida, "OrigemForaDoTreinamento");
    }
}

#[test]
fn previa_modelo_manual_confere_oferta_sem_abrir_fonte_ou_criar() {
    let mut previa = criar(false);
    previa.extend(["--modelo", "0011526"]);
    let (saidas, trilha, quantidade) = executar(&[&previa], false);
    assert_eq!(saidas[0].status.code(), Some(0));
    let resultado = json(&saidas[0]);
    assert_eq!(resultado["modelo_solicitado"], "0011526");
    assert_eq!(resultado["confirmado"], false);
    assert_eq!(quantidade, 1);
    let (_, trilha_normal, _) = executar(&[&criar(false)], false);
    assert_eq!(trilha, trilha_normal, "nenhuma consulta adicional da fonte");
}

#[test]
fn modelo_com_confirmacao_recusa_antes_da_sessao() {
    let mut confirmacao = criar(true);
    confirmacao.extend(["--modelo", "0011526"]);
    let (saidas, trilha, quantidade) = executar(&[&confirmacao], false);
    assert_eq!(saidas[0].status.code(), Some(2));
    assert_eq!(trilha, [] as [std::string::String; 0]);
    assert_eq!(quantidade, 1);
}
