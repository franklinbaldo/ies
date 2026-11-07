//! Entrada e trava de escrita de `ies processo relacionar`.
//! Não abre login nem acessa SEI. Testa a ausência de efeitos remotos.
use std::process::{Command, Output};

const NUP: &str = "00001.000001/2026-01";
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
#[test]
fn entrada_vazia_sai_2_antes_da_sessao() {
    for args in [
        vec!["processo", "relacionar", " ", "--com", NUP, "--json"],
        vec!["processo", "relacionar", NUP, "--com", " ", "--json"],
    ] {
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2));
        assert_eq!(json(&saida)["subcodigo"], "numero_processo_vazio");
    }
}
#[test]
fn sem_configuracao_previa_recusa_sem_rede() {
    let saida = sem_sessao(&["processo", "relacionar", NUP, "--com", NUP, "--json"]);
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(json(&saida)["codigo"], "fonte_invalida");
}
#[test]
fn confirmar_e_remover_confirmar_sem_sessao_recusam_sem_escrita() {
    for remover in [false, true] {
        let mut args = vec![
            "processo",
            "relacionar",
            NUP,
            "--com",
            NUP,
            "--confirmar",
            "--json",
        ];
        if remover {
            args.push("--remover");
        }
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(1));
        assert_eq!(
            json(&saida)["subcodigo"],
            "arquivo_privado_ausente_ou_inacessivel"
        );
    }
}
#[test]
fn destino_obrigatorio_e_flag_desconhecida_saem_2() {
    for args in [
        vec!["processo", "relacionar", NUP],
        vec!["processo", "relacionar", NUP, "--com", NUP, "--rest"],
    ] {
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(2));
    }
}
