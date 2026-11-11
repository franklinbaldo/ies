//! Regressão offline do binário real: o setup não esconde a causa de E/S.
#[cfg(unix)]
#[test]
fn setup_recusa_ancestral_regular_sem_expor_caminho_ou_bytes() {
    use std::fs;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pasta =
        std::env::temp_dir().join(format!("ies-setup-causa-{}-{nonce}", std::process::id()));
    fs::create_dir(&pasta).unwrap();
    let arquivo = |nome: &str, bytes: &str| {
        let caminho = pasta.join(nome);
        fs::write(&caminho, bytes).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&caminho, fs::Permissions::from_mode(0o600)).unwrap();
        caminho
    };
    let url = arquivo("url", "URL FICTICIA");
    let origem = arquivo("origem", "ORIGEM FICTICIA");
    let ancestral = arquivo("ancestral", "SENTINELA DE TESTE");
    let destino = pasta.join("config.json");
    let cookie = ancestral.join("cookie");

    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["setup", "--config-file"])
        .arg(&destino)
        .arg("--url-controle-file")
        .arg(&url)
        .arg("--cookie-file")
        .arg(&cookie)
        .arg("--origem-file")
        .arg(&origem)
        .arg("--json")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let erro = String::from_utf8_lossy(&output.stderr);
    assert!(
        erro.contains("ArquivoPrivadoNaoLido(NotADirectory)"),
        "{erro}"
    );
    for privado in [&cookie, &ancestral, &destino] {
        assert!(!erro.contains(privado.to_string_lossy().as_ref()));
    }
    assert!(!erro.contains("SENTINELA DE TESTE"));
    assert!(!destino.exists());
    assert_eq!(
        fs::read_to_string(&ancestral).unwrap(),
        "SENTINELA DE TESTE"
    );
    fs::remove_dir_all(pasta).unwrap();
}
