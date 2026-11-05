//! Contrato temporário: enquanto o fluxo Web não está integrado,
//! a CLI não pode aceitar uma solicitação de escrita.
use std::process::Command;

#[test]
fn criar_indisponivel_recusa_antes_de_qualquer_rede() {
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "documento",
            "criar",
            "--processo",
            "00001.000001/2026-01",
            "--tipo",
            "Despacho",
            "--confirmar",
        ])
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .output()
        .expect("IES deve iniciar");
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(saida.stdout, b"");
}
