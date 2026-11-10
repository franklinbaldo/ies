//! #244: contrato observável da pesquisa e listagem a partir do índice.
//! Estes testes não dependem de nenhuma instalação do SEI nem fazem HTTP.
use std::process::{Command, Output};

#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::falha;

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
    falha(&result, "UrlDoControleSemCaminho");
}

#[test]
fn listar_unidades_reconhece_a_cli_e_pede_configuracao_local() {
    let result = executar(&["unidade", "listar", "--json"]);
    falha(&result, "UrlDoControleSemCaminho");
}

#[test]
fn atualizar_sem_sessao_recusa_antes_de_qualquer_leitura_web() {
    let result = executar(&["unidade", "listar", "--atualizar", "--json"]);
    falha(&result, "UrlDoControleSemCaminho");
}
