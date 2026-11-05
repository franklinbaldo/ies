//! `--saida` e a prévia de `documento salvar` sem sessão nem rede (#207):
//! a foto `DOC.vN.html` conferida antes da rede e o par vN / v(N+1).
use std::process::Command;

fn ies_sem_sessao(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .expect("IES deve iniciar")
}

#[test]
fn documento_saida_confere_a_foto_e_o_diretorio_antes_da_sessao() {
    // #207: com `--versao N` o nome `DOC.vN.html` é conhecido antes da
    // rede; uma foto presente que declara outro documento é recusada.
    let pasta = std::env::temp_dir().join(format!("ies-foto-versao-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    let outra =
        "<html><head><title>SEI/X - 0099999 - Despacho</title></head><body><p>x</p></body></html>";
    let alvo = pasta.join("0011519.v2.html");
    std::fs::write(&alvo, outra).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&alvo, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let dir = pasta.to_str().unwrap();
    let args = [
        "documento",
        "0011519",
        "--versao",
        "2",
        "--saida",
        dir,
        "--json",
    ];
    let output = ies_sem_sessao(&args);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "foto_local_divergente");
    let ausente = pasta.join("nao-existe");
    let args = [
        "documento",
        "0011519",
        "--saida",
        ausente.to_str().unwrap(),
        "--json",
    ];
    let output = ies_sem_sessao(&args);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "diretorio_texto_documento_ausente");
    std::fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn salvar_local_sem_foto_registrada_recusa_a_base() {
    // #207: o par `DOC.v2.html` / `DOC.v3.html`, sem índice que registre
    // o SHA-256 da foto v2: a base não é aceita.
    let pasta = std::env::temp_dir().join(format!("ies-salvar-par-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    for (n, corpo) in [(2, "<p>a</p>"), (3, "<p>b</p>")] {
        let alvo = pasta.join(format!("0011519.v{n}.html"));
        std::fs::write(&alvo, corpo).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&alvo, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .current_dir(&pasta)
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .args(["documento", "salvar", "0011519", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "base_versao_nao_registrada");
    std::fs::remove_dir_all(pasta).unwrap();
}
