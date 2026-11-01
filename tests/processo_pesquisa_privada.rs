//! CLI de domínio: NUP em resposta de pesquisa previamente capturada.
//! Não estabelece sessão, não consulta o SEI e não segue links HTML.
use std::{fs, path::PathBuf, process::Command, time::{SystemTime, UNIX_EPOCH}};

const HTML: &str = r#"<html><table class="pesquisaResultado">
<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal"
href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=7">00001.123456/2026-07</a>
</td></tr></table></html>"#;

fn arquivo(corpo: &str) -> PathBuf {
    let caminho = std::env::temp_dir().join(format!(
        "ies-busca-privada-{}-{}.html",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::write(&caminho, corpo).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&caminho, fs::Permissions::from_mode(0o600)).unwrap();
    }
    caminho
}

fn executar(numero: &str, caminho: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["--json", "processo", numero, "--resultado-pesquisa-file"])
        .arg(caminho)
        .output()
        .unwrap()
}

#[test]
fn sucesso_exibe_apenas_numero_e_escopo_offline_sem_links_ou_html() {
    let caminho = arquivo(HTML);
    let resposta = executar("00001.123456/2026-07", &caminho);
    fs::remove_file(caminho).unwrap();
    assert_eq!(resposta.status.code(), Some(0));
    let saida: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(saida["processo"]["numero"], "00001.123456/2026-07");
    assert_eq!(saida["requisicoes"], 0);
    assert_eq!(saida["autenticacao_remota"], false);
    assert_eq!(saida["escopo"], "resposta_pesquisa_html_privado_offline");
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("controlador.php"));
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("id_procedimento"));
}

#[test]
fn nup_ausente_retorna_numero_nao_observado_sem_alegar_inexistencia() {
    let caminho = arquivo(HTML);
    let resposta = executar("00002.123456/2026-07", &caminho);
    fs::remove_file(caminho).unwrap();
    assert_eq!(resposta.status.code(), Some(1));
    let saida: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(saida["codigo"], "numero_nao_observado");
    assert_eq!(saida["subcodigo"], "numero_nao_observado_na_resposta_de_pesquisa");
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("processo_inexistente"));
}

#[cfg(unix)]
#[test]
fn symlink_de_captura_privada_falha_antes_de_ler() {
    use std::os::unix::fs::symlink;
    let original = arquivo(HTML);
    let atalho = original.with_extension("atalho");
    symlink(&original, &atalho).unwrap();
    let resposta = executar("00001.123456/2026-07", &atalho);
    fs::remove_file(atalho).unwrap();
    fs::remove_file(original).unwrap();
    assert_eq!(resposta.status.code(), Some(1));
    let saida: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(saida["subcodigo"], "arquivo_privado_link_simbolico");
}
