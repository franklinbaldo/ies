//! GET HTTPS da caixa; cookies só seguem para a origem explicitamente autorizada.
//! Sem login, redirects, POST ou tentativa de outro backend.
use crate::instalacao::{Instalacao, host_valido};
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FalhaTransporte {
    UrlInvalida,
    ChaveDeConsultaNaoCanonica,
    SeparadorDeConsultaAmbiguo,
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
            Self::ChaveDeConsultaNaoCanonica => "chave_de_consulta_do_controle_nao_canonica",
            Self::SeparadorDeConsultaAmbiguo => "separador_de_consulta_do_controle_ambiguo",
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
            Self::ChaveDeConsultaNaoCanonica => "nome de parâmetro da URL de controle não é canônico",
            Self::SeparadorDeConsultaAmbiguo => "separador de parâmetros da URL de controle não é inequívoco",
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


// PHP decodifica percent/plus e converte pontos/espaços em underscores;
// preservar a rejeição de nomes não literais já exigida pelo transporte.
fn conferir_chave_de_consulta(chave: &str) -> Result<(), FalhaTransporte> {
    if chave.is_empty() || !chave.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
        return Err(FalhaTransporte::ChaveDeConsultaNaoCanonica);
    }
    Ok(())
}

// Alguns servidores PHP tratam ';' como separador adicional de parâmetros.
fn conferir_valor_de_consulta(valor: &str) -> Result<(), FalhaTransporte> {
    if valor.contains(';') {
        return Err(FalhaTransporte::SeparadorDeConsultaAmbiguo);
    }
    Ok(())
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
        conferir_chave_de_consulta(nome)?;
        conferir_valor_de_consulta(valor)?;
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

pub(crate) fn arquivo_privado(path: &Path, limite: u64) -> Result<String, FalhaTransporte> {
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
    if !crate::mime_web::e_html(&resposta.mime) {
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
#[path = "transporte/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "transporte/web_mock_tests.rs"]
mod web_mock_tests;

#[cfg(all(test, target_os = "linux"))]
#[path = "transporte/paired_worker.rs"]
mod paired_worker;
