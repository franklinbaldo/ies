//! Regressões da ordem de pré-validação da leitura Web da caixa (#57).
//! Não criam sessão, não acessam SEI e não introduzem opções novas.
use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};

fn executar_com_complemento(caminho: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .arg("--json")
        .args(["unidade", "listar-processos", "--pagina-arquivo"])
        .arg(caminho)
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .output()
        .expect("executar IES localmente")
}

fn subcodigo(resultado: &Output) -> String {
    assert_eq!(resultado.status.code(), Some(1));
    assert_eq!(resultado.stderr, Vec::<u8>::new());
    let valor: Value = serde_json::from_slice(&resultado.stdout).unwrap();
    valor["subcodigo"].as_str().unwrap().to_owned()
}

#[test]
fn complemento_ausente_falha_antes_de_consultar_configuracao_ou_rede() {
    let nome = std::env::temp_dir().join(format!(
        "ies-pagina-ausente-{}-{}.html",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(!nome.exists());
    assert_eq!(
        subcodigo(&executar_com_complemento(&nome)),
        "arquivo_privado_ausente_ou_inacessivel"
    );
}

#[cfg(unix)]
#[test]
fn complemento_simbolico_falha_antes_de_consultar_configuracao_ou_rede() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let pasta = std::env::temp_dir().join(format!(
        "ies-pagina-atalho-{}-{}.dir",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&pasta).unwrap();
    let arquivo = pasta.join("pagina.html");
    let atalho = pasta.join("atalho.html");
    std::fs::write(&arquivo, "<html>captura privada</html>").unwrap();
    std::fs::set_permissions(&arquivo, std::fs::Permissions::from_mode(0o600)).unwrap();
    symlink(&arquivo, &atalho).unwrap();

    assert_eq!(
        subcodigo(&executar_com_complemento(&atalho)),
        "arquivo_privado_link_simbolico"
    );
    std::fs::remove_dir_all(&pasta).unwrap();
}
