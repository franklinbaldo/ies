//! GET do Controle que preserva URL e `Content-Type`, e o POST real da
//! submissão derivada. Mesmas travas do GET: HTTPS na instalação autorizada,
//! sem redirects, 8 MB durante o recebimento e uma única tentativa.
use super::cliente_https::{self, CorpoHttps, Envio};
use super::resposta_http::{Corpo, Resposta};
use super::sessao::{CookieDaSessao, ReferenciasDaSessao};
use super::{
    FalhaTransporte, RespostaWeb, StatusHttpRecusado, decodificar_resposta, get_https,
    validar_origem,
};
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::instalacao::Instalacao;
use std::io::Read;

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
pub(crate) fn ler_tela_via<R: Read, F>(
    url: &str,
    cookie: &CookieDaSessao,
    origem_autorizada: &str,
    requisitar: F,
) -> Result<TelaDoControle, FalhaTransporte>
where
    F: FnOnce(&str, &str) -> Result<RespostaWeb<R>, FalhaTransporte>,
{
    validar_origem(url, origem_autorizada)?;
    let instalacao =
        Instalacao::interpretar(origem_autorizada).ok_or(FalhaTransporte::OrigemNaoAutorizada)?;
    let resposta = requisitar(url, cookie.valor()?)?;
    // O 302 é dado, nunca seguido: só a entrada do login da instalação
    // vira sessão encerrada; qualquer outro 3xx continua recusado.
    if resposta.status_code == 302
        && resposta.localizacao.as_deref().is_some_and(|local| {
            crate::sessao_login::entrada_do_login::aponta_para_o_login(&instalacao, url, local)
        })
    {
        return Err(FalhaTransporte::ControleRedirecionadoParaOLogin(
            StatusHttpRecusado::sem_mensagem(302),
        ));
    }
    let mime = resposta.mime.clone();
    let html = decodificar_resposta(resposta)?;
    Ok(TelaDoControle {
        url: url.to_owned(),
        html,
        mime,
        instalacao,
    })
}

/// GET único do Controle com o retrato coerente da sessão (#222).
pub(crate) fn ler_tela_do_controle(
    referencias: &ReferenciasDaSessao,
) -> Result<TelaDoControle, FalhaTransporte> {
    ler_tela_via(
        referencias.url_do_controle(),
        referencias.cookie(),
        referencias.origem(),
        get_https,
    )
}

/// POST HTTPS único pelo cliente HTTPS (com o proxy do ambiente), sem
/// seguir redirects. A URL, a instalação e o corpo vêm da derivação;
/// `Origin` e `Referer` são os da tela servida. O `Content-Type` é o do
/// navegador, sem parâmetro: o corpo já está no charset da tela. Status,
/// MIME, limite e login ficam com o envio.
// A submissão é recebida por valor, como no `FnOnce` do envio: consumida
// numa única tentativa, sem cópia para retry.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn enviar_post(
    submissao: SubmissaoDerivada,
    cookie: &str,
) -> Result<RespostaDoPost<CorpoHttps>, FalhaTransporte> {
    let envio = envio_de(&submissao);
    let resposta = cliente_https::requisitar("POST", submissao.url(), cookie, Some(&envio))?;
    Ok(resposta_do_post(resposta))
}

/// GET HTTPS único que devolve também o `Location`: o 302 volta como dado,
/// nunca seguido (`ies processo NUP` precisa dele na entrada por id).
pub(crate) fn get_com_localizacao(
    url: &str,
    cookie: &str,
) -> Result<RespostaDoPost<CorpoHttps>, FalhaTransporte> {
    let resposta = cliente_https::requisitar("GET", url, cookie, None)?;
    Ok(resposta_do_post(resposta))
}

fn envio_de(submissao: &SubmissaoDerivada) -> Envio<'_> {
    Envio {
        tipo: SubmissaoDerivada::TIPO_DE_CONTEUDO,
        origem: submissao.origem(),
        referer: submissao.referer(),
        corpo: submissao.corpo(),
    }
}

fn resposta_do_post<R: std::io::BufRead>(resposta: Resposta<R>) -> RespostaDoPost<Corpo<R>> {
    let localizacao = resposta.cabecalho("location").map(str::to_owned);
    RespostaDoPost {
        resposta: cliente_https::resposta_web(resposta),
        localizacao,
    }
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
        let tela = ler_tela_via(
            URL,
            &CookieDaSessao::ler(&arquivo),
            "https://sei.exemplo.gov.br/sei",
            |_, c| {
                assert_eq!(c, "PHPSESSID=SINTETICO");
                Ok(RespostaWeb {
                    status_code: 200,
                    mime: "text/html; charset=iso-8859-1".to_owned(),
                    content_length: None,
                    localizacao: None,
                    corpo: Cursor::new(b"<html>ok</html>".to_vec()),
                })
            },
        )
        .unwrap();
        assert_eq!(tela.url, URL);
        assert_eq!(tela.mime, "text/html; charset=iso-8859-1");
        assert!(!format!("{tela:?}").contains("aaaa"));
        let recusada = ler_tela_via(
            URL,
            &CookieDaSessao::ler(&arquivo),
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
