//! Replay offline seguro de resposta HTTP a partir de cassette pública.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::io::{Cursor, Read};
use std::path::Path;
use zip::ZipArchive;

fn campo<'a>(source: &'a str, field: &str) -> Option<&'a str> {
    let rest = source.strip_prefix("---\n")?;
    rest.split("\n---\n").next()?.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key == field).then_some(value.trim())
    })
}

fn legivel(bytes: &[u8], encoding: &str) -> Result<String, String> {
    let enc = encoding.to_ascii_lowercase();
    if enc.contains("utf-8") || enc == "utf8" {
        return String::from_utf8(bytes.to_vec())
            .map_err(|_| "resposta não é UTF-8 válido".to_owned());
    }
    if enc.contains("8859-1") || enc.contains("latin1") || enc.is_empty() {
        return Ok(bytes.iter().copied().map(char::from).collect());
    }
    Err("codificação de resposta ainda não suportada".into())
}

// O acervo OKF exige sufixo `.zip` exatamente em minúsculas.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub fn archive_filename(card: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(card).map_err(|_| "card ausente")?;
    let url = campo(&text, "source_zip").ok_or("origem ausente")?;
    let base = url
        .strip_prefix("https://archive.org/download/todos-sei-cassettes-2026/")
        .ok_or("origem fora do acervo")?;
    if base.contains("..") || !base.ends_with(".zip") {
        return Err("origem insegura".into());
    }
    let name = base.rsplit('/').next().ok_or("nome ausente")?;
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
    {
        return Err("nome de ZIP inválido".into());
    }
    Ok(name.to_owned())
}

/// O replay escolhe pelo conteúdo verificado da cassette, nunca pelo nome do cenário.
pub enum ReplayLido {
    Caixa(Vec<String>, String),
    Historico(String, String),
}

pub fn read_cassette_auto(
    card: &Path,
    archive: &Path,
    passos: &[usize],
) -> Result<ReplayLido, String> {
    let (pages, origem, operacao) = read_verified_html(card, archive, passos, None)?;
    if operacao == "procedimento_consultar_historico" {
        let mut pages = pages;
        Ok(ReplayLido::Historico(pages.remove(0), origem))
    } else {
        Ok(ReplayLido::Caixa(pages, origem))
    }
}

pub fn read_cassette_document(
    card: &Path,
    archive: &Path,
    passos: &[usize],
) -> Result<(String, String), String> {
    let (mut pages, instalacao, _) =
        read_verified_html(card, archive, passos, Some("documento_imprimir_web"))?;
    if pages.len() != 1 {
        return Err("um único HTML do documento deve ser selecionado".into());
    }
    Ok((pages.remove(0), instalacao))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroSelecaoOperacao {
    InteracoesAusentes,
    PassoNaoEncontrado { indice: usize },
    OperacaoDoPassoNaoSuportada { indice: usize },
    PassosComOperacoesDiferentes,
    HistoricoComVariasRespostas,
}
impl ErroSelecaoOperacao {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::InteracoesAusentes => "interacoes_da_cassette_ausentes",
            Self::PassoNaoEncontrado { .. } => "passo_da_cassette_ausente",
            Self::OperacaoDoPassoNaoSuportada { .. } => "operacao_do_passo_nao_suportada",
            Self::PassosComOperacoesDiferentes => "passos_de_operacoes_diferentes",
            Self::HistoricoComVariasRespostas => "historico_exige_um_passo",
        }
    }
}

