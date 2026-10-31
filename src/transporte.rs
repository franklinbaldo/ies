//! GET HTTPS da caixa; cookies só seguem para a origem explicitamente autorizada.
//! Sem login, redirects, POST ou tentativa de outro backend.
use crate::instalacao::{Instalacao, host_valido};
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FalhaTransporte {
    UrlInvalida,
    OrigemNaoAutorizada,
    ArquivoPrivadoAusenteOuInacessivel,
    ArquivoPrivadoComPermissoesInseguras,
    CaminhoDeArquivoNaoEhArquivo,
    ArquivoPrivadoLinkSimbolico,
    ArquivoPrivadoVazio,
    ArquivoPrivadoAcimaDoLimite,
    ArquivoPrivadoNaoPodeSerLido,
    CookieInvalido,
    AcessoNegado,
    Redirecionamento,
    RespostaInesperada,
    RespostaExcessiva,
    Rede,
}

impl FalhaTransporte {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::UrlInvalida => "url_invalida",
            Self::OrigemNaoAutorizada => "origem_nao_autorizada",
            Self::ArquivoPrivadoAusenteOuInacessivel => "arquivo_privado_ausente_ou_inacessivel",
            Self::ArquivoPrivadoComPermissoesInseguras => {
                "arquivo_privado_com_permissoes_inseguras"
            }
            Self::CaminhoDeArquivoNaoEhArquivo => "caminho_nao_eh_arquivo",
            Self::ArquivoPrivadoLinkSimbolico => "arquivo_privado_link_simbolico",
            Self::ArquivoPrivadoVazio => "arquivo_privado_vazio",
            Self::ArquivoPrivadoAcimaDoLimite => "arquivo_privado_acima_do_limite",
            Self::ArquivoPrivadoNaoPodeSerLido => "arquivo_privado_nao_pode_ser_lido",
            Self::CookieInvalido => "cookie_invalido",
            Self::AcessoNegado => "sessao_nao_autorizada",
            Self::Redirecionamento => "redirecionamento_recusado",
            Self::RespostaInesperada => "resposta_inesperada",
            Self::RespostaExcessiva => "resposta_excessiva",
            Self::Rede => "falha_de_rede",
        }
    }
    pub fn mensagem(self) -> &'static str {
        match self {
            Self::UrlInvalida => "URL assinada da caixa inválida",
            Self::OrigemNaoAutorizada => "destino HTTPS diferente da origem autorizada da sessão",
            Self::ArquivoPrivadoAusenteOuInacessivel => {
                "arquivo privado não encontrado ou sem acesso"
            }
            Self::ArquivoPrivadoComPermissoesInseguras => {
                "arquivo privado permite acesso por outros usuários"
            }
            Self::CaminhoDeArquivoNaoEhArquivo => {
                "caminho informado não aponta para arquivo regular"
            }
            Self::ArquivoPrivadoVazio => "arquivo privado vazio",
            Self::ArquivoPrivadoAcimaDoLimite => "arquivo privado supera o limite permitido",
            Self::ArquivoPrivadoNaoPodeSerLido => "não foi possível ler o arquivo privado",
            Self::CookieInvalido => "formato de cookie de sessão inválido",
            Self::AcessoNegado => "SEI negou acesso; a sessão pode estar vencida",
            Self::Redirecionamento => "redirecionamento recusado; a sessão pode exigir novo login",
            Self::RespostaInesperada => "servidor não retornou a caixa HTML esperada",
            Self::RespostaExcessiva => "resposta HTML supera o limite permitido",
            Self::Rede => "falha de transporte HTTPS; nenhum pedido foi repetido",
        }
    }
}

fn validar_url(url: &str) -> Result<(&str, &str), FalhaTransporte> {
    let (authority, path) = url
        .strip_prefix("https://")
        .and_then(|rest| rest.split_once('/'))
        .ok_or(FalhaTransporte::UrlInvalida)?;
    if !host_valido(authority)
        || !url.bytes().all(|byte| (33..=126).contains(&byte))
        || url.contains('#')
        || url.contains('\\')
    {
        return Err(FalhaTransporte::UrlInvalida);
    }
    let (route, query) = path.split_once('?').ok_or(FalhaTransporte::UrlInvalida)?;
    if route.rsplit('/').next() != Some("controlador.php") || route.contains("..") {
        return Err(FalhaTransporte::UrlInvalida);
    }
    let mut actions = 0;
    let mut hashes = 0;
    for item in query.split('&') {
        let (nome, valor) = item.split_once('=').ok_or(FalhaTransporte::UrlInvalida)?;
        if !nome.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
            return Err(FalhaTransporte::UrlInvalida);
        }
        match nome {
            "acao" if valor == "procedimento_controlar" => actions += 1,
            "infra_hash" if valor.len() == 64 && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hashes += 1;
            }
            "acao" | "infra_hash" | "acao_ajax" => {
                return Err(FalhaTransporte::UrlInvalida);
            }
            _ => {}
        }
    }
    if actions != 1 || hashes != 1 {
        return Err(FalhaTransporte::UrlInvalida);
    }
    Ok((authority, route))
}

