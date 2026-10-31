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
    PaginaDeLoginRecebidaNoControle,
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
            Self::PaginaDeLoginRecebidaNoControle => "pagina_de_login_recebida_no_controle",
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
            Self::ArquivoPrivadoLinkSimbolico => "arquivo privado não pode ser link simbólico",
            Self::ArquivoPrivadoVazio => "arquivo privado vazio",
            Self::ArquivoPrivadoAcimaDoLimite => "arquivo privado supera o limite permitido",
            Self::ArquivoPrivadoNaoPodeSerLido => "não foi possível ler o arquivo privado",
            Self::CookieInvalido => "formato de cookie de sessão inválido",
            Self::AcessoNegado => "SEI negou acesso; a sessão pode estar vencida",
            Self::Redirecionamento => "redirecionamento recusado; a sessão pode exigir novo login",
            Self::RespostaInesperada => "servidor não retornou a caixa HTML esperada",
            Self::PaginaDeLoginRecebidaNoControle => {
                "o SEI devolveu formulário de login em vez da caixa"
            }
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
    // A checagem do caminho fornece erros específicos; a decisão de
    // segurança usa o descritor aberto, não metadados sujeitos a TOCTOU.
    let observado = fs::symlink_metadata(path)
        .map_err(|_| FalhaTransporte::ArquivoPrivadoAusenteOuInacessivel)?;
    if observado.file_type().is_symlink() {
        return Err(FalhaTransporte::ArquivoPrivadoLinkSimbolico);
    }
    if !observado.is_file() {
        return Err(FalhaTransporte::CaminhoDeArquivoNaoEhArquivo);
    }

    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // O_NOFOLLOW impede a troca do último componente por symlink.
        // O_NONBLOCK impede que a troca por FIFO bloqueie a leitura.
        options.custom_flags(0o400_000 | 0o4_000);
    }
    let arquivo = options.open(path).map_err(|erro| {
        if cfg!(target_os = "linux") && erro.raw_os_error() == Some(40) {
            FalhaTransporte::ArquivoPrivadoLinkSimbolico
        } else {
            FalhaTransporte::ArquivoPrivadoNaoPodeSerLido
        }
    })?;
    let metadata = arquivo
        .metadata()
        .map_err(|_| FalhaTransporte::ArquivoPrivadoNaoPodeSerLido)?;
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
    let mut bytes = Vec::new();
    arquivo
        .take(limite + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| FalhaTransporte::ArquivoPrivadoNaoPodeSerLido)?;
    if u64::try_from(bytes.len()).is_ok_and(|tamanho| tamanho > limite) {
        return Err(FalhaTransporte::ArquivoPrivadoAcimaDoLimite);
    }
    if bytes.is_empty() {
        return Err(FalhaTransporte::ArquivoPrivadoVazio);
    }
    String::from_utf8(bytes).map_err(|_| FalhaTransporte::ArquivoPrivadoNaoPodeSerLido)
}

fn validar_cookie(cookie: &str) -> Result<(), FalhaTransporte> {
    if cookie.is_empty() || cookie.contains(':') || !cookie.bytes().all(|b| (32..=126).contains(&b))
    {
        return Err(FalhaTransporte::CookieInvalido);
    }
    Ok(())
}

/// Pré-validação offline: usa exatamente as regras do transporte Web,
/// sem enviar cookies nem testar se a sessão continua autenticada.
pub fn verificar_sessao_local_de_arquivos(
    url_file: &Path,
    cookie_file: &Path,
    origem_file: &Path,
) -> Result<(), FalhaTransporte> {
    let url = arquivo_privado(url_file, 8_192)?;
    let origem = arquivo_privado(origem_file, 512)?;
    // Bloquear destino divergente ANTES de abrir o cookie confidencial.
    validar_origem(url.trim(), origem.trim())?;
    let cookie = arquivo_privado(cookie_file, 8_192)?;
    validar_cookie(cookie.trim())
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

/// Resposta HTTP separada do transporte para ensaiar a mesma verificação
/// de status, MIME e limite com um leitor sintético, sem acesso ao SEI.
struct RespostaWeb<R: Read> {
    status_code: i32,
    mime: String,
    content_length: Option<usize>,
    corpo: R,
}

fn decodificar_resposta<R: Read>(mut resposta: RespostaWeb<R>) -> Result<String, FalhaTransporte> {
    if (300..400).contains(&resposta.status_code) {
        return Err(FalhaTransporte::Redirecionamento);
    }
    if resposta.status_code == 401 || resposta.status_code == 403 {
        return Err(FalhaTransporte::AcessoNegado);
    }
    if resposta.status_code != 200 {
        return Err(FalhaTransporte::RespostaInesperada);
    }
    if !resposta.mime.to_ascii_lowercase().contains("text/html") {
        return Err(FalhaTransporte::RespostaInesperada);
    }
    if resposta
        .content_length
        .is_some_and(|tamanho| tamanho > crate::MAX_HTML_BYTES)
    {
        return Err(FalhaTransporte::RespostaExcessiva);
    }
    // A resposta é limitada DURANTE a leitura, mesmo sem Content-Length.
    let bytes = ler_limitado(&mut resposta.corpo, crate::MAX_HTML_BYTES)?;
    let html = crate::cassette::decode_http_body(&bytes, &resposta.mime)
        .map_err(|_| FalhaTransporte::RespostaInesperada)?;
    if crate::tela_login::e_tela_login(&html) {
        return Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle);
    }
    Ok(html)
}

