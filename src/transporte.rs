//! GET HTTPS da caixa; cookies só seguem para a origem explicitamente autorizada.
//! Sem login, redirects ou tentativa de outro backend; o único POST é o de
//! `envio_post`, usado por `--pagina N`.
use crate::instalacao::{Instalacao, host_valido};
use std::fs;
use std::io::Read;
use std::path::Path;

/// Status de uma recusa HTTP e o texto que o SEI enviou junto, já saneado
/// (`corpo_de_recusa`). É apresentação: o subcódigo da variante classifica.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusHttpRecusado {
    pub status: u16,
    /// `None` em 3xx (corpo não lido) ou quando o corpo não rendeu texto.
    pub mensagem_do_servidor: Option<String>,
}

impl StatusHttpRecusado {
    pub(crate) fn sem_mensagem(status: u16) -> Self {
        Self {
            status,
            mensagem_do_servidor: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Status HTTP 401 ou 403.
    AcessoNegado(StatusHttpRecusado),
    /// Status HTTP 3xx; o corpo não é lido.
    Redirecionamento(StatusHttpRecusado),
    /// Status HTTP 5xx, fora 3xx/401/403 já tipados.
    StatusHttpDeErroDoServidor(StatusHttpRecusado),
    /// Status HTTP 429.
    StatusHttpDeLimiteDeRequisicoes(StatusHttpRecusado),
    /// Qualquer outro status diferente de 200.
    StatusHttpDiferenteDe200(StatusHttpRecusado),
    /// `Content-Type` cujo tipo principal não é `text/html`.
    TipoDeConteudoNaoHtml,
    /// Corpo que não decodifica no charset declarado (ou UTF-8).
    CorpoNaoDecodificavelNoCharsetDeclarado,
    PaginaDeLoginRecebidaNoControle,
    RespostaExcessiva,
    Rede,
}

impl FalhaTransporte {
    pub fn subcodigo(&self) -> &'static str {
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
            Self::AcessoNegado(_) => "sessao_nao_autorizada",
            Self::Redirecionamento(_) => "redirecionamento_recusado",
            Self::StatusHttpDeErroDoServidor(_) => "status_http_de_erro_do_servidor",
            Self::StatusHttpDeLimiteDeRequisicoes(_) => "status_http_de_limite_de_requisicoes",
            Self::StatusHttpDiferenteDe200(_) => "status_http_diferente_de_200",
            Self::TipoDeConteudoNaoHtml => "tipo_de_conteudo_nao_html",
            Self::CorpoNaoDecodificavelNoCharsetDeclarado => {
                "corpo_nao_decodificavel_no_charset_declarado"
            }
            Self::PaginaDeLoginRecebidaNoControle => "pagina_de_login_recebida_no_controle",
            Self::RespostaExcessiva => "resposta_excessiva",
            Self::Rede => "falha_de_rede",
        }
    }
    pub fn mensagem(&self) -> &'static str {
        match self {
            Self::UrlInvalida => "URL assinada da caixa inválida",
            Self::ChaveDeConsultaNaoCanonica => {
                "nome de parâmetro da URL de controle não é canônico"
            }
            Self::SeparadorDeConsultaAmbiguo => {
                "separador de parâmetros da URL de controle não é inequívoco"
            }
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
            Self::AcessoNegado(_) => "SEI negou acesso; a sessão pode estar vencida",
            Self::Redirecionamento(_) => {
                "redirecionamento recusado; a sessão pode exigir novo login"
            }
            Self::StatusHttpDeErroDoServidor(_) => "o SEI respondeu com erro do servidor (5xx)",
            Self::StatusHttpDeLimiteDeRequisicoes(_) => "o SEI limitou as requisições (429)",
            Self::StatusHttpDiferenteDe200(_) => "o SEI respondeu com status HTTP diferente de 200",
            Self::TipoDeConteudoNaoHtml => "a resposta não declara Content-Type text/html",
            Self::CorpoNaoDecodificavelNoCharsetDeclarado => {
                "o corpo da resposta não decodifica no charset declarado"
            }
            Self::PaginaDeLoginRecebidaNoControle => {
                "o SEI devolveu formulário de login em vez da caixa"
            }
            Self::RespostaExcessiva => "resposta HTML supera o limite permitido",
            Self::Rede => "falha de transporte HTTPS; nenhum pedido foi repetido",
        }
    }
}

impl FalhaTransporte {
    /// Status e mensagem do SEI, só nas recusas por status HTTP.
    pub fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::AcessoNegado(recusa)
            | Self::Redirecionamento(recusa)
            | Self::StatusHttpDeErroDoServidor(recusa)
            | Self::StatusHttpDeLimiteDeRequisicoes(recusa)
            | Self::StatusHttpDiferenteDe200(recusa) => Some(recusa),
            _ => None,
        }
    }
}