fn operation_for_replay(
    json: &serde_json::Value,
    passos: &[usize],
) -> Result<&'static str, ErroSelecaoOperacao> {
    let interactions = json["interactions"]
        .as_array()
        .ok_or(ErroSelecaoOperacao::InteracoesAusentes)?;

    // Seleção explícita tem precedência sobre outras operações da cassette.
    // Não deduzir o tipo pelo nome do cenário nem misturar respostas.
    if !passos.is_empty() {
        let mut escolhido = None;
        for &indice in passos {
            let passo = interactions
                .get(indice)
                .ok_or(ErroSelecaoOperacao::PassoNaoEncontrado { indice })?;
            let operacao = match passo["request"]["operation"].as_str() {
                Some("procedimento_controlar") => "procedimento_controlar",
                Some("procedimento_consultar_historico") => "procedimento_consultar_historico",
                _ => return Err(ErroSelecaoOperacao::OperacaoDoPassoNaoSuportada { indice }),
            };
            if escolhido.is_some_and(|anterior| anterior != operacao) {
                return Err(ErroSelecaoOperacao::PassosComOperacoesDiferentes);
            }
            escolhido = Some(operacao);
        }
        if escolhido == Some("procedimento_consultar_historico") && passos.len() != 1 {
            return Err(ErroSelecaoOperacao::HistoricoComVariasRespostas);
        }
        return Ok(escolhido.expect("passos não vazios"));
    }

    // No modo automático, histórico é preferido somente quando observado
    // em GET; a cardinalidade é conferida pelo leitor de páginas.
    let has_history = interactions.iter().any(|it| {
        it["request"]["operation"] == "procedimento_consultar_historico"
            && it["request"]["method"] == "GET"
    });
    Ok(if has_history {
        "procedimento_consultar_historico"
    } else {
        "procedimento_controlar"
    })
}

