//! #851: CLI real rejeita origens não autorizadas antes de transporte.
//! Teste offline de fronteira, não captura nem simula o POST nativo do SEI.
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
fn documento_salvar_recusa_origens_sem_transporte_nativo() {
    let pasta = std::env::temp_dir().join(format!(
        "ies-salvar-origem-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&pasta).unwrap();

    // A escuta atua como proxy de teste. Ela detecta tentativas que respeitem
    // HTTPS_PROXY; não prova, isoladamente, a ausência de todos os sockets.
    // Em particular, o teste deve também rodar em executor sem rede.
    let escuta = TcpListener::bind("127.0.0.1:0").unwrap();
    escuta.set_nonblocking(true).unwrap();
    let porta = escuta.local_addr().unwrap().port();
    let proxy = format!("http://127.0.0.1:{porta}");
    let origem_com_porta = format!("https://127.0.0.1:{porta}/sei");

    arquivo_privado(&pasta.join("cookie"), "PHPSESSID=TESTE_LOCAL");
    arquivo_privado(&pasta.join("0011519.v2.html"), "<p>Foto</p>");
    arquivo_privado(&pasta.join("0011519.v3.html"), "<p>Edição</p>");

    // Exercitar a CLI, e não apenas a função exigir: origem parseável
    // recusada, origem ilegível, treinamento permitido sem editor nativo
    // e hostname semelhante que não pertence à lista fechada. Não há
    // acesso deliberado a esses hosts: o comando recusa antes do transporte;
    // o proxy local detecta somente conexões que respeitem suas variáveis.
    for (origem, erro_esperado) in [
        ("https://127.0.0.1/sei", "OrigemForaDoTreinamento"),
        (&origem_com_porta, "InstalacaoSemEditorObservado"),
        (
            "https://sei.treinamento.unb.br/sei",
            "InstalacaoSemEditorObservado",
        ),
        (
            "https://sei.treinamento.unb.br.exemplo/sei",
            "OrigemForaDoTreinamento",
        ),
    ] {
        let url_controle = format!(
            "{origem}/controlador.php?acao=procedimento_controlar&infra_hash={}",
            "a".repeat(64)
        );
        arquivo_privado(&pasta.join("origem"), origem);
        arquivo_privado(&pasta.join("url"), &url_controle);

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
                .env("HTTP_PROXY", &proxy)
                .env("HTTPS_PROXY", &proxy)
                .env("ALL_PROXY", &proxy)
                .env("NO_PROXY", "");
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
                erro.starts_with(&format!("Error: {erro_esperado}")),
                "recusa tipada esperada ({erro_esperado}): {erro}"
            );
            let tentativa = escuta.accept();
            assert!(
                tentativa
                    .as_ref()
                    .is_err_and(|erro| erro.kind() == std::io::ErrorKind::WouldBlock),
                "zero conexões via proxy de teste: {tentativa:?}"
            );
        }
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
