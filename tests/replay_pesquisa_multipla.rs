use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn acervo_de_pesquisa(numeros: &[&str]) -> PathBuf {
    let pasta = std::env::temp_dir().join(format!(
        "ies-pesquisa-multipla-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let zip = pasta.join(".cache/ies/todos-sei-cassettes-unb.zip");
    fs::create_dir_all(zip.parent().unwrap()).unwrap();
    let interacoes: Vec<_> = numeros
        .iter()
        .enumerate()
        .map(|(i, numero)| {
            let html = format!(
                r#"<html><table class="pesquisaResultado"><tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento={}">{numero}</a></td></tr></table></html>"#,
                i + 10
            );
            serde_json::json!({
                "request":{"method":"POST","operation":"protocolo_pesquisar"},
                "response":{"status_code":200,"encoding":"utf-8",
                    "headers":{"content-type":"text/html; charset=utf-8"},
                    "body_base64":STANDARD.encode(html.as_bytes())}
            })
        })
        .collect();
    let dados = serde_json::json!({
        "schema_version":1,"complete":true,"sei_root":"https://sei.treinamento.unb.br/",
        "interactions":interacoes
    })
    .to_string();
    let mut writer = zip::ZipWriter::new(File::create(&zip).unwrap());
    writer
        .start_file(
            "pesquisa-unb.json",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    writer.write_all(dados.as_bytes()).unwrap();
    writer.finish().unwrap();
    let card = pasta.join("cassettes/unb/pesquisa-multipla--synthetic.md");
    fs::create_dir_all(card.parent().unwrap()).unwrap();
    fs::write(
        card,
        format!(
            "---\ntype: Cassette\ninstallation: unb\nsource_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip\narchive_member: pesquisa-unb.json\narchive_sha1: {:x}\ncassette_sha256_lf: {:x}\n---\n",
            Sha1::digest(fs::read(&zip).unwrap()),
            Sha256::digest(dados.as_bytes())
        ),
    )
    .unwrap();
    pasta
}

fn consultar(pasta: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "pesquisa-multipla",
            "--acervo",
            pasta.to_str().unwrap(),
            "--resultado-pesquisa",
            "--unir-resultados-pesquisa",
            "--passo",
            "0",
            "--passo",
            "1",
            "--json",
        ])
        .output()
        .unwrap()
}

#[test]
fn reune_duas_respostas_verificadas_sem_proclamar_cobertura_integral() {
    let pasta = acervo_de_pesquisa(&["00001.123456/2026-07", "00002.123456/2026-07"]);
    let output = consultar(&pasta);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let valor: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(valor["escopo"], "respostas_pesquisa_cassette_offline");
    assert_eq!(valor["cobertura"], "somente_passos_selecionados");
    assert_eq!(valor["passos"], serde_json::json!([0, 1]));
    assert_eq!(valor["total_observado"], 2);
    assert_eq!(valor["processos"][0]["numero"], "00001.123456/2026-07");
    assert_eq!(valor["processos"][1]["numero"], "00002.123456/2026-07");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("id_procedimento"));
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn nup_repetido_em_respostas_distintas_recusa_consolidacao() {
    let pasta = acervo_de_pesquisa(&["00001.123456/2026-07", "00001.123456/2026-07"]);
    let output = consultar(&pasta);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let valor: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        valor["subcodigo"],
        "numero_de_pesquisa_repetido_entre_respostas"
    );
    assert!(valor.get("processos").is_none());
    fs::remove_dir_all(pasta).unwrap();
}
