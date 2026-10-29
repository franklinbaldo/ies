use std::{fs, process::Command};

#[test]
fn setup_grava_apenas_caminhos_e_nao_sobrescreve_configuracao() {
    let dir = std::env::temp_dir().join(format!("ies-setup-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let paths: Vec<_> = ["url", "cookie", "origem"].iter().map(|s| dir.join(s)).collect();
    for path in &paths {
        fs::write(path, "CONTEUDO_PRIVADO").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
    let destino = dir.join("config.json");
    let args = [
        "setup", "--config-file", destino.to_str().unwrap(),
        "--url-controle-file", paths[0].to_str().unwrap(),
        "--cookie-file", paths[1].to_str().unwrap(),
        "--origem-file", paths[2].to_str().unwrap(), "--json",
    ];
    let primeira = Command::new(env!("CARGO_BIN_EXE_ies")).args(args).output().unwrap();
    assert!(primeira.status.success(), "{primeira:?}");
    let json = fs::read_to_string(&destino).unwrap();
    assert!(!json.contains("CONTEUDO_PRIVADO"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(&destino).unwrap().permissions().mode() & 0o077, 0);
    }
    let segunda = Command::new(env!("CARGO_BIN_EXE_ies")).args(args).output().unwrap();
    assert_eq!(segunda.status.code(), Some(1));
    let valor: serde_json::Value = serde_json::from_slice(&segunda.stdout).unwrap();
    assert_eq!(valor["subcodigo"], "configuracao_ja_existe_ou_sem_permissao");
    let status = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["status", "--json"])
        .env("IES_CONFIG_FILE", &destino)
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .unwrap();
    assert!(status.status.success());
    let diagnostico: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(diagnostico["configuracao"], "caminhos_informados");
    fs::remove_dir_all(dir).unwrap();
}
