//! Transporte de visualização por URL assinada já publicada ao operador.
//! Nenhum link é construído, renovado ou enviado para origem diferente.
use crate::instalacao::{Instalacao, host_valido};
use crate::transporte::{self, FalhaTransporte};
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErroDocumentoTransporte {
    Fonte(FalhaTransporte),
    UrlNaoEhVisualizacaoAssinada,
    OrigemDoDocumentoNaoAutorizada,
}
impl ErroDocumentoTransporte {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Fonte(e) => e.subcodigo(),
            Self::UrlNaoEhVisualizacaoAssinada => "url_documento_sem_visualizacao_assinada",
            Self::OrigemDoDocumentoNaoAutorizada => "origem_documento_nao_autorizada",
        }
    }
}

/// Falha fechada: só a ação de visualização comprovada recebe cookie.
pub(crate) fn validar_link(url: &str, origem: &str) -> Result<(), ErroDocumentoTransporte> {
    let ruim = ErroDocumentoTransporte::UrlNaoEhVisualizacaoAssinada;
    if url.contains('#') || url.contains('\\') || !url.bytes().all(|b| (33..=126).contains(&b)) {
        return Err(ruim);
    }
    let (host, resto) = url
        .strip_prefix("https://")
        .and_then(|v| v.split_once('/'))
        .ok_or(ruim)?;
    if !host_valido(host) {
        return Err(ruim);
    }
    let (caminho, query) = resto.split_once('?').ok_or(ruim)?;
    if caminho.contains("..") || caminho.rsplit('/').next() != Some("controlador.php") {
        return Err(ruim);
    }
    let instalacao = Instalacao::interpretar(origem)
        .ok_or(ErroDocumentoTransporte::OrigemDoDocumentoNaoAutorizada)?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(ErroDocumentoTransporte::OrigemDoDocumentoNaoAutorizada);
    }
    let (mut acao, mut id_documento, mut hash) = (None, None, None);
    for parte in query.split('&') {
        let (chave, valor) = parte.split_once('=').ok_or(ruim)?;
        if !chave.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
            return Err(ruim);
        }
        let slot = match chave {
            "acao" => &mut acao,
            "id_documento" => &mut id_documento,
            "infra_hash" => &mut hash,
            _ => return Err(ruim),
        };
        if slot.replace(valor).is_some() {
            return Err(ruim);
        }
    }
    if acao != Some("documento_visualizar")
        || !id_documento.is_some_and(|id: &str| {
            !id.is_empty()
                && id.bytes().all(|b| b.is_ascii_digit())
                && id.bytes().any(|b| b != b'0')
        })
        || !hash.is_some_and(|h: &str| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(ruim);
    }
    Ok(())
}

struct Resposta<R: Read> {
    status_code: i32,
    mime: String,
    content_length: Option<usize>,
    corpo: R,
}

fn decodificar<R: Read>(mut resposta: Resposta<R>) -> Result<String, ErroDocumentoTransporte> {
    let fonte = |e| ErroDocumentoTransporte::Fonte(e);
    if (300..400).contains(&resposta.status_code) {
        return Err(fonte(FalhaTransporte::Redirecionamento));
    }
    if matches!(resposta.status_code, 401 | 403) {
        return Err(fonte(FalhaTransporte::AcessoNegado));
    }
    if resposta.status_code != 200 || !crate::mime_web::e_html(&resposta.mime) {
        return Err(fonte(FalhaTransporte::RespostaInesperada));
    }
    if resposta
        .content_length
        .is_some_and(|n| n > crate::MAX_HTML_BYTES)
    {
        return Err(fonte(FalhaTransporte::RespostaExcessiva));
    }
    let bytes =
        transporte::ler_limitado(&mut resposta.corpo, crate::MAX_HTML_BYTES).map_err(fonte)?;
    let html = crate::cassette::decode_http_body(&bytes, &resposta.mime)
        .map_err(|_| fonte(FalhaTransporte::RespostaInesperada))?;
    if crate::tela_login::e_tela_login(&html) {
        return Err(fonte(FalhaTransporte::PaginaDeLoginRecebidaNoControle));
    }
    Ok(html)
}

/// A closure `FnOnce` impede retry oculto; a origem antecede leitura do cookie.
fn ler_via<R: Read, F>(
    url: &str,
    cookie_file: &Path,
    origem: &str,
    requisitar: F,
) -> Result<String, ErroDocumentoTransporte>
where
    F: FnOnce(&str, &str) -> Result<Resposta<R>, ErroDocumentoTransporte>,
{
    validar_link(url, origem)?;
    let cookie =
        transporte::arquivo_privado(cookie_file, 8_192).map_err(ErroDocumentoTransporte::Fonte)?;
    let cookie = cookie.trim();
    transporte::validar_cookie(cookie).map_err(ErroDocumentoTransporte::Fonte)?;
    decodificar(requisitar(url, cookie)?)
}

pub(crate) fn ler_de_arquivos(
    url_file: &Path,
    cookie_file: &Path,
    origem_file: &Path,
) -> Result<String, ErroDocumentoTransporte> {
    let url =
        transporte::arquivo_privado(url_file, 8_192).map_err(ErroDocumentoTransporte::Fonte)?;
    let origem =
        transporte::arquivo_privado(origem_file, 512).map_err(ErroDocumentoTransporte::Fonte)?;
    ler_via(url.trim(), cookie_file, origem.trim(), |destino, cookie| {
        let resposta = minreq::get(destino)
            .with_timeout(30)
            .with_follow_redirects(false)
            .with_header("Cookie", cookie)
            .send_lazy()
            .map_err(|_| ErroDocumentoTransporte::Fonte(FalhaTransporte::Rede))?;
        let mime = resposta
            .headers
            .get("content-type")
            .cloned()
            .unwrap_or_default();
        let content_length = resposta
            .headers
            .get("content-length")
            .and_then(|n| n.trim().parse::<usize>().ok());
        Ok(Resposta {
            status_code: resposta.status_code,
            mime,
            content_length,
            corpo: resposta,
        })
    })
}

#[cfg(test)]
#[path = "documento_transporte/tests.rs"]
mod tests;
