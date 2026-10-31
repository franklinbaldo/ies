use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn pasta_teste() -> PathBuf {
    let agora = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ies-status-{}-{agora}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}
fn arquivo(dir: &Path, nome: &str) -> PathBuf {
    let path = dir.join(nome);
    fs::write(&path, "DADO FICTICIO").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    path
}
fn executar(url: &Path, cookie: &Path, origem: &Path, verificar: bool) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ies"));
    cmd.args(["status", "--json"])
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", url)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem);
    if verificar {
        cmd.arg("--verificar-arquivos");
    }
    cmd.output().unwrap()
}

fn executar_sessao_local(url: &Path, cookie: &Path, origem: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["status", "--verificar-sessao-local", "--json"])
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", url)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .output()
        .unwrap()
}

fn arquivo_privado_com(dir: &Path, nome: &str, valor: &str) -> PathBuf {
    let path = arquivo(dir, nome);
    fs::write(&path, valor).unwrap();
    path
}

fn url_assinada_teste() -> String {
    format!(
        "https://sei.exemplo/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
        "a".repeat(64)
    )
}

#[test]
fn preflight_confere_formato_e_origem_sem_rede_nem_vazar_cookie_ou_hash() {
    let dir = pasta_teste();
    let url = arquivo_privado_com(&dir, "url", &url_assinada_teste());
    let cookie = arquivo_privado_com(&dir, "cookie", "PHPSESSID=FICTICIO");
    let origem = arquivo_privado_com(&dir, "origem", "https://sei.exemplo/sei");
    let output = executar_sessao_local(&url, &cookie, &origem);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let resposta: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(resposta["sessao_local"], "formato_e_origem_verificados");
    assert_eq!(resposta["conexao"], "nao_verificada");
    assert_eq!(resposta["acesso_remoto"], false);
    assert_eq!(resposta["autenticacao_remota"], false);
    let saida = String::from_utf8_lossy(&output.stdout);
    assert!(!saida.contains("PHPSESSID"));
    assert!(!saida.contains("infra_hash"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn preflight_recusa_origem_antes_de_tentar_abrir_cookie_inexistente() {
    let dir = pasta_teste();
    let url = arquivo_privado_com(&dir, "url", &url_assinada_teste());
    let origem = arquivo_privado_com(&dir, "origem", "https://outro.exemplo/sei");
    let output = executar_sessao_local(&url, &dir.join("cookie-inexistente"), &origem);
    assert_eq!(output.status.code(), Some(1));
    let resposta: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(resposta["subcodigo"], "origem_nao_autorizada");
    assert_eq!(resposta["acesso_remoto"], false);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn preflight_distingue_cookie_invalido_e_url_sem_assinatura() {
    let dir = pasta_teste();
    let url = arquivo_privado_com(&dir, "url", &url_assinada_teste());
    let cookie = arquivo_privado_com(&dir, "cookie", "Cookie: PHPSESSID=FICTICIO");
    let origem = arquivo_privado_com(&dir, "origem", "https://sei.exemplo/sei");
    let erro_cookie = executar_sessao_local(&url, &cookie, &origem);
    let resposta: serde_json::Value = serde_json::from_slice(&erro_cookie.stdout).unwrap();
    assert_eq!(resposta["subcodigo"], "cookie_invalido");
    fs::write(&url, "https://sei.exemplo/sei/controlador.php?acao=procedimento_controlar")
        .unwrap();
    let erro_url = executar_sessao_local(&url, &cookie, &origem);
    let resposta: serde_json::Value = serde_json::from_slice(&erro_url.stdout).unwrap();
    assert_eq!(resposta["subcodigo"], "url_invalida");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn verifica_metadados_sem_ler_conteudo_e_sem_conectar() {
    let dir = pasta_teste();
    let url = arquivo(&dir, "url");
    let cookie = arquivo(&dir, "cookie");
    let origem = arquivo(&dir, "origem");
    let output = executar(&url, &cookie, &origem, true);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["arquivos_privados"], "metadados_verificados");
    assert_eq!(json["conexao"], "nao_verificada");
    assert_eq!(json["acesso_remoto"], false);
    assert!(!String::from_utf8_lossy(&output.stdout).contains("DADO FICTICIO"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn arquivo_de_cookie_ausente_tem_erro_especifico_e_sem_caminho_exposto() {
    let dir = pasta_teste();
    let url = arquivo(&dir, "url");
    let origem = arquivo(&dir, "origem");
    let cookie = dir.join("cookie-ausente");
    let output = executar(&url, &cookie, &origem, true);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["subcodigo"], "arquivo_privado_ausente");
    assert_eq!(json["arquivo"], "cookie");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("cookie-ausente"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn modo_basico_preserva_diagnostico_sem_inspecionar_arquivos() {
    let dir = pasta_teste();
    let output = executar(
        &dir.join("url"),
        &dir.join("cookie"),
        &dir.join("origem"),
        false,
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["configuracao"], "caminhos_informados");
    assert_eq!(json["conexao"], "nao_verificada");
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn link_simbolico_e_permissoes_inseguras_sao_falhas_distintas() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let dir = pasta_teste();
    let url = arquivo(&dir, "url");
    let cookie = arquivo(&dir, "cookie-real");
    let origem = arquivo(&dir, "origem");
    let elo = dir.join("cookie");
    symlink(&cookie, &elo).unwrap();
    let saida_link = executar(&url, &elo, &origem, true);
    let link: serde_json::Value = serde_json::from_slice(&saida_link.stdout).unwrap();
    assert_eq!(link["subcodigo"], "arquivo_privado_link_simbolico");
    assert_eq!(link["arquivo"], "cookie");
    fs::remove_file(&elo).unwrap();
    fs::set_permissions(&origem, fs::Permissions::from_mode(0o644)).unwrap();
    let saida_permissao = executar(&url, &cookie, &origem, true);
    let permissoes: serde_json::Value = serde_json::from_slice(&saida_permissao.stdout).unwrap();
    assert_eq!(
        permissoes["subcodigo"],
        "arquivo_privado_permissoes_inseguras"
    );
    assert_eq!(permissoes["arquivo"], "origem");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn arquivo_vazio_nao_recebe_selo_de_validacao() {
    let dir = pasta_teste();
    let url = arquivo(&dir, "url");
    let cookie = arquivo(&dir, "cookie");
    let origem = dir.join("origem");
    fs::write(&origem, "").unwrap();
    let output = executar(&url, &cookie, &origem, true);
    assert_eq!(output.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["subcodigo"], "arquivo_privado_vazio");
    assert_eq!(json["arquivo"], "origem");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn caminho_relativo_nao_e_interpretado_a_partir_do_diretorio_atual() {
    let dir = pasta_teste();
    let cookie = arquivo(&dir, "cookie");
    let origem = arquivo(&dir, "origem");
    let output = executar(Path::new("url-relative"), &cookie, &origem, true);
    assert_eq!(output.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["subcodigo"], "caminho_privado_nao_absoluto");
    assert_eq!(json["arquivo"], "url_controle");
    fs::remove_dir_all(dir).unwrap();
}
