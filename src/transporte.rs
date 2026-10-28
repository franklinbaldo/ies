//! Transporte mínimo de leitura, restrito a GET da caixa, sem login implícito.
use std::fs;
use std::path::Path;

fn validar_url(url: &str) -> Result<(), String> {
    let origin = url
        .strip_prefix("https://")
        .ok_or("a caixa exige URL HTTPS")?;
    let (authority, path) = origin.split_once('/').ok_or("URL da caixa inválida")?;
    if authority.is_empty()
        || authority.contains('@')
        || authority.contains(':')
        || authority.contains('#')
        || authority.chars().any(char::is_whitespace)
    {
        return Err("host da caixa inválido".into());
    }
    let (route, query) = path.split_once('?').ok_or("query assinada ausente")?;
    if route.rsplit('/').next() != Some("controlador.php")
        || route.contains("..")
        || url.contains('#')
    {
        return Err("não é uma URL de controlador do SEI".into());
    }
    let mut actions = 0;
    let mut hashes = 0;
    for item in query.split('&') {
        let (name, value) = item.split_once('=').ok_or("parâmetro inválido")?;
        if !name.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
            return Err("nome de parâmetro não permitido".into());
        }
        if name == "acao" {
            if value != "procedimento_controlar" {
                return Err("ação diferente da caixa".into());
            }
            actions += 1;
        }
        if name == "infra_hash" {
            if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("assinatura inválida".into());
            }
            hashes += 1;
        }
        if name == "acao_ajax" {
            return Err("ação AJAX não permitida".into());
        }
    }
    if actions != 1 || hashes != 1 {
        return Err("a URL não é a caixa assinada da sessão atual".into());
    }
    Ok(())
}

pub fn read_box(url: &str, cookie_file: &Path) -> Result<String, String> {
    validar_url(url)?;
    let meta = fs::metadata(cookie_file).map_err(|_| "arquivo de cookie não encontrado")?;
    if !meta.is_file() || meta.len() > 8_192 {
        return Err("arquivo de cookie inválido".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err("arquivo de cookie deve ter permissão 0600".into());
        }
    }
    let cookie = fs::read_to_string(cookie_file).map_err(|_| "cookie não é UTF-8")?;
    let cookie = cookie.trim();
    if cookie.is_empty()
        || cookie.contains('\r')
        || cookie.contains('\n')
        || cookie.contains(':')
        || cookie.bytes().any(|b| b < 32 || b == 127)
    {
        return Err("formato de cookie inválido; use apenas os pares nome=valor".into());
    }
    let response = minreq::get(url)
        .with_timeout(30)
        .with_follow_redirects(false)
        .with_header("Cookie", cookie)
        .send()
        .map_err(|_| "falha de transporte HTTPS; nenhum pedido foi repetido".to_owned())?;
    if response.status_code != 200 {
        return Err(
            "SEI não retornou HTTP 200; confira a sessão, sem repetir automaticamente".into(),
        );
    }
    let mime = response
        .headers
        .get("content-type")
        .map(String::as_str)
        .unwrap_or("");
    if !mime.to_ascii_lowercase().contains("text/html") {
        return Err("resposta inesperada: não é HTML do SEI".into());
    }
    crate::cassette::decode_http_body(response.as_bytes(), mime)
}

pub fn read_box_from_file(url_file: &Path, cookie_file: &Path) -> Result<String, String> {
    let meta = fs::metadata(url_file).map_err(|_| "arquivo de URL não encontrado")?;
    if !meta.is_file() || meta.len() > 8192 {
        return Err("arquivo de URL inválido".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err("arquivo de URL deve ter permissão 0600".into());
        }
    }
    let text = fs::read_to_string(url_file).map_err(|_| "URL inválida")?;
    read_box(text.trim(), cookie_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nao_aceita_url_sem_assinatura() {
        assert!(
            validar_url("http://sei.exemplo/controlador.php?acao=procedimento_controlar").is_err()
        );
        assert!(
            validar_url("https://sei.exemplo/controlador.php?acao=procedimento_controlar").is_err()
        );
        assert!(
            validar_url("https://usuario@sei.exemplo/controlador.php?acao=procedimento_controlar")
                .is_err()
        );
    }
    #[test]
    fn permite_somente_url_assinada_da_caixa() {
        let url = format!(
            "https://sei.exemplo/controlador.php?acao=procedimento_controlar&infra_hash={}",
            "a".repeat(64)
        );
        assert!(validar_url(&url).is_ok());
        assert!(
            validar_url(&url.replace("procedimento_controlar", "procedimento_excluir")).is_err()
        );
        assert!(validar_url(&format!("{url}&acao=procedimento_excluir")).is_err());
        assert!(validar_url(&format!("{url}&infra_hash={}", "b".repeat(64))).is_err());
        assert!(validar_url(&format!("{url}&a%63ao=procedimento_excluir")).is_err());
    }
}