fn validar_origem(url: &str, origem: &str) -> Result<(), FalhaTransporte> {
    let (host, rota) = validar_url(url)?;
    let instalacao = Instalacao::interpretar(origem).ok_or(FalhaTransporte::OrigemNaoAutorizada)?;
    if !instalacao.autoriza_controlador(host, rota) {
        return Err(FalhaTransporte::OrigemNaoAutorizada);
    }
    Ok(())
}

fn arquivo_privado(path: &Path, limite: u64) -> Result<String, FalhaTransporte> {
    let metadata =
        fs::metadata(path).map_err(|_| FalhaTransporte::ArquivoPrivadoAusenteOuInacessivel)?;
    if !metadata.is_file() {
        return Err(FalhaTransporte::CaminhoDeArquivoNaoEhArquivo);
    }
    if metadata.len() == 0 {
        return Err(FalhaTransporte::ArquivoPrivadoVazio);
    }
    if metadata.len() > limite {
        return Err(FalhaTransporte::ArquivoPrivadoAcimaDoLimite);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(FalhaTransporte::ArquivoPrivadoComPermissoesInseguras);
        }
    }
    fs::read_to_string(path).map_err(|_| FalhaTransporte::ArquivoPrivadoNaoPodeSerLido)
}

pub(crate) fn ler_limitado<R: Read>(leitor: R, limite: usize) -> Result<Vec<u8>, FalhaTransporte> {
    let mut bytes = Vec::new();
    leitor
        .take(limite as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| FalhaTransporte::Rede)?;
    if bytes.len() > limite {
        return Err(FalhaTransporte::RespostaExcessiva);
    }
    Ok(bytes)
}

pub fn read_box(
    url: &str,
    cookie_file: &Path,
    origem_autorizada: &str,
) -> Result<String, FalhaTransporte> {
    // Antes de ler o cookie ou abrir o socket, conferir o destino.
    validar_origem(url, origem_autorizada)?;
    let texto = arquivo_privado(cookie_file, 8_192)?;
    let cookie = texto.trim();
    if cookie.is_empty() || cookie.contains(':') || !cookie.bytes().all(|b| (32..=126).contains(&b))
    {
        return Err(FalhaTransporte::CookieInvalido);
    }
    let mut resposta = minreq::get(url)
        .with_timeout(30)
        .with_follow_redirects(false)
        .with_header("Cookie", cookie)
        .send_lazy()
        .map_err(|_| FalhaTransporte::Rede)?;
    if (300..400).contains(&resposta.status_code) {
        return Err(FalhaTransporte::Redirecionamento);
    }
    if resposta.status_code == 401 || resposta.status_code == 403 {
        return Err(FalhaTransporte::AcessoNegado);
    }
    if resposta.status_code != 200 {
        return Err(FalhaTransporte::RespostaInesperada);
    }
    let mime = resposta
        .headers
        .get("content-type")
        .cloned()
        .unwrap_or_default();
    if !mime.to_ascii_lowercase().contains("text/html") {
        return Err(FalhaTransporte::RespostaInesperada);
    }
    if resposta
        .headers
        .get("content-length")
        .and_then(|s| s.trim().parse::<usize>().ok())
        .is_some_and(|n| n > crate::MAX_HTML_BYTES)
    {
        return Err(FalhaTransporte::RespostaExcessiva);
    }
    // O limite vale para os bytes REALMENTE recebidos, ainda que a
    // resposta omita ou minta sobre Content-Length (inclusive chunked).
    let bytes = ler_limitado(&mut resposta, crate::MAX_HTML_BYTES)?;
    crate::cassette::decode_http_body(&bytes, &mime)
        .map_err(|_| FalhaTransporte::RespostaInesperada)
}

