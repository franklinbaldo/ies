//! GET do Controle que preserva URL e `Content-Type`, e o POST real da
//! submissão derivada. Mesmas travas do GET: HTTPS na instalação autorizada,
//! sem redirects, 8 MB durante o recebimento e uma única tentativa.
use super::{
    FalhaTransporte, RespostaWeb, arquivo_privado, decodificar_resposta, get_https, resposta_web,
    validar_cookie, validar_origem,
};
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::instalacao::Instalacao;
use std::io::Read;
use std::path::Path;

/// Tela do GET com o que o POST seguinte precisa: URL exata, tipo e instalação.
pub(crate) struct TelaDoControle {
    pub(crate) url: String,
    pub(crate) html: String,
    pub(crate) mime: String,
    pub(crate) instalacao: Instalacao,
}

/// URL assinada e HTML nunca aparecem no `Debug`.
impl std::fmt::Debug for TelaDoControle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TelaDoControle")
            .field("url", &"<redigido>")
            .field("html", &"<redigido>")
            .field("mime", &self.mime)
            .finish_non_exhaustive()
    }
}

/// O GET é injetável; URL, origem e cookie passam pelas validações do GET
/// da caixa antes de qualquer conexão. `FnOnce`: uma única tentativa.
pub(super) fn ler_tela_via<R: Read, F>(
    url: &str,
    cookie_file: &Path,
    origem_autorizada: &str,
    requisitar: F,
) -> Result<TelaDoControle, FalhaTransporte>
where
    F: FnOnce(&str, &str) -> Result<RespostaWeb<R>, FalhaTransporte>,
{
    validar_origem(url, origem_autorizada)?;
    let instalacao =
        Instalacao::interpretar(origem_autorizada).ok_or(FalhaTransporte::OrigemNaoAutorizada)?;
    let texto = arquivo_privado(cookie_file, 8_192)?;
    let cookie = texto.trim();
    validar_cookie(cookie)?;
    let resposta = requisitar(url, cookie)?;
    let mime = resposta.mime.clone();
    let html = decodificar_resposta(resposta)?;
    Ok(TelaDoControle {
        url: url.to_owned(),
        html,
        mime,
        instalacao,
    })
}

/// GET único do Controle a partir dos arquivos privados.
pub(crate) fn ler_tela_do_controle(
    url_file: &Path,
    cookie_file: &Path,
    origem_file: &Path,
) -> Result<TelaDoControle, FalhaTransporte> {
    let url = arquivo_privado(url_file, 8_192)?;
    let origem = arquivo_privado(origem_file, 512)?;
    ler_tela_via(url.trim(), cookie_file, origem.trim(), get_https)
}

/// POST HTTPS único, sem seguir redirects. A URL, a instalação e o corpo
/// vêm da derivação; `Origin` e `Referer` são os da tela servida. O
/// `Content-Type` é o do navegador, sem parâmetro: o corpo já está no
/// charset da tela. Status, MIME, limite e login ficam com o envio.
// A submissão é recebida por valor, como no `FnOnce` do envio: consumida
// numa única tentativa, sem cópia para retry.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn enviar_post(
    submissao: SubmissaoDerivada,
    cookie: &str,
) -> Result<RespostaDoPost<minreq::ResponseLazy>, FalhaTransporte> {
    if !submissao.url().starts_with("https://") {
        return Err(FalhaTransporte::UrlInvalida);
    }
    let resposta = minreq::post(submissao.url())
        .with_timeout(30)
        .with_follow_redirects(false)
        .with_header("Cookie", cookie)
        .with_header("Content-Type", SubmissaoDerivada::TIPO_DE_CONTEUDO)
        .with_header("Origin", submissao.origem())
        .with_header("Referer", submissao.referer())
        .with_body(submissao.corpo().to_vec())
        .send_lazy()
        .map_err(|_| FalhaTransporte::Rede)?;
    let localizacao = resposta.headers.get("location").cloned();
    Ok(RespostaDoPost {
        resposta: resposta_web(resposta),
        localizacao,
    })
}

/// POST com ambiente e camada TLS injetáveis (testes com proxy falso).
pub(crate) fn enviar_post_via<S: Read + std::io::Write>(
    submissao: SubmissaoDerivada,
    cookie: &str,
    _ambiente: impl Fn(&str) -> Option<String>,
    _envolver: impl FnOnce(&str, std::net::TcpStream) -> Result<S, FalhaTransporte>,
) -> Result<RespostaDoPost<impl Read>, FalhaTransporte> {
    enviar_post(submissao, cookie)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const URL: &str = "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar&infra_hash=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn cookie() -> std::path::PathBuf {
        let caminho = std::env::temp_dir().join(format!(
            "ies-envio-post-cookie-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&caminho, "PHPSESSID=SINTETICO").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        caminho
    }

    #[test]
    fn tela_preserva_url_e_tipo_e_recusa_origem_antes_do_cookie() {
        let arquivo = cookie();
        let tela = ler_tela_via(URL, &arquivo, "https://sei.exemplo.gov.br/sei", |_, c| {
            assert_eq!(c, "PHPSESSID=SINTETICO");
            Ok(RespostaWeb {
                status_code: 200,
                mime: "text/html; charset=iso-8859-1".to_owned(),
                content_length: None,
                corpo: Cursor::new(b"<html>ok</html>".to_vec()),
            })
        })
        .unwrap();
        assert_eq!(tela.url, URL);
        assert_eq!(tela.mime, "text/html; charset=iso-8859-1");
        assert!(!format!("{tela:?}").contains("aaaa"));
        let recusada = ler_tela_via(
            URL,
            &arquivo,
            "https://outro.gov.br/sei",
            |_, _| -> Result<RespostaWeb<Cursor<Vec<u8>>>, FalhaTransporte> {
                panic!("não pode enviar fora da origem autorizada")
            },
        )
        .map(|_| ());
        assert_eq!(recusada, Err(FalhaTransporte::OrigemNaoAutorizada));
        std::fs::remove_file(arquivo).unwrap();
    }
}
