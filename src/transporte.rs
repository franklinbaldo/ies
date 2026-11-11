//! GET HTTPS da caixa; cookies só seguem para a origem explicitamente autorizada.
//! Sem login, redirects ou tentativa de outro backend; os POSTs são os de
//! `envio_post`. Todo pedido passa pelo `cliente_https` único (com proxy).
use crate::acao_do_sei::AcaoDoSei;
use crate::instalacao::Instalacao;
use std::fs;
use std::io::Read;
use std::path::Path;

pub(crate) use falha::FatoDaResposta;
pub use falha::{FalhaTransporte, StatusHttpRecusado};

#[path = "transporte/falha.rs"]
mod falha;

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
    let partes =
        crate::url_observada::UrlHttps::interpretar(url).ok_or(FalhaTransporte::UrlInvalida)?;
    let authority = partes.host;
    let route = partes.caminho;
    let query = partes.consulta.ok_or(FalhaTransporte::UrlInvalida)?;
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
            "acao" if AcaoDoSei::ler(valor) == Some(AcaoDoSei::ProcedimentoControlar) => {
                actions += 1;
            }
            // 192: o Controle a que a troca de unidade redireciona (#205).
            "infra_hash"
                if matches!(valor.len(), 64 | 192)
                    && valor.bytes().all(|b| b.is_ascii_hexdigit()) =>
            {
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

pub(crate) fn validar_origem(url: &str, origem: &str) -> Result<(), FalhaTransporte> {
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
    // A leitura coerente bloqueia destino divergente ANTES de abrir o
    // cookie confidencial.
    let referencias = sessao::ReferenciasDaSessao::ler(url_file, origem_file, cookie_file)?;
    referencias.cookie().valor().map(|_| ())
}

pub(crate) fn ler_limitado<R: Read>(leitor: R, limite: usize) -> Result<Vec<u8>, FalhaTransporte> {
    let mut bytes = Vec::new();
    leitor
        .take(u64::try_from(limite).map_or(u64::MAX, |limite| limite.saturating_add(1)))
        .read_to_end(&mut bytes)
        .map_err(|erro| resposta_http::falha_de_leitura(&erro))?;
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
    /// `Location` da resposta, como dado: nunca seguido. Só o GET do
    /// Controle o lê, para reconhecer o 302 para o login da instalação.
    pub(crate) localizacao: Option<String>,
    pub(crate) corpo: R,
}

pub(crate) fn decodificar_resposta<R: Read>(
    resposta: RespostaWeb<R>,
) -> Result<String, FalhaTransporte> {
    if resposta.status_code == 200 && !crate::mime_web::e_html(&resposta.mime) {
        return Err(FalhaTransporte::TipoDeConteudoNaoHtml);
    }
    let html = decodificar_corpo(resposta)?;
    if crate::tela_login::e_tela_login(&html) {
        return Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle);
    }
    Ok(html)
}

/// Resposta XML das consultas AJAX (`controlador_ajax.php`): o mesmo
/// status, limite e charset da tela HTML, com o MIME XML.
pub(crate) fn decodificar_resposta_xml<R: Read>(
    resposta: RespostaWeb<R>,
) -> Result<String, FalhaTransporte> {
    let principal = resposta.mime.split(';').next().unwrap_or_default().trim();
    if resposta.status_code == 200
        && !principal.eq_ignore_ascii_case("application/xml")
        && !principal.eq_ignore_ascii_case("text/xml")
    {
        return Err(FalhaTransporte::TipoDeConteudoNaoXml);
    }
    decodificar_corpo(resposta)
}

fn decodificar_corpo<R: Read>(mut resposta: RespostaWeb<R>) -> Result<String, FalhaTransporte> {
    if resposta.status_code != 200 {
        return Err(recusar_status(resposta));
    }
    if resposta
        .content_length
        .is_some_and(|tamanho| tamanho > crate::MAX_HTML_BYTES)
    {
        return Err(FalhaTransporte::RespostaExcessiva);
    }
    // A resposta é limitada DURANTE a leitura, mesmo sem Content-Length.
    let bytes = ler_limitado(&mut resposta.corpo, crate::MAX_HTML_BYTES)?;
    corpo::decode_http_body(&bytes, &resposta.mime)
        .map_err(|_| FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado)
}