pub fn read_box_from_file(
    url_file: &Path,
    cookie_file: &Path,
    origem_file: &Path,
) -> Result<String, FalhaTransporte> {
    let url = arquivo_privado(url_file, 8_192)?;
    let origem = arquivo_privado(origem_file, 512)?;
    read_box(url.trim(), cookie_file, origem.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn url(host: &str) -> String {
        format!(
            "https://{host}/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
            "a".repeat(64)
        )
    }

    #[test]
    fn nao_aceita_url_sem_assinatura() {
        assert!(
            validar_url("http://sei.exemplo/controlador.php?acao=procedimento_controlar").is_err()
        );
        assert!(
            validar_url("https://sei.exemplo/controlador.php?acao=procedimento_controlar").is_err()
        );
        assert!(validar_url(&url("usuario@sei.exemplo")).is_err());
        assert!(validar_url(&url("sei.exemplo:443")).is_err());
        assert!(validar_url(&format!("{}&param=\r\nCookie:unsafe", url("sei.exemplo"))).is_err());
        assert!(validar_url(&url("sei.exemplo\\outro")).is_err());
    }

    #[test]
    fn permite_somente_url_assinada_da_caixa() {
        let url = url("sei.exemplo");
        assert!(validar_url(&url).is_ok());
        assert!(
            validar_url(&url.replace("procedimento_controlar", "procedimento_excluir")).is_err()
        );
        assert!(validar_url(&format!("{url}&acao=procedimento_excluir")).is_err());
        assert!(validar_url(&format!("{url}&infra_hash={}", "b".repeat(64))).is_err());
        assert!(validar_url(&format!("{url}&a%63ao=procedimento_excluir")).is_err());
    }

    #[test]
    fn host_de_cookies_precisa_conferir_antes_da_rede() {
        let legitimo = url("sei.treinamento.unb.br");
        let adulterado = url("sei.exemplo");
        assert_eq!(
            validar_origem(&legitimo, "https://sei.treinamento.unb.br/sei"),
            Ok(())
        );
        assert_eq!(
            validar_origem(&legitimo, "https://SEI.TREINAMENTO.UNB.BR/sei/"),
            Ok(())
        );
        assert_eq!(
            validar_origem(&adulterado, "https://sei.treinamento.unb.br/sei"),
            Err(FalhaTransporte::OrigemNaoAutorizada)
        );
        assert_eq!(
            validar_origem(&legitimo, "https://sei.treinamento.unb.br.evil.tld/sei"),
            Err(FalhaTransporte::OrigemNaoAutorizada)
        );
        assert_eq!(
            validar_origem(&legitimo, "http://sei.treinamento.unb.br/sei"),
            Err(FalhaTransporte::OrigemNaoAutorizada)
        );
    }

    #[test]
    fn corpo_e_limitado_durante_a_leitura() {
        assert_eq!(
            ler_limitado(Cursor::new(vec![42; 10]), 10).unwrap().len(),
            10
        );
        assert_eq!(
            ler_limitado(Cursor::new(vec![42; 11]), 10),
            Err(FalhaTransporte::RespostaExcessiva)
        );
        assert_eq!(
            ler_limitado(Cursor::new(vec![42; 10_000]), 10),
            Err(FalhaTransporte::RespostaExcessiva)
        );
    }

    #[cfg(unix)]
    #[test]
    fn links_simbolicos_para_arquivos_privados_nao_podem_ser_lidos() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let raiz = std::env::temp_dir().join(format!(
            "ies-privados-symlink-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&raiz).unwrap();
        let original = raiz.join("cookie-original");
        fs::write(&original, "PHPSESSID=DEMONSTRACAO").unwrap();
        fs::set_permissions(&original, fs::Permissions::from_mode(0o600)).unwrap();
        let atalho = raiz.join("cookie-atalho");
        symlink(&original, &atalho).unwrap();
        assert_eq!(
            arquivo_privado(&atalho, 8_192),
            Err(FalhaTransporte::ArquivoPrivadoLinkSimbolico)
        );
        assert_eq!(
            arquivo_privado(&original, 8_192),
            Ok("PHPSESSID=DEMONSTRACAO".to_owned())
        );
        fs::remove_dir_all(raiz).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn url_assinada_via_link_simbolico_falha_antes_de_consultar_rede() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let raiz = std::env::temp_dir().join(format!(
            "ies-url-symlink-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&raiz).unwrap();
        let original = raiz.join("url-original");
        fs::write(&original, url("sei.exemplo")).unwrap();
        fs::set_permissions(&original, fs::Permissions::from_mode(0o600)).unwrap();
        let atalho = raiz.join("url-atalho");
        symlink(&original, &atalho).unwrap();
        assert_eq!(
            read_box_from_file(&atalho, &raiz.join("cookie-nao-existe"), &raiz.join("origem-nao-existe")),
            Err(FalhaTransporte::ArquivoPrivadoLinkSimbolico)
        );
        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn mesma_origem_em_outra_pasta_nao_recebe_cookie() {
        let legitimo = url("sei.exemplo");
        assert_eq!(
            validar_origem(&legitimo, "https://sei.exemplo"),
            Err(FalhaTransporte::OrigemNaoAutorizada)
        );
        assert_eq!(
            validar_origem(&legitimo, "https://sei.exemplo/outro"),
            Err(FalhaTransporte::OrigemNaoAutorizada)
        );
        assert_eq!(
            validar_origem(&legitimo, "https://sei.exemplo/sei\nversao=5.0.4"),
            Ok(())
        );
        assert_eq!(
            validar_origem(&legitimo, "https://sei.exemplo/sei\nversao=4.0.9"),
            Ok(())
        );
    }
}
