//! Refs #60. BDD: propriedades observadas pertencem ao leitor Web.
use std::process::Command;

#[test]
fn documento_com_propriedades_usa_leitura_web_e_nao_stub() {
    // Dado o número SEI válido e nenhuma sessão configurada.
    // Quando executo ies documento DOC --com-propriedades --json.
    // Então a CLI tenta a leitura Web, falha por sessão ausente,
    // e NÃO responde capacidade nao_implementado.
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["documento", "0011519", "--com-propriedades", "--json"])
        .env("IES_CONFIG_FILE", "/caminho/que-nao-existe/ies-config.json")
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .expect("o binário do IES deve iniciar");
    assert_eq!(saida.status.code(), Some(1));
    let valor: serde_json::Value =
        serde_json::from_slice(&saida.stdout).expect("falha da CLI em JSON");
    assert_ne!(
        valor["codigo"], "nao_implementado",
        "as propriedades precisam passar pelo leitor Web: {valor}"
    );
}