/// Status diferente de 200: o número sobe como veio; o corpo (fora 3xx)
/// é lido uma vez, até `corpo_de_recusa::LIMITE`, só para a mensagem.
pub(crate) fn recusar_status<R: Read>(resposta: RespostaWeb<R>) -> FalhaTransporte {
    let codigo = resposta.status_code;
    let status = u16::try_from(codigo).unwrap_or(0);
    if (300..400).contains(&codigo) {
        // 3xx: o corpo não é lido.
        return FalhaTransporte::Status(StatusHttpRecusado::sem_mensagem(status));
    }
    let lido = corpo_de_recusa::ler(resposta.corpo, &resposta.mime);
    let recusa = StatusHttpRecusado {
        status,
        mensagem_do_servidor: lido.mensagem,
    };
    if codigo == 403 && lido.bloqueio_de_seguranca {
        FalhaTransporte::BloqueioDeSegurancaDaInstalacao(recusa)
    } else {
        FalhaTransporte::Status(recusa)
    }
}

/// A única chamada GET é injetável em testes; URL, origem e cookie passam
/// pelas mesmas validações que o transporte real, antes de qualquer conexão.
/// A closure `FnOnce` impede retries ocultos ou consultas adicionais.
/// Só nos testes: é a bancada das validações de origem, cookie e resposta
/// (o resumo pela caixa, que a usava, foi aposentado em 04/11/2026).
#[cfg(test)]
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
    let cookie = sessao::CookieDaSessao::ler(cookie_file);
    let resposta = requisitar(url, cookie.valor()?)?;
    decodificar_resposta(resposta)
}

#[cfg(all(test, unix))] // só o teste de link simbólico
pub fn read_box(
    url: &str,
    cookie_file: &Path,
    origem_autorizada: &str,
) -> Result<String, FalhaTransporte> {
    read_box_via(url, cookie_file, origem_autorizada, get_https)
}

/// GET real único pelo cliente HTTPS (com o proxy do ambiente), sem
/// seguir redirects; o corpo é lido depois, limitado.
pub(crate) fn get_https(
    destino: &str,
    cookie: &str,
) -> Result<RespostaWeb<cliente_https::CorpoHttps>, FalhaTransporte> {
    cliente_https::requisitar("GET", destino, cookie, None).map(cliente_https::resposta_web)
}

#[cfg(all(test, unix))] // só o teste de link simbólico
pub fn read_box_from_file(
    url_file: &Path,
    cookie_file: &Path,
    origem_file: &Path,
) -> Result<String, FalhaTransporte> {
    let referencias = sessao::ReferenciasDaSessao::ler(url_file, origem_file, cookie_file)?;
    read_box(
        referencias.url_do_controle(),
        cookie_file,
        referencias.origem(),
    )
}

#[path = "transporte/corpo.rs"]
pub(crate) mod corpo;

#[path = "transporte/corpo_de_recusa.rs"]
pub(crate) mod corpo_de_recusa;

#[path = "transporte/texto_de_recusa.rs"]
pub(crate) mod texto_de_recusa;

#[path = "transporte/bloqueio_de_seguranca.rs"]
pub(crate) mod bloqueio_de_seguranca;

#[cfg(test)]
#[path = "transporte/tests_bloqueio_de_seguranca.rs"]
mod tests_bloqueio_de_seguranca;

#[path = "transporte/saneamento.rs"]
pub(crate) mod saneamento;

#[path = "transporte/validacao_da_infra.rs"]
pub(crate) mod validacao_da_infra;

#[path = "transporte/resposta_http.rs"]
pub(crate) mod resposta_http;

#[path = "transporte/cliente_https.rs"]
pub(crate) mod cliente_https;

#[path = "transporte/envio_post.rs"]
pub(crate) mod envio_post;

#[path = "transporte/resposta_binaria.rs"]
pub(crate) mod resposta_binaria;

#[path = "transporte/sessao.rs"]
pub(crate) mod sessao;

#[path = "transporte/contagem.rs"]
pub(crate) mod contagem;

#[cfg(test)]
#[path = "transporte/testes_http.rs"]
pub(crate) mod testes_http;

#[cfg(test)]
#[path = "transporte/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "transporte/web_mock_tests.rs"]
mod web_mock_tests;

#[cfg(test)]
#[path = "transporte/tests_resposta.rs"]
mod tests_resposta;

#[cfg(test)]
#[path = "transporte/tests_recusa_http.rs"]
mod tests_recusa_http;

#[cfg(all(test, target_os = "linux"))]
#[path = "transporte/paired_worker.rs"]
mod paired_worker;
