use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn acervo_de_documentos(titulos: &[&str]) -> PathBuf {
    let pasta = std::env::temp_dir().join(format!(
        "ies-lista-documentos-{}-{}",
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
        .map(|titulo| serde_json::json!({
            "request": {"method":"GET", "operation":"documento_imprimir_web"},
            "response": {
                "status_code":200,"encoding":"utf-8",
                "headers":{"content-type":"text/html; charset=utf-8"},
                "body_base64":STANDARD.encode(format!("<html><head><title>{titulo}</title></head><body>FICTICIO</body></html>"))
            }
        }))
        .collect();
    let dados = serde_json::json!({
        "schema_version":1,"complete":true,"sei_root":"https://sei.treinamento.unb.br/",
        "interactions":interacoes
    })
    .to_string();
    let mut arquivo = zip::ZipWriter::new(File::create(&zip).unwrap());
    arquivo
        .start_file(
            "documentos-unb.json",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    arquivo.write_all(dados.as_bytes()).unwrap();
    arquivo.finish().unwrap();
    let card = pasta.join("cassettes/unb/documentos--synthetic.md");
    fs::create_dir_all(card.parent().unwrap()).unwrap();
    fs::write(card,format!(
        "---\ntype: Cassette\ninstallation: unb\nsource_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip\narchive_member: documentos-unb.json\narchive_sha1: {:x}\ncassette_sha256_lf: {:x}\n---\n",
        Sha1::digest(fs::read(&zip).unwrap()),Sha256::digest(dados.as_bytes())
    )).unwrap();
    pasta
}
fn executar(pasta: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "documentos",
            "--acervo",
            pasta.to_str().unwrap(),
            "--documento",
            "--listar-documentos",
            "--json",
        ])
        .output()
        .unwrap()
}
#[test]
fn lista_todas_as_identidades_sem_expor_conteudos() {
    let pasta = acervo_de_documentos(&["SEI/UnB - 0007 - Ofício", "SEI/UnB - 0008 - Memorando"]);
    let saida = executar(&pasta);
    assert_eq!(saida.status.code(), Some(0), "{saida:?}");
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["documentos"][0]["numero"], "0007");
    assert_eq!(json["documentos"][1]["tipo"], "Memorando");
    assert_eq!(json["escopo"], "titulos_documentos_offline");
    assert!(!String::from_utf8_lossy(&saida.stdout).contains("FICTICIO"));
    fs::remove_dir_all(pasta).unwrap();
}
#[test]
fn repeticao_nao_e_disfarcada_de_inventario() {
    let pasta = acervo_de_documentos(&["SEI/UnB - 0007 - Ofício", "SEI/UnB - 0007 - Ofício"]);
    let saida = executar(&pasta);
    assert_eq!(saida.status.code(), Some(1), "{saida:?}");
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["subcodigo"], "numero_documento_repetido_na_cassette");
    fs::remove_dir_all(pasta).unwrap();
}
