//! BDD-620-01/02: recusa de origem via CLI real, com escuta local sem tráfego.
//! Não usa credenciais, rede externa, cassette sintética ou SEI.
use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const NUP: &str = "00001.000001/2026-01";

fn privado(caminho: &Path, conteudo: &str) {
    fs::write(caminho, conteudo).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(caminho, fs::Permissions::from_mode(0o600)).unwrap();
    }
}

fn executar(args: &[&str]) -> Output {
    let pasta = std::env::temp_dir().join(format!(
        "ies-620-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&pasta).unwrap();

    // O endereço é deliberadamente não autorizado. Se a guarda falhar,
    // uma eventual tentativa de conexão será detectada nesta escuta.
    let escuta = TcpListener::bind("127.0.0.1:0").unwrap();
    escuta.set_nonblocking(true).unwrap();
    let origem = format!(
        "https://127.0.0.1:{}/sei",
        escuta.local_addr().unwrap().port()
    );
    let url = format!(
        "{origem}/controlador.php?acao=procedimento_controlar&infra_hash={}",
        "a".repeat(64)
    );
    let arquivo_origem = pasta.join("origem");
    let arquivo_url = pasta.join("url");
    let arquivo_cookie = pasta.join("cookie");
    privado(&arquivo_origem, &origem);
    privado(&arquivo_url, &url);
    privado(&arquivo_cookie, "PHPSESSID=SINTETICO");

    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env("IES_ORIGEM_FILE", &arquivo_origem)
        .env("IES_URL_CONTROLE_FILE", &arquivo_url)
        .env("IES_COOKIE_FILE", &arquivo_cookie)
        .env("IES_INDICE_FILE", pasta.join("indice.sqlite3"))
        .env("HOME", &pasta)
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE")
        .env_remove("HTTP_PROXY")
        .env_remove("HTTPS_PROXY")
        .env("NO_PROXY", "*")
        .output()
        .unwrap();

    let tentativa = escuta.accept();
    assert!(
        tentativa
            .as_ref()
            .is_err_and(|erro| erro.kind() == std::io::ErrorKind::WouldBlock),
        "zero conexões, inclusive GET: {tentativa:?}"
    );
    fs::remove_dir_all(&pasta).unwrap();
    saida
}

#[test]
fn bdd_620_01_andamento_humano_recusa_antes_de_qualquer_requisicao() {
    let saida = executar(&[
        "processo",
        "andamento",
        NUP,
        "--texto",
        "Texto de teste",
        "--confirmar",
    ]);
    assert_eq!(saida.status.code(), Some(1));
    assert!(saida.stdout.is_empty());
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(erro.contains("origem da sessão"), "{erro}");
    assert!(erro.contains("[origem_fora_do_treinamento]"), "{erro}");
}

#[test]
fn bdd_620_02_atribuicao_json_recusa_antes_de_qualquer_requisicao() {
    let saida = executar(&[
        "--json",
        "processo",
        "atribuir",
        NUP,
        "--usuario",
        "aluno2",
        "--confirmar",
    ]);
    assert_eq!(saida.status.code(), Some(1));
    let resposta: serde_json::Value =
        serde_json::from_slice(&saida.stdout).expect("JSON de recusa");
    assert_eq!(resposta["codigo"], "fonte_invalida");
    assert_eq!(resposta["subcodigo"], "origem_fora_do_treinamento");
    assert_eq!(resposta["retentavel"], false);
    assert!(
        resposta["erro"]
            .as_str()
            .unwrap()
            .contains("origem da sessão")
    );
    assert!(resposta.get("status_http").is_none());
    assert!(resposta.get("mensagem_do_servidor").is_none());
}
