//! Sem --saida, nenhum segredo e nenhum GET são necessários para recusar o comando.
use std::process::Command;

#[test]
fn get_opt_in_sem_destino_privado_falha_antes_de_sessao_ou_url() {
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "--json",
            "documento",
            "0007",
            "--url-documento-file",
            "/arquivo-nao-existente-para-teste",
            "--autorizar-efeito-de-leitura",
        ])
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(2));
    let saida: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(saida["codigo"], "entrada_invalida");
    assert_eq!(saida["subcodigo"], "saida_privada_obrigatoria_para_documento_web");
    assert!(resposta.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("PHPSESSID"));
}
