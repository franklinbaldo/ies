//! A saída local inviável deve ser recusada antes de um GET do Controle.
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

fn comando(args: &[&str]) -> Value {
    let resultado = Command::new(env!("CARGO_BIN_EXE_ies"))
        .arg("--json")
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .expect("executa CLI sem credenciais");
    assert!(!resultado.status.success(), "recusa sem tocar na rede");
    serde_json::from_slice(&resultado.stdout).expect("erro JSON tipado")
}

fn raiz() -> std::path::PathBuf {
    let pasta = std::env::temp_dir().join(format!(
        "ies-caixa-preflight-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&pasta).unwrap();
    pasta
}

fn caminho(p: &Path) -> &str {
    p.to_str().expect("pasta UTF-8 de teste")
}

#[test]
fn unidade_sem_diretorio_recusa_antes_da_configuracao() {
    let base = raiz();
    let destino = base.join("ausente").join("caixa.json");
    let erro = comando(&["unidade", "listar-processos", "--saida", caminho(&destino)]);
    assert_eq!(erro["subcodigo"], "diretorio_do_arquivo_da_caixa_ausente");
    assert!(!destino.exists());
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn categoria_com_destino_ja_ocupado_nao_consulta_sessao() {
    let base = raiz();
    let destino = base.join("caixa.json");
    fs::write(&destino, "ORIGINAL").unwrap();
    let erro = comando(&[
        "unidade",
        "listar-processos",
        "--categoria",
        "gerados",
        "--saida",
        caminho(&destino),
    ]);
    assert_eq!(erro["subcodigo"], "arquivo_da_caixa_ja_existe");
    assert_eq!(fs::read_to_string(&destino).unwrap(), "ORIGINAL");
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn resumo_com_destino_existente_nao_consulta_sessao() {
    let base = raiz();
    let destino = base.join("resumo.json");
    fs::write(&destino, "ORIGINAL").unwrap();
    let erro = comando(&[
        "processo",
        "000042/2026",
        "--resumo",
        "--saida",
        caminho(&destino),
    ]);
    assert_eq!(erro["subcodigo"], "arquivo_da_caixa_ja_existe");
    assert_eq!(fs::read_to_string(&destino).unwrap(), "ORIGINAL");
    fs::remove_dir_all(base).unwrap();
}

#[cfg(unix)]
#[test]
fn destino_symlink_pendente_conta_como_existente_antes_de_get() {
    use std::os::unix::fs::symlink;
    let base = raiz();
    let destino = base.join("caixa.json");
    symlink(base.join("inexistente"), &destino).unwrap();
    let erro = comando(&["unidade", "listar-processos", "--saida", caminho(&destino)]);
    assert_eq!(erro["subcodigo"], "arquivo_da_caixa_ja_existe");
    fs::remove_dir_all(base).unwrap();
}

#[cfg(unix)]
#[test]
fn diretorio_simbolico_recusado_antes_da_configuracao() {
    use std::os::unix::fs::symlink;
    let base = raiz();
    let real = base.join("real");
    let alias = base.join("alias");
    fs::create_dir(&real).unwrap();
    symlink(&real, &alias).unwrap();
    let destino = alias.join("caixa.json");
    let erro = comando(&["unidade", "listar-processos", "--saida", caminho(&destino)]);
    assert_eq!(
        erro["subcodigo"],
        "diretorio_do_arquivo_da_caixa_link_simbolico"
    );
    assert!(!real.join("caixa.json").exists());
    fs::remove_dir_all(base).unwrap();
}
