//! Superfície de sobrestar e retomar (#269): entradas inválidas e
//! confirmação não implementada recusam ANTES de ler configuração ou rede.
use std::process::{Command, Output};

fn sem_sessao(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .env("IES_INDICE_FILE", "/nao/existe/ies.sqlite3")
        .output()
        .expect("IES deve iniciar")
}
fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("JSON na saída")
}
const NUP: &str = "00001.000001/2026-01";

#[test]
fn motivo_em_branco_e_erro_de_uso_sem_rede() {
    let saida = sem_sessao(&["processo", "sobrestar", NUP, "--motivo", " \t ", "--json"]);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(json(&saida)["subcodigo"], "motivo_do_sobrestamento_vazio");
}
#[test]
fn nup_invalido_prioritario_antes_da_sessao() {
    let saida = sem_sessao(&["processo", "sobrestar", " ", "--motivo", "teste", "--json"]);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(json(&saida)["subcodigo"], "numero_processo_vazio");
}
#[test]
fn motivo_obrigatorio_no_clap() {
    let saida = sem_sessao(&["processo", "sobrestar", NUP, "--json"]);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(saida.stdout, b"");
}
#[test]
fn confirmar_nao_escreve_nem_le_sessao_antes_da_implementacao() {
    for argumentos in [
        vec![
            "processo",
            "sobrestar",
            NUP,
            "--motivo",
            "teste",
            "--confirmar",
            "--json",
        ],
        vec!["processo", "retomar", NUP, "--confirmar", "--json"],
    ] {
        let saida = sem_sessao(&argumentos);
        assert_eq!(saida.status.code(), Some(1), "{argumentos:?}");
        let valor = json(&saida);
        assert_eq!(
            valor["subcodigo"],
            "confirmacao_de_sobrestamento_nao_implementada"
        );
        assert_eq!(valor["retentavel"], false);
    }
}
