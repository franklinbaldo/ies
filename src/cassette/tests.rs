//! Conferência das cassettes com um acervo SINTÉTICO (ZIP e card gerados no
//! teste). Portado de `tests/replay_unidade.rs` (`hash_adulterado_é_recusado`),
//! que conferia o mesmo pelo antigo `ies replay`.
use super::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::io::Write as _;
use std::path::{Path, PathBuf};

const LIMITE: u64 = 30_000_000;
const HTML: &str = "<html><body><a id=\"lnkInfraUnidade\">TESTE</a></body></html>";

struct Acervo {
    pasta: PathBuf,
    card: PathBuf,
    zip: PathBuf,
}

impl Drop for Acervo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.pasta);
    }
}

fn acervo(nome: &str, sei_root: &str) -> Acervo {
    let pasta = std::env::temp_dir().join(format!(
        "ies-cassette-{nome}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    let membro = "controle-unb.json";
    let json = serde_json::json!({
        "complete": true, "schema_version": 1, "sei_root": sei_root,
        "interactions": [{
            "request": {"method": "GET", "operation": "procedimento_controlar"},
            "response": {
                "status_code": 200, "encoding": "utf-8",
                "headers": {"content-type": "text/html; charset=utf-8"},
                "body_base64": STANDARD.encode(HTML)
            }
        }]
    })
    .to_string();
    let zip = pasta.join("todos-sei-cassettes-unb.zip");
    let mut escritor = zip::ZipWriter::new(std::fs::File::create(&zip).unwrap());
    escritor
        .start_file(membro, zip::write::SimpleFileOptions::default())
        .unwrap();
    escritor.write_all(json.as_bytes()).unwrap();
    escritor.finish().unwrap();
    let card = pasta.join("controle--synthetic.md");
    std::fs::write(
        &card,
        format!(
            "---\ntype: Cassette\ninstallation: unb\nsource_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip\narchive_member: {membro}\narchive_sha1: {}\ncassette_sha256_lf: {}\n---\n",
            hex::encode(Sha1::digest(std::fs::read(&zip).unwrap())),
            hex::encode(Sha256::digest(json.as_bytes())),
        ),
    )
    .unwrap();
    Acervo { pasta, card, zip }
}

fn conferir(card: &Path, zip: &Path) -> Result<super::conferencia::CassetteConferida, String> {
    conferir_cassette_ate(card, &ZipDoAcervo::ler(zip)?, LIMITE)
}

fn adulterar(card: &Path, campo: &str) {
    let texto = std::fs::read_to_string(card).unwrap();
    let linha = texto
        .lines()
        .find(|l| l.starts_with(&format!("{campo}: ")))
        .unwrap();
    let valor = &linha[campo.len() + 2..];
    let outro: String = valor
        .chars()
        .map(|c| if c == '0' { '1' } else { '0' })
        .collect();
    std::fs::write(card, texto.replace(linha, &format!("{campo}: {outro}"))).unwrap();
}

#[test]
fn cassette_integra_e_conferida_e_o_html_e_lido() {
    let a = acervo("integra", "https://sei.treinamento.unb.br/");
    let conferida = conferir(&a.card, &a.zip).unwrap();
    assert_eq!(conferida.instalacao, "unb");
    let html = html_da_resposta(&conferida.json["interactions"][0]).unwrap();
    assert_eq!(html, HTML);
    assert_eq!(
        archive_filename(&a.card).unwrap(),
        "todos-sei-cassettes-unb.zip"
    );
}

#[test]
fn hash_adulterado_do_zip_ou_da_cassette_e_recusado() {
    let a = acervo("sha1", "https://sei.treinamento.unb.br/");
    adulterar(&a.card, "archive_sha1");
    assert_eq!(
        conferir(&a.card, &a.zip).err().as_deref(),
        Some("SHA-1 do ZIP não confere")
    );
    let b = acervo("sha256", "https://sei.treinamento.unb.br/");
    adulterar(&b.card, "cassette_sha256_lf");
    assert_eq!(
        conferir(&b.card, &b.zip).err().as_deref(),
        Some("SHA-256 da cassette não confere")
    );
}

#[test]
fn host_fora_da_instalacao_do_card_e_recusado() {
    let a = acervo("host", "https://sei.adulterado.tld/");
    assert_eq!(
        conferir(&a.card, &a.zip).err().as_deref(),
        Some("host da cassette não corresponde à instalação")
    );
}
