use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

const HTML: &str = r#"<html><body>
<table id="tblProcessosRecebidos"><caption>2 registros:</caption>
<tr><th>Número</th></tr>
<tr id="P42"><td><a class="processoVisualizado" href="controlador.php?acao=procedimento_trabalhar&amp;infra_hash=SEGREDO">23106.000001/2026-01</a></td></tr>
<tr id="P43"><td><a class="processoNaoVisualizado" href="controlador.php?acao=procedimento_trabalhar">23106.000002/2026-02</a></td></tr>
</table>
<table id="tblProcessosGerados"><caption>0 registros:</caption></table>
</body></html>"#;

fn preparar_fixture() -> (PathBuf, PathBuf, PathBuf) {
    let folder = std::env::temp_dir().join(format!(
        "ies-replay-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&folder).unwrap();
    let zip = folder.join("replay.zip");
    let member = "unidade-processos-unb.json";
    let cassette = serde_json::json!({
        "complete": true, "schema_version": 1,
        "sei_root": "https://sei.treinamento.unb.br/",
        "interactions": [{
            "request":{"method":"GET","operation":"procedimento_controlar"},
            "response":{
                "status_code":200,
                "encoding":"utf-8",
                "headers":{"content-type":"text/html; charset=utf-8"},
                "body_base64": STANDARD.encode(HTML.as_bytes())
            }
        }]
    });
    let json_bytes = cassette.to_string();
    let mut writer = zip::ZipWriter::new(File::create(&zip).unwrap());
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    writer.start_file(member, options).unwrap();
    writer.write_all(json_bytes.as_bytes()).unwrap();
    writer.finish().unwrap();

    let card = folder.join("replay.md");
    fs::write(
        &card,
        format!(
            "---\ntype: Cassette\ninstallation: unb\nsource_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip\narchive_member: {member}\narchive_sha1: {:x}\ncassette_sha256_lf: {:x}\n---\n\n# Replay\n",
            Sha1::digest(fs::read(&zip).unwrap()),
            Sha256::digest(json_bytes.as_bytes()),
        ),
    )
    .unwrap();
    (folder, card, zip)
}

#[test]
fn replay_valido_lê_duas_tabelas_sem_url_assinada() {
    let (folder, card, zip) = preparar_fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "unidade",
            "listar-processos",
            "--cassette-card",
            card.to_str().unwrap(),
            "--zip-file",
            zip.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["completude"], "completa");
    assert_eq!(result["recebidos"]["total_declarado"], 2);
    assert_eq!(result["gerados"]["total_declarado"], 0);
    assert_eq!(result["recebidos"]["processos"][0]["id"], "42");
    assert_eq!(result["recebidos"]["processos"][1]["visualizado"], false);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("SEGREDO"));
    assert!(!stdout.contains("infra_hash"));
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn hash_adulterado_é_recusado() {
    let (folder, card, zip) = preparar_fixture();
    let text = fs::read_to_string(&card)
        .unwrap()
        .replace("archive_sha1: ", "archive_sha1: 0000");
    fs::write(&card, text).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "unidade",
            "listar-processos",
            "--cassette-card",
            card.to_str().unwrap(),
            "--zip-file",
            zip.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["codigo"], "fonte_invalida");
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn comando_sem_fonte_conserva_stubs() {
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["unidade", "listar-processos", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["codigo"], "nao_implementado");
}
