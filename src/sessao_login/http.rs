//! Cliente HTTPS mínimo do login: HTTP/1.1 sobre `native-tls`, uma
//! conexão por pedido, `Connection: close`, sem redirecionamento, sem
//! retry, 8 MB durante o recebimento. Existe porque o `minreq` 2.14.1 guarda
//! um valor por nome de cabeçalho (`HashMap::insert` em `read_metadata`) e
//! perderia todos os `Set-Cookie` de uma resposta menos o último.
use super::{RespostaHttp, TransporteDeLogin};
use crate::formulario_web::SubmissaoDerivada;
use crate::transporte::FalhaTransporte;
use std::io::BufRead;

/// Transporte real do login.
pub(crate) struct HttpsDoLogin;

impl TransporteDeLogin for HttpsDoLogin {
    fn get(&mut self, url: &str, cookie: &str) -> Result<RespostaHttp, FalhaTransporte> {
        let _ = (url, cookie);
        todo!("CONTRACT: GET HTTPS do login")
    }

    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaHttp, FalhaTransporte> {
        let _ = (submissao, cookie);
        todo!("CONTRACT: POST HTTPS do login")
    }
}

/// Lê status, todos os cabeçalhos e o corpo (não lido em 3xx).
pub(crate) fn ler_resposta<R: BufRead>(leitor: &mut R) -> Result<RespostaHttp, FalhaTransporte> {
    let _ = leitor;
    todo!("CONTRACT: ler resposta HTTP/1.1")
}