// PHP decodifica percent/plus e converte pontos/espaços em underscores;
// preservar a rejeição de nomes não literais já exigida pelo transporte.
pub(crate) fn conferir_chave_de_consulta(chave: &str) -> Result<(), FalhaTransporte> {
    if chave.is_empty() || !chave.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
        return Err(FalhaTransporte::ChaveDeConsultaNaoCanonica);
    }
    Ok(())
}

// Alguns servidores PHP tratam ';' como separador adicional de parâmetros.
pub(crate) fn conferir_valor_de_consulta(valor: &str) -> Result<(), FalhaTransporte> {
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

pub(crate) fn validar_cookie(cookie: &str) -> Result<(), FalhaTransporte> {
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
pub(crate) struct RespostaWeb<R: Read> {
    pub(crate) status_code: i32,
    pub(crate) mime: String,
    pub(crate) content_length: Option<usize>,
    pub(crate) corpo: R,
}

pub(crate) fn decodificar_resposta<R: Read>(
    mut resposta: RespostaWeb<R>,
) -> Result<String, FalhaTransporte> {
    if resposta.status_code != 200 {
        return Err(recusar_status(resposta));
    }
    if !crate::mime_web::e_html(&resposta.mime) {
        return Err(FalhaTransporte::TipoDeConteudoNaoHtml);
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
        .map_err(|_| FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado)?;
    if crate::tela_login::e_tela_login(&html) {
        return Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle);
    }
    Ok(html)
}

/// Status diferente de 200: a variante sai do status; o corpo (fora 3xx)
/// é lido uma vez, até `corpo_de_recusa::LIMITE`, só para a mensagem.
fn recusar_status<R: Read>(resposta: RespostaWeb<R>) -> FalhaTransporte {
    let codigo = resposta.status_code;
    let status = u16::try_from(codigo).unwrap_or(0);
    if (300..400).contains(&codigo) {
        return FalhaTransporte::Redirecionamento(StatusHttpRecusado::sem_mensagem(status));
    }
    let recusa = StatusHttpRecusado {
        status,
        mensagem_do_servidor: corpo_de_recusa::mensagem(resposta.corpo, &resposta.mime),
    };
    match codigo {
        401 | 403 => FalhaTransporte::AcessoNegado(recusa),
        429 => FalhaTransporte::StatusHttpDeLimiteDeRequisicoes(recusa),
        500..=599 => FalhaTransporte::StatusHttpDeErroDoServidor(recusa),
        _ => FalhaTransporte::StatusHttpDiferenteDe200(recusa),
    }
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
    read_box_via(url, cookie_file, origem_autorizada, get_https)
}

/// Resposta `minreq` como `RespostaWeb`, sem ler o corpo.
fn resposta_web(resposta: minreq::ResponseLazy) -> RespostaWeb<minreq::ResponseLazy> {
    let mime = resposta
        .headers
        .get("content-type")
        .cloned()
        .unwrap_or_default();
    let content_length = resposta
        .headers
        .get("content-length")
        .and_then(|n| n.trim().parse::<usize>().ok());
    RespostaWeb {
        status_code: resposta.status_code,
        mime,
        content_length,
        corpo: resposta,
    }
}

/// GET real único, sem seguir redirects; o corpo é lido depois, limitado.
fn get_https(
    destino: &str,
    cookie: &str,
) -> Result<RespostaWeb<minreq::ResponseLazy>, FalhaTransporte> {
    let resposta = minreq::get(destino)
        .with_timeout(30)
        .with_follow_redirects(false)
        .with_header("Cookie", cookie)
        .send_lazy()
        .map_err(|_| FalhaTransporte::Rede)?;
    Ok(resposta_web(resposta))
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

#[path = "transporte/corpo_de_recusa.rs"]
pub(crate) mod corpo_de_recusa;

#[path = "transporte/texto_de_recusa.rs"]
pub(crate) mod texto_de_recusa;

#[path = "transporte/saneamento.rs"]
pub(crate) mod saneamento;

#[path = "transporte/envio_post.rs"]
pub(crate) mod envio_post;

#[cfg(test)]
#[path = "transporte/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "transporte/web_mock_tests.rs"]
mod web_mock_tests;

#[cfg(test)]
#[path = "transporte/tests_resposta.rs"]
mod tests_resposta;

#[cfg(all(test, target_os = "linux"))]
#[path = "transporte/paired_worker.rs"]
mod paired_worker;
