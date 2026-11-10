//! #244: contrato observável da pesquisa e listagem a partir do índice.
//! Estes testes não dependem de nenhuma instalação do SEI nem fazem HTTP.
use std::process::{Command, Output};

fn executar(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .env_remove("IES_COOKIE_FILE")
        .output()
        .expect("binário ies")
}

#[test]
fn pesquisar_reconhece_a_cli_e_pede_configuracao_local() {
    let result = executar(&["unidade", "pesquisar", "sao/cde", "--json"]);
    assert_eq!(result.status.code(), Some(1));
    let saida: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(saida["subcodigo"], "url_do_controle_sem_caminho");
}

#[test]
fn listar_unidades_reconhece_a_cli_e_pede_configuracao_local() {
    let result = executar(&["unidade", "listar", "--json"]);
    assert_eq!(result.status.code(), Some(1));
    let saida: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(saida["subcodigo"], "url_do_controle_sem_caminho");
}

#[test]
fn atualizar_sem_sessao_recusa_antes_de_qualquer_leitura_web() {
    let result = executar(&["unidade", "listar", "--atualizar", "--json"]);
    assert_eq!(result.status.code(), Some(1));
    let saida: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(saida["subcodigo"], "url_do_controle_sem_caminho");
    assert_eq!(saida["codigo"], "fonte_invalida");
    assert_eq!(saida["retentavel"], false);
}
