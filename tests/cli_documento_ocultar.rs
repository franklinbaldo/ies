//! BDD de #60: --ocultar-inacessiveis pertence à leitura Web, não ao stub.
use std::process::Command;

#[test]
fn documento_ocultar_inacessiveis_chega_a_validacao_da_sessao() {
    // Dado um DOC válido e nenhuma configuração de acesso.
    // Quando executo ies documento DOC --ocultar-inacessiveis --json.
    // Então a CLI tenta a leitura Web e reporta a ausência de sessão,
    // sem devolvê-lo ao stub de capacidade não implementada.
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["documento", "0011519", "--ocultar-inacessiveis", "--json"])
        .env("IES_CONFIG_FILE", "/caminho/que-nao-existe/ies-config.json")
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .expect("o binário do IES deve iniciar");
    // O stub de capacidade não implementada publicava JSON no stdout;
    // a leitura Web falha tipada, com stdout vazio.
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert_eq!(saida.status.code(), Some(1), "{erro}");
    assert_eq!(
        saida.stdout, b"",
        "a opção deve ser atendida pelo leitor Web"
    );
    assert!(
        erro.starts_with("Error: "),
        "a opção deve ser atendida pelo leitor Web: {erro}"
    );
}
