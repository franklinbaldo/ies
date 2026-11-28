//! E2E sintético da capacidade estritamente local.
#[test]
fn inventario_cli_nao_reescreve_html_nem_exige_sessao() {
    let pasta = std::env::temp_dir().join(format!(
        "ies-referencias-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    std::fs::create_dir(&pasta).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&pasta, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let arquivo = pasta.join("texto.html");
    let html = "<p hidden>12345678<p>0012345678 e 0012345678";
    std::fs::write(&arquivo, html).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&arquivo, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let saida = std::process::Command::new(env!("CARGO_BIN_EXE_ies"))
        .env_clear()
        .args(["--json", "documento", "referencias", "--arquivo"])
        .arg(&arquivo)
        .output()
        .unwrap();
    assert!(saida.status.success(), "{:?}", saida.stderr);
    let resultado: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(resultado, serde_json::json!({
        "escopo": "texto_extraido_offline",
        "candidatos": [{"tipo":"documento","texto":"0012345678","ocorrencias":2,"estado":"candidato"}]
    }));
    assert_eq!(std::fs::read_to_string(&arquivo).unwrap(), html);
    std::fs::remove_dir_all(pasta).unwrap();
}