// O formato canônico exige sufixos em minúsculas nas evidências.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn read_verified_html(
    card: &Path,
    archive: &Path,
    passos: &[usize],
    operation: Option<&str>,
) -> Result<(Vec<String>, String, String), String> {
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

    let metadata = archive.metadata().map_err(|_| "ZIP não encontrado")?;
    if metadata.len() > 160_000_000 {
        return Err("ZIP maior que 160 MB".into());
    }
    let raw = std::fs::read(archive).map_err(|_| "falha ao ler ZIP")?;
    if format!("{:x}", Sha1::digest(&raw)) != expected_zip {
        return Err("SHA-1 do ZIP não confere".into());
    }
    let mut zip = ZipArchive::new(Cursor::new(raw)).map_err(|_| "ZIP inválido")?;
    let file = zip
        .by_name(member)
        .map_err(|_| "membro não encontrado no ZIP")?;
    if file.size() > 30_000_000 {
        return Err("cassette JSON maior que 30 MB".into());
    }
    let mut json_bytes = Vec::new();
    file.take(30_000_001)
        .read_to_end(&mut json_bytes)
        .map_err(|_| "falha de leitura da cassette")?;
    if json_bytes.len() > 30_000_000 {
        return Err("cassette maior que 30 MB".into());
    }
    let normalized = json_bytes.replace_with_lf();
    if format!("{:x}", Sha256::digest(&normalized)) != expected_json {
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
    let operation = match operation {
        Some(operation) => operation,
        None => operation_for_replay(&json, passos)
            .map_err(|erro| erro.subcodigo().to_owned())?,
    };
    let pages = selected_html_pages(&json, passos, operation)?;
    Ok((pages, installation, operation.to_owned()))
}

fn selected_html_pages(
    json: &serde_json::Value,
    passos: &[usize],
    operation: &str,
) -> Result<Vec<String>, String> {
    let list = json["interactions"]
        .as_array()
        .ok_or("interações ausentes")?;
    let chosen: Vec<usize> = if passos.is_empty() {
        list.iter()
            .enumerate()
            .filter_map(|(i, it)| {
                (it["request"]["operation"] == operation && it["request"]["method"] == "GET")
                    .then_some(i)
            })
            .collect()
    } else {
        passos.to_vec()
    };
    if chosen.is_empty() || chosen.len() > 200 {
        return Err("quantidade de páginas inválida".into());
    }
    if passos.is_empty() && chosen.len() != 1 {
        return Err("cassette com várias caixas: selecione os passos explicitamente".into());
    }
    let mut seen = std::collections::HashSet::new();
    if !chosen.iter().all(|i| seen.insert(*i)) {
        return Err("passo repetido na seleção".into());
    }
    let mut pages = Vec::with_capacity(chosen.len());
    let mut total_bytes = 0usize;
    for (order, step) in chosen.into_iter().enumerate() {
        let it = list.get(step).ok_or("passo inexistente")?;
        let method = it["request"]["method"].as_str();
        if it["request"]["operation"] != operation
            || !(method == Some("GET")
                || (operation == "procedimento_controlar" && order > 0 && method == Some("POST")))
            || it["response"]["status_code"] != 200
        {
            return Err("passo não é uma resposta de caixa admitida (GET inicial; GET/POST subsequentes offline)".into());
        }
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
        total_bytes += bytes.len();
        if total_bytes > 48_000_000 {
            return Err("conjunto de HTML maior que 48 MB".into());
        }
        pages.push(legivel(&bytes, encoding)?);
    }
    Ok(pages)
}

trait NewlineNormalization {
    fn replace_with_lf(&self) -> Vec<u8>;
}
impl NewlineNormalization for Vec<u8> {
    fn replace_with_lf(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(self.len());
        let mut i = 0;
        while i < self.len() {
            if self[i] == b'\r' && self.get(i + 1) == Some(&b'\n') {
                ret.push(b'\n');
                i += 2;
            } else {
                ret.push(self[i]);
                i += 1;
            }
        }
        ret
    }
}

pub fn decode_http_body(bytes: &[u8], content_type: &str) -> Result<String, String> {
    if bytes.len() > crate::MAX_HTML_BYTES {
        return Err("HTML maior que 8 MB".into());
    }
    let enc = if content_type.to_ascii_lowercase().contains("8859-1") {
        "iso-8859-1"
    } else {
        "utf-8"
    };
    legivel(bytes, enc)
}

#[cfg(test)]
mod replay_operation_tests {
    use super::*;

    #[test]
    fn historia_e_escolhida_pela_operacao_e_nao_pelo_nome() {
        let json = serde_json::json!({"interactions": [
            {"request": {"operation": "procedimento_controlar", "method": "GET"}},
            {"request": {"operation": "procedimento_consultar_historico", "method": "GET"}}
        ]});
        assert_eq!(
            operation_for_replay(&json, &[]),
            Ok("procedimento_consultar_historico")
        );
        assert_eq!(
            operation_for_replay(&json, &[0]),
            Ok("procedimento_controlar")
        );
    }

    #[test]
    fn caixa_e_padrao_quando_nao_ha_historico() {
        let json = serde_json::json!({"interactions": [
            {"request": {"operation": "procedimento_controlar", "method": "GET"}}
        ]});
        assert_eq!(
            operation_for_replay(&json, &[]),
            Ok("procedimento_controlar")
        );
    }
    #[test]
    fn passo_explicito_de_historico_escolhe_a_operacao_observada() {
        let json = serde_json::json!({"interactions": [
            {"request": {"operation": "procedimento_controlar", "method": "GET"}},
            {"request": {"operation": "procedimento_consultar_historico", "method": "GET"}}
        ]});
        assert_eq!(
            operation_for_replay(&json, &[1]),
            Ok("procedimento_consultar_historico")
        );
        assert_eq!(
            operation_for_replay(&json, &[0]),
            Ok("procedimento_controlar")
        );
        assert_eq!(
            operation_for_replay(&json, &[0, 1]),
            Err(ErroSelecaoOperacao::PassosComOperacoesDiferentes)
        );
    }

    #[test]
    fn passo_inexistente_falha_com_erro_tipado() {
        let json = serde_json::json!({"interactions": [
            {"request": {"operation": "procedimento_controlar", "method": "GET"}}
        ]});
        assert_eq!(
            operation_for_replay(&json, &[7]),
            Err(ErroSelecaoOperacao::PassoNaoEncontrado { indice: 7 })
        );
    }

    #[test]
    fn replay_nao_mistura_multiplos_historicos_nem_aceita_acao_aleatoria() {
        let json = serde_json::json!({"interactions": [
            {"request": {"operation": "procedimento_consultar_historico", "method": "GET"}},
            {"request": {"operation": "procedimento_consultar_historico", "method": "GET"}},
            {"request": {"operation": "documento_assinar", "method": "POST"}}
        ]});
        assert_eq!(
            operation_for_replay(&json, &[0, 1]),
            Err(ErroSelecaoOperacao::HistoricoComVariasRespostas)
        );
        assert_eq!(
            operation_for_replay(&json, &[2]),
            Err(ErroSelecaoOperacao::OperacaoDoPassoNaoSuportada { indice: 2 })
        );
    }

}
