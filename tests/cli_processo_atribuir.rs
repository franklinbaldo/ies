//! Trava de escrita de `ies processo atribuir` (#264).
//! Não abre login nem acessa o SEI: só confere a recusa antes da sessão.
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

#[test]
fn confirmar_recusa_antes_da_sessao() {
    for operacao in [&["--usuario", "fulano"][..], &["--remover"][..]] {
        let mut args = vec!["processo", "atribuir", NUP];
        args.extend_from_slice(operacao);
        args.extend_from_slice(&["--confirmar", "--json"]);
        let saida = sem_sessao(&args);
        assert_eq!(saida.status.code(), Some(1), "{args:?}");
        let valor: serde_json::Value =
            serde_json::from_slice(&saida.stdout).expect("JSON na saída");
        assert_eq!(
            valor["subcodigo"],
            "escrita_de_atribuicao_ainda_nao_verificada"
        );
    }
}

#[test]
fn usuario_e_remover_juntos_sao_recusados_pelo_clap() {
    let saida = sem_sessao(&["processo", "atribuir", NUP, "--usuario", "x", "--remover"]);
    assert_eq!(saida.status.code(), Some(2));
}
