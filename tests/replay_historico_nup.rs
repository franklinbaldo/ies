use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn acervo_com_historico(numero: &str) -> PathBuf {
    let pasta = std::env::temp_dir().join(format!(
        "ies-historico-nup-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let arquivo_zip = pasta.join(".cache/ies/todos-sei-cassettes-unb.zip");
    fs::create_dir_all(arquivo_zip.parent().unwrap()).unwrap();
    let html = format!(
        "<html><head><title>SEI - Histórico do Processo {numero}</title></head>\
         <body><table id='tblHistorico'><tr><td>30/10/2026</td><td>Unidade A</td>\
         <td>usuario</td><td>Processo recebido</td></tr></table></body></html>"
    );
    let json = serde_json::json!({
        "schema_version":1,
        "complete":true,
        "sei_root":"https://sei.treinamento.unb.br/",
        "interactions":[{
            "request":{"method":"GET","operation":"procedimento_consultar_historico"},
            "response":{
                "status_code":200,
                "encoding":"utf-8",
                "headers":{"content-type":"text/html; charset=utf-8"},
                "body_base64":STANDARD.encode(html.as_bytes())
            }
        }]
    })
    .to_string();
    let mut writer = zip::ZipWriter::new(File::create(&arquivo_zip).unwrap());
    writer
        .start_file(
            "historico-sei-unb.json",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    writer.write_all(json.as_bytes()).unwrap();
    writer.finish().unwrap();
    let card = pasta.join("cassettes/unb/historico-sei-unb--synthetic.md");
    fs::create_dir_all(card.parent().unwrap()).unwrap();
    fs::write(
        card,
        format!(
            "---\ntype: Cassette\ninstallation: unb\n\
             source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip\n\
             archive_member: historico-sei-unb.json\narchive_sha1: {:x}\n\
             cassette_sha256_lf: {:x}\n---\n",
            Sha1::digest(fs::read(&arquivo_zip).unwrap()),
            Sha256::digest(json.as_bytes())
        ),
    )
    .unwrap();
    pasta
}

fn consultar(pasta: &Path, numero: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "historico-sei-unb",
            "--acervo",
            pasta.to_str().unwrap(),
            "--numero-processo",
            numero,
            "--json",
        ])
        .output()
        .unwrap()
}

#[test]
fn verifica_nup_opaco_exatamente_e_preserva_zeros() {
    let pasta = acervo_com_historico("000123/2026-X");
    let output = consultar(&pasta, "000123/2026-X");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["processo"]["numero"], "000123/2026-X");
    assert_eq!(
        json["processo"]["andamentos"][0]["descricao"],
        "Processo recebido"
    );
    assert_eq!(json["escopo"], "historico_offline");
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn numero_diferente_nao_e_inferido_pelo_titulo() {
    let pasta = acervo_com_historico("000123/2026-X");
    let output = consultar(&pasta, "123/2026-X");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["subcodigo"],
        "numero_do_historico_diferente_do_solicitado"
    );
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn entrada_contendo_caractere_de_controle_e_recusada_com_erro_tipado() {
    let pasta = acervo_com_historico("000123/2026-X");
    let output = consultar(&pasta, "000123\nSECRETO");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["codigo"], "entrada_invalida");
    assert_eq!(json["subcodigo"], "numero_processo_controle");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("SECRETO"));
    fs::remove_dir_all(pasta).unwrap();
}