/// A única chamada GET é injetável em testes; URL, origem e cookie passam
/// pelas mesmas validações que o transporte real, antes de qualquer conexão.
/// A closure `FnOnce` impede retries ocultos ou consultas adicionais.
fn read_box_via<R: Read, F>(
    url: &str,
    cookie_file: &Path,
    origem_autorizada: &str,
    requisitar: F,
) -> Result<String, FalhaTransporte>
where
    F: FnOnce(&str, &str) -> Result<RespostaWeb<R>, FalhaTransporte>,
{
    validar_origem(url, origem_autorizada)?;
    let texto = arquivo_privado(cookie_file, 8_192)?;
    let cookie = texto.trim();
    validar_cookie(cookie)?;
    let resposta = requisitar(url, cookie)?;
    decodificar_resposta(resposta)
}

pub fn read_box(
    url: &str,
    cookie_file: &Path,
    origem_autorizada: &str,
) -> Result<String, FalhaTransporte> {
    read_box_via(url, cookie_file, origem_autorizada, |destino, cookie| {
        let resposta = minreq::get(destino)
            .with_timeout(30)
            .with_follow_redirects(false)
            .with_header("Cookie", cookie)
            .send_lazy()
            .map_err(|_| FalhaTransporte::Rede)?;
        let mime = resposta
            .headers
            .get("content-type")
            .cloned()
            .unwrap_or_default();
        let content_length = resposta
            .headers
            .get("content-length")
            .and_then(|n| n.trim().parse::<usize>().ok());
        Ok(RespostaWeb {
            status_code: resposta.status_code,
            mime,
            content_length,
            corpo: resposta,
        })
    })
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
    fn http_200_com_formulario_de_login_nao_eh_caixa_autenticada() {
        let corpo = b"<form id='frmLogin'><input name='txtUsuario'></form>";
        let resposta = RespostaWeb {
            status_code: 200,
            mime: "text/html; charset=utf-8".to_owned(),
            content_length: None,
            corpo: Cursor::new(corpo),
        };
        assert_eq!(
            decodificar_resposta(resposta),
            Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle)
        );
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
        use std::os::unix::fs::{PermissionsExt, symlink};
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
        use std::os::unix::fs::{PermissionsExt, symlink};
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
            read_box_from_file(
                &atalho,
                &raiz.join("cookie-nao-existe"),
                &raiz.join("origem-nao-existe"),
            ),
            Err(FalhaTransporte::ArquivoPrivadoLinkSimbolico)
        );
        fs::remove_dir_all(raiz).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn descritor_revalida_permissoes_e_limite_de_conteudo() {
        use std::os::unix::fs::PermissionsExt;
        let pasta = std::env::temp_dir().join(format!(
            "ies-descritor-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&pasta).unwrap();
        let arquivo = pasta.join("privado");
        fs::write(&arquivo, "CONTEUDO").unwrap();
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            arquivo_privado(&arquivo, 128),
            Err(FalhaTransporte::ArquivoPrivadoComPermissoesInseguras)
        );
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            arquivo_privado(&arquivo, 4),
            Err(FalhaTransporte::ArquivoPrivadoAcimaDoLimite)
        );
        assert_eq!(arquivo_privado(&arquivo, 128), Ok("CONTEUDO".to_owned()));
        fs::remove_dir_all(&pasta).unwrap();
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

#[cfg(test)]
#[path = "transporte/web_mock_tests.rs"]
mod web_mock_tests;
