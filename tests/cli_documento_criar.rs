//! Regressão do contrato de CLI #246, sem sessão nem requisições SEI.
use std::process::Command;

fn executar(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .output()
        .expect("IES deve iniciar")
}

#[test]
fn criar_tem_cli_publica_e_flags_do_contrato() {
    let saida = executar(&["documento", "criar", "--help"]);
    assert!(saida.status.success());
    let ajuda = String::from_utf8(saida.stdout).unwrap();
    for flag in [
        "--processo",
        "--tipo",
        "--descricao",
        "--nivel",
        "--confirmar",
    ] {
        assert!(ajuda.contains(flag), "{flag} ausente da ajuda");
    }
}

#[test]
fn criar_confirmar_sem_sessao_falha_fechado_sem_acesso_a_rede() {
    let saida = executar(&[
        "--json",
        "documento",
        "criar",
        "--processo",
        "00001.000001/2026-01",
        "--tipo",
        "Despacho",
        "--confirmar",
    ]);
    assert_eq!(saida.status.code(), Some(1));
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_ne!(
        valor["subcodigo"],
        "criacao_de_documento_ainda_nao_validada"
    );
    assert!(!String::from_utf8_lossy(&saida.stderr).contains("/nao/existe"));
}

#[test]
fn criar_sem_tipo_e_nup_invalido_sao_recusados_no_clap_ou_preflight() {
    let sem_tipo = executar(&["documento", "criar", "--processo", "00001.000001/2026-01"]);
    assert_eq!(sem_tipo.status.code(), Some(2));
    let nup_invalido = executar(&[
        "documento",
        "criar",
        "--processo",
        "nup-invalido",
        "--tipo",
        "Despacho",
    ]);
    assert_eq!(nup_invalido.status.code(), Some(2));
}
