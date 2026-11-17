//! #851: CLI real rejeita uma origem fora de treinamento sem abrir socket.
//! Teste de fronteira local. Não captura nem simula protocolo de POST do SEI.
use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn arquivo_privado(caminho: &Path, conteudo: &str) {
    fs::write(caminho, conteudo).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(caminho, fs::Permissions::from_mode(0o600)).unwrap();
    }
}

#[test]
fn documento_salvar_recusa_origem_nao_autorizada_antes_de_get_ou_post() {
    let pasta = std::env::temp_dir().join(format!(
        "ies-salvar-origem-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&pasta).unwrap();

    // Uma escuta exclusiva de loopback detectaria conexão indevida. Não há
    // DNS, servidor SEI, cookie real ou instalação de produção neste teste.
    let escuta = TcpListener::bind("127.0.0.1:0").unwrap();
    escuta.set_nonblocking(true).unwrap();
    let origem = format!("https://127.0.0.1:{}/sei", escuta.local_addr().unwrap().port());
    let url_controle = format!(
        "{origem}/controlador.php?acao=procedimento_controlar&infra_hash={}",
        "a".repeat(64)
    );

    arquivo_privado(&pasta.join("origem"), &origem);
    arquivo_privado(&pasta.join("url"), &url_controle);
    arquivo_privado(&pasta.join("cookie"), "PHPSESSID=TESTE_LOCAL");
    arquivo_privado(&pasta.join("0011519.v2.html"), "<p>Foto</p>");
    arquivo_privado(&pasta.join("0011519.v3.html"), "<p>Edição</p>");

    for modo_json in [false, true] {
        let mut comando = Command::new(env!("CARGO_BIN_EXE_ies"));
        comando
            .current_dir(&pasta)
            .env("IES_ORIGEM_FILE", pasta.join("origem"))
            .env("IES_URL_CONTROLE_FILE", pasta.join("url"))
            .env("IES_COOKIE_FILE", pasta.join("cookie"))
            .env("IES_INDICE_FILE", pasta.join("indice.sqlite3"))
            .env("HOME", &pasta)
            .env_remove("IES_CONFIG_FILE")
            .env_remove("IES_CREDENCIAIS_FILE")
            .env_remove("HTTP_PROXY")
            .env_remove("HTTPS_PROXY")
            .env("NO_PROXY", "*");
        if modo_json {
            comando.arg("--json");
        }
        let saida = comando
            .args([
                "documento",
                "salvar",
                "0011519",
                "--versao",
                "2",
                "--confirmar",
            ])
            .output()
            .unwrap();

        assert_eq!(
            saida.status.code(),
            Some(1),
            "recusa esperada: {}",
            String::from_utf8_lossy(&saida.stderr)
        );
        assert!(saida.stdout.is_empty(), "nenhuma saída de sucesso");
        let erro = String::from_utf8_lossy(&saida.stderr);
        assert!(
            erro.starts_with("Error: OrigemForaDoTreinamento"),
            "recusa tipada esperada: {erro}"
        );
        let tentativa = escuta.accept();
        assert!(
            tentativa
                .as_ref()
                .is_err_and(|erro| erro.kind() == std::io::ErrorKind::WouldBlock),
            "zero conexões, inclusive GET: {tentativa:?}"
        );
    }

    assert_eq!(
        fs::read_to_string(pasta.join("0011519.v2.html")).unwrap(),
        "<p>Foto</p>"
    );
    assert_eq!(
        fs::read_to_string(pasta.join("0011519.v3.html")).unwrap(),
        "<p>Edição</p>"
    );
    fs::remove_dir_all(pasta).unwrap();
}
