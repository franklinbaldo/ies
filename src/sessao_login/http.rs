//! Adaptador do login ao cliente HTTPS único (`transporte::cliente_https`):
//! o mesmo HTTP/1.1 sobre `native-tls`, proxy do ambiente, sem
//! redirecionamento e sem retry. O login lê o corpo inteiro (8 MB durante o
//! recebimento) e guarda **todos** os `Set-Cookie`; o intérprete de
//! redirecionamentos (`redirecionamento`) é só do login.
use super::{RespostaHttp, TransporteDeLogin};
use crate::formulario_web::SubmissaoDerivada;
use crate::transporte::cliente_https::{self, Envio};
use crate::transporte::resposta_http::Resposta;
use crate::transporte::{FalhaTransporte, ler_limitado};
use std::io::BufRead;

/// Transporte real do login.
pub(crate) struct HttpsDoLogin;

impl TransporteDeLogin for HttpsDoLogin {
    fn get(&mut self, url: &str, cookie: &str) -> Result<RespostaHttp, FalhaTransporte> {
        completar(cliente_https::requisitar("GET", url, cookie, None)?)
    }

    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaHttp, FalhaTransporte> {
        let envio = Envio {
            tipo: SubmissaoDerivada::TIPO_DE_CONTEUDO,
            origem: submissao.origem(),
            referer: submissao.referer(),
            corpo: submissao.corpo(),
        };
        let resposta = cliente_https::requisitar("POST", submissao.url(), cookie, Some(&envio))?;
        completar(resposta)
    }
}

/// Corpo inteiro (não lido em 3xx), com `Content-Length` acima de 8 MB
/// recusado antes da leitura.
fn completar<R: BufRead>(resposta: Resposta<R>) -> Result<RespostaHttp, FalhaTransporte> {
    if resposta
        .corpo
        .tamanho_declarado()
        .is_some_and(|tamanho| tamanho > crate::MAX_HTML_BYTES)
    {
        return Err(FalhaTransporte::RespostaExcessiva);
    }
    let Resposta {
        status,
        cabecalhos,
        corpo,
    } = resposta;
    Ok(RespostaHttp {
        status,
        cabecalhos,
        corpo: ler_limitado(corpo, crate::MAX_HTML_BYTES)?,
    })
}

/// Completa uma resposta sintética como o login real.
#[cfg(test)]
pub(crate) fn completar_sintetica(
    status: u16,
    cabecalhos: &[(&str, &str)],
    corpo: &[u8],
) -> Result<RespostaHttp, FalhaTransporte> {
    completar(crate::transporte::resposta_http::sintetica(
        status, cabecalhos, corpo,
    ))
}
