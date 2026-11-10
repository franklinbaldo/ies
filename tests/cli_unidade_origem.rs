//! Arquivos de sessão da caixa conferidos antes de qualquer acesso: sem
//! configuração, sem origem ou com URL de outro host, `ies unidade
//! listar-processos` falha tipado sem abrir cookies nem enviar HTTP.
//! (Vieram de `tests/replay_unidade.rs`, que reunia também o antigo replay.)
use std::fs;
use std::process::Command;

#[test]
fn comando_sem_configuracao_informa_arquivo_de_controle_ausente() {
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["unidade", "listar-processos", "--json"])
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let _result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
}

#[test]
fn origem_http_e_obrigatoria_antes_de_qualquer_acesso() {
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["unidade", "listar-processos", "--json"])
        .env("IES_URL_CONTROLE_FILE", "qualquer-arquivo-de-url")
        .env("IES_COOKIE_FILE", "qualquer-arquivo-de-cookie")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(1));
    let _dados: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
}

#[test]
fn url_adulterada_falha_antes_de_abrir_cookies_ou_enviar_http() {
    let dir = std::env::temp_dir().join(format!(
        "ies-origem-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let url = dir.join("url");
    let origem = dir.join("origem");
    fs::write(
        &url,
        format!(
            "https://sei.adulterado.tld/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
            "a".repeat(64)
        ),
    ).unwrap();
    fs::write(&origem, "https://sei.treinamento.unb.br").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for p in [&url, &origem] {
            fs::set_permissions(p, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["unidade", "listar-processos", "--json"])
        .env("IES_URL_CONTROLE_FILE", url.to_str().unwrap())
        .env("IES_ORIGEM_FILE", origem.to_str().unwrap())
        .env("IES_COOKIE_FILE", dir.join("cookie-inexistente"))
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(json["codigo"], "fonte_invalida");
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("infra_hash"));
    fs::remove_dir_all(dir).unwrap();
}
