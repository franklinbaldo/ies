use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn acervo_de_documentos(titulos: &[&str]) -> PathBuf {
    let pasta = std::env::temp_dir().join(format!(
        "ies-doc-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let zip = pasta.join(".cache/ies/todos-sei-cassettes-unb.zip");
    fs::create_dir_all(zip.parent().unwrap()).unwrap();
    let interacoes: Vec<_> = titulos
        .iter()
        .map(|titulo| {
            let html = format!(
                "<html><head><title>{titulo}</title></head><body>SEM SEGREDOS</body></html>"
            );
            serde_json::json!({
                "request": {"method":"GET", "operation":"documento_imprimir_web"},
                "response": {
                    "status_code": 200, "encoding": "utf-8",
                    "headers":{"content-type":"text/html; charset=utf-8"},
                    "body_base64":STANDARD.encode(html.as_bytes())
                }
            })
        })
        .collect();
    let member = "documentos-unb.json";
    let dados = serde_json::json!({
        "complete": true, "schema_version":1, "sei_root":"https://sei.treinamento.unb.br/",
        "interactions":interacoes
    })
    .to_string();
    let mut arquivo = zip::ZipWriter::new(File::create(&zip).unwrap());
    arquivo
        .start_file(member, zip::write::SimpleFileOptions::default())
        .unwrap();
    arquivo.write_all(dados.as_bytes()).unwrap();
    arquivo.finish().unwrap();
    let card = pasta.join("cassettes/unb/documentos--synthetic.md");
    fs::create_dir_all(card.parent().unwrap()).unwrap();
    fs::write(&card, format!(
        "---\ntype: Cassette\ninstallation: unb\nsource_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip\narchive_member: {member}\narchive_sha1: {:x}\ncassette_sha256_lf: {:x}\n---\n",
        Sha1::digest(fs::read(&zip).unwrap()), Sha256::digest(dados.as_bytes())
    )).unwrap();
    pasta
}

fn consultar(pasta: &Path, numero: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "documentos",
            "--acervo",
            pasta.to_str().unwrap(),
            "--documento",
            "--numero-documento",
            numero,
            "--json",
        ])
        .output()
        .unwrap()
}

#[test]
fn encontra_numero_sei_exato_em_varias_respostas_verificadas() {
    let pasta = acervo_de_documentos(&[
        "SEI/UnB - 0011354 - Ofício",
        "SEI/UnB - 0000007 - Memorando",
    ]);
    let resposta = consultar(&pasta, "0000007");
    assert_eq!(resposta.status.code(), Some(0), "{resposta:?}");
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(json["documento"]["numero"], "0000007");
    assert_eq!(json["documento"]["tipo"], "Memorando");
    assert_eq!(json["instalacao"], "unb");
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("SEM SEGREDOS"));
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn nao_considera_documento_sem_zeros_equivalente_ao_identificado() {
    let pasta = acervo_de_documentos(&["SEI/UnB - 0011354 - Ofício"]);
    let resposta = consultar(&pasta, "11354");
    assert_eq!(resposta.status.code(), Some(1), "{resposta:?}");
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(
        json["subcodigo"],
        "numero_documento_nao_observado_na_cassette"
    );
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn recusa_numero_de_documento_duplicado_na_cassette() {
    let pasta = acervo_de_documentos(&["SEI/UnB - 0011354 - Ofício", "SEI/UnB - 0011354 - Ofício"]);
    let resposta = consultar(&pasta, "0011354");
    assert_eq!(resposta.status.code(), Some(1), "{resposta:?}");
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(json["subcodigo"], "numero_documento_repetido_na_cassette");
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn numero_nao_decimal_e_recusado_antes_da_escolha() {
    let pasta = acervo_de_documentos(&["SEI/UnB - 0011354 - Ofício"]);
    let resposta = consultar(&pasta, "001A");
    assert_eq!(resposta.status.code(), Some(2), "{resposta:?}");
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(json["subcodigo"], "numero_documento_nao_numerico");
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn exporta_texto_verificado_sem_divulgar_conteudo_no_terminal() {
    let pasta = acervo_de_documentos(&["SEI/UnB - 0007 - Ofício"]);
    let destino = pasta.join("0007.txt");
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "documentos",
            "--acervo",
            pasta.to_str().unwrap(),
            "--documento",
            "--numero-documento",
            "0007",
            "--texto-documento",
            "--arquivo-texto",
            destino.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(saida.status.code(), Some(0), "{saida:?}");
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["escopo"], "texto_documento_exportado_offline");
    assert_eq!(json["documento"]["numero"], "0007");
    assert_eq!(fs::read_to_string(&destino).unwrap(), "SEM SEGREDOS");
    assert!(!String::from_utf8_lossy(&saida.stdout).contains("SEM SEGREDOS"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&destino).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn exportacao_recusa_sobrescrever_texto_preexistente() {
    let pasta = acervo_de_documentos(&["SEI/UnB - 0007 - Ofício"]);
    let destino = pasta.join("0007.txt");
    fs::write(&destino, "MANTER").unwrap();
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "documentos",
            "--acervo",
            pasta.to_str().unwrap(),
            "--documento",
            "--numero-documento",
            "0007",
            "--texto-documento",
            "--arquivo-texto",
            destino.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(saida.status.code(), Some(1), "{saida:?}");
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["subcodigo"], "arquivo_texto_documento_ja_existe");
    assert_eq!(fs::read_to_string(&destino).unwrap(), "MANTER");
    fs::remove_dir_all(pasta).unwrap();
}
