//! Conferência integral de uma cassette pública antes de qualquer leitura:
//! card OKF, SHA-1 do ZIP, SHA-256 do membro após CRLF→LF, `schema_version`,
//! completude e host da instalação. O ZIP é lido e conferido uma vez
//! ([`ZipDoAcervo`]) e pode servir a vários cards do mesmo lote.
use super::campo;
use crate::transporte::corpo::legivel;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::io::{Cursor, Read};
use std::path::Path;
use zip::ZipArchive;

/// Bytes de um ZIP do acervo e o SHA-1 calculado deles.
pub(crate) struct ZipDoAcervo {
    raw: Vec<u8>,
    sha1: String,
}

impl ZipDoAcervo {
    pub(crate) fn ler(archive: &Path) -> Result<Self, String> {
        let metadata = archive.metadata().map_err(|_| "ZIP não encontrado")?;
        if metadata.len() > 160_000_000 {
            return Err("ZIP maior que 160 MB".into());
        }
        let raw = std::fs::read(archive).map_err(|_| "falha ao ler ZIP")?;
        let sha1 = hex::encode(Sha1::digest(&raw));
        Ok(Self { raw, sha1 })
    }
}

/// A cassette conferida e a instalação declarada pelo card.
pub(crate) struct CassetteConferida {
    pub(crate) json: serde_json::Value,
    pub(crate) instalacao: String,
}

/// Conferência integral com teto para o membro JSON (o replay do acervo lê
/// também as cassettes grandes de pastas, de até ~40 MB).
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub(crate) fn conferir_cassette_ate(
    card: &Path,
    zip: &ZipDoAcervo,
    limite: u64,
) -> Result<CassetteConferida, String> {
    let card_text = std::fs::read_to_string(card).map_err(|_| "card não encontrado")?;
    if campo(&card_text, "type") != Some("Cassette") {
        return Err("card não é uma cassette OKF".into());
    }
    let member = campo(&card_text, "archive_member").ok_or("cassette sem membro do ZIP")?;
    if member.contains('/') || member.contains('\\') || !member.ends_with(".json") {
        return Err("membro inseguro da cassette".into());
    }
    let source = campo(&card_text, "source_zip").ok_or("URL do ZIP ausente")?;
    let suffix = source
        .strip_prefix("https://archive.org/download/todos-sei-cassettes-2026/")
        .ok_or("ZIP fora do acervo público")?;
    if suffix.contains("..")
        || !suffix.ends_with(".zip")
        || !(suffix.starts_with("lotes/lote-") || suffix.starts_with("todos-sei-cassettes-"))
    {
        return Err("caminho do ZIP não é permitido".into());
    }
    let expected_zip = campo(&card_text, "archive_sha1").ok_or("hash do ZIP ausente")?;
    let expected_json =
        campo(&card_text, "cassette_sha256_lf").ok_or("hash da cassette ausente")?;
    if expected_zip.len() != 40
        || !expected_zip.bytes().all(|b| b.is_ascii_hexdigit())
        || expected_json.len() != 64
        || !expected_json.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("digest do card inválido".into());
    }
    let installation = campo(&card_text, "installation")
        .ok_or("instalação ausente")?
        .to_owned();
    if !["unb", "ufrj", "go", "pen1", "pen2", "pen3", "pen4"].contains(&installation.as_str()) {
        return Err("instalação fora da lista pública".into());
    }
    if zip.sha1 != expected_zip {
        return Err("SHA-1 do ZIP não confere".into());
    }
    let mut archive =
        ZipArchive::new(Cursor::new(zip.raw.as_slice())).map_err(|_| "ZIP inválido")?;
    let file = archive
        .by_name(member)
        .map_err(|_| "membro não encontrado no ZIP")?;
    if file.size() > limite {
        return Err(format!("cassette JSON maior que {} MB", limite / 1_000_000));
    }
    let mut json_bytes = Vec::new();
    file.take(limite + 1)
        .read_to_end(&mut json_bytes)
        .map_err(|_| "falha de leitura da cassette")?;
    if json_bytes.len() as u64 > limite {
        return Err(format!("cassette maior que {} MB", limite / 1_000_000));
    }
    if hex::encode(Sha256::digest(com_lf(&json_bytes))) != expected_json {
        return Err("SHA-256 da cassette não confere".into());
    }
    let json: serde_json::Value =
        serde_json::from_slice(&json_bytes).map_err(|_| "JSON de cassette inválido")?;
    if json["complete"] != true || json["schema_version"] != 1 {
        return Err("cassette incompleta ou schema desconhecido".into());
    }
    let root = json["sei_root"].as_str().unwrap_or("");
    let allowed = match installation.as_str() {
        "unb" => "sei.treinamento.unb.br",
        "ufrj" => "teste.sei.tic.ufrj.br",
        "go" => "seitreinamento.go.gov.br",
        "pen1" => "sei.orgao1.tramita.processoeletronico.gov.br",
        "pen2" => "sei.orgao2.tramita.processoeletronico.gov.br",
        "pen3" => "sei.orgao3.tramita.processoeletronico.gov.br",
        _ => "sei.orgao4.tramita.processoeletronico.gov.br",
    };
    if !root.starts_with(&format!("https://{allowed}/")) && root != format!("https://{allowed}") {
        return Err("host da cassette não corresponde à instalação".into());
    }
    Ok(CassetteConferida {
        json,
        instalacao: installation,
    })
}

/// Bytes e codificação de uma resposta HTML gravada (sem conferir status nem
/// operação, que são do chamador).
pub(super) fn corpo_html(it: &serde_json::Value) -> Result<(Vec<u8>, String), String> {
    let mime = it["response"]["headers"]["content-type"]
        .as_str()
        .unwrap_or("");
    if !mime.to_ascii_lowercase().contains("text/html") {
        return Err("resposta não é HTML".into());
    }
    let b64 = it["response"]["body_base64"]
        .as_str()
        .ok_or("body_base64 ausente")?;
    let bytes = STANDARD.decode(b64).map_err(|_| "body_base64 inválido")?;
    if bytes.len() > crate::MAX_HTML_BYTES {
        return Err("HTML acima do limite".into());
    }
    let encoding = it["response"]["encoding"].as_str().unwrap_or("");
    let encoding = if encoding.is_empty() && mime.to_ascii_lowercase().contains("8859-1") {
        "iso-8859-1"
    } else {
        encoding
    };
    Ok((bytes, encoding.to_owned()))
}

/// O HTML decodificado de uma resposta gravada.
pub(crate) fn html_da_resposta(it: &serde_json::Value) -> Result<String, String> {
    let (bytes, encoding) = corpo_html(it)?;
    legivel(&bytes, &encoding)
}

fn com_lf(bytes: &[u8]) -> Vec<u8> {
    let mut ret = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
            ret.push(b'\n');
            i += 2;
        } else {
            ret.push(bytes[i]);
            i += 1;
        }
    }
    ret
}
