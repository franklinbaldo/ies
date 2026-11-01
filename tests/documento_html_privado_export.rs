//! Teste da CLI efetiva: exportação local sem rede nem exposição de texto.
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn arquivo_html_privado(destino: &Path) {
    let html = "<html><head><title>SEI/UnB - 0007 - Ofício</title></head>\
                <body><svg><title>Ícone do SEI</title></svg>\
                <template><p>Texto de modelo oculto</p></template>\
                <p>Texto privado fictício.</p></body></html>";
    fs::write(destino, html).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(destino, fs::Permissions::from_mode(0o600)).unwrap();
    }
}

fn executar(numero: &str, fonte: &Path, destino: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .arg("documento")
        .arg(numero)
        .arg("--html-privado")
        .arg(fonte)
        .arg("--saida")
        .arg(destino)
        .arg("--json")
        .output()
        .unwrap()
}

#[test]
fn cli_exporta_privado_sem_vazar_texto_e_recusa_sobrescrita_ou_outro_numero() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let raiz = std::env::temp_dir().join(format!("ies-html-export-{}-{nanos}", std::process::id()));
    fs::create_dir(&raiz).unwrap();
    let fonte = raiz.join("fonte.html");
    arquivo_html_privado(&fonte);
    let pasta = raiz.join("exportacao");
    fs::create_dir(&pasta).unwrap();

    let primeiro = executar("0007", &fonte, &pasta);
    assert_eq!(primeiro.status.code(), Some(0), "{primeiro:?}");
    let resposta: serde_json::Value = serde_json::from_slice(&primeiro.stdout).unwrap();
    assert_eq!(resposta["documento"]["numero"], "0007");
    assert_eq!(resposta["escopo"], "html_privado_texto_exportado_offline");
    assert_eq!(resposta["autenticacao_remota"], false);
    assert!(resposta["versao_atual"].is_null());
    assert!(resposta.get("texto").is_none());
    assert!(!String::from_utf8_lossy(&primeiro.stdout).contains("Texto privado"));
    assert_eq!(primeiro.stderr, Vec::<u8>::new());

    let texto = fs::read_to_string(pasta.join("0007.txt")).unwrap();
    assert_eq!(texto, "Texto privado fictício.");
    assert!(!texto.contains("Ícone do SEI"));
    assert!(!texto.contains("Texto de modelo oculto"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(pasta.join("0007.txt"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    let repetido = executar("0007", &fonte, &pasta);
    assert_eq!(repetido.status.code(), Some(1));
    let erro: serde_json::Value = serde_json::from_slice(&repetido.stdout).unwrap();
    assert_eq!(erro["subcodigo"], "arquivo_texto_documento_ja_existe");
    assert_eq!(fs::read_to_string(pasta.join("0007.txt")).unwrap(), texto);

    let outro = executar("7", &fonte, &pasta);
    assert_eq!(outro.status.code(), Some(1));
    let erro: serde_json::Value = serde_json::from_slice(&outro.stdout).unwrap();
    assert_eq!(erro["subcodigo"], "numero_do_html_diferente_do_solicitado");
    assert!(!pasta.join("7.txt").exists());
    fs::remove_dir_all(raiz).unwrap();
}
