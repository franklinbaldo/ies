//! Bancada sintética compartilhada para respostas HTTP em testes Rust.
//! Não faz rede, não guarda credenciais e não inspeciona URLs assinadas.
//! A política de cada operação (302, login, efeitos) fica nos testes dela.
use super::RespostaWeb;
use crate::formulario_web::RespostaDoPost;
use std::collections::VecDeque;
use std::io::Cursor;

/// Monta a resposta com bytes EXATOS: os testes escolhem charset e
/// Content-Length. Não tentar normalizar ISO-8859-1 ou corrigir HTML.
pub(crate) fn resposta(
    status_code: i32,
    mime: &str,
    corpo: Vec<u8>,
    content_length: Option<usize>,
) -> RespostaWeb<Cursor<Vec<u8>>> {
    RespostaWeb {
        status_code,
        mime: mime.to_owned(),
        content_length,
        localizacao: None,
        corpo: Cursor::new(corpo),
    }
}

/// Mesmo corpo de resposta, com Location como dado (nunca seguido).
pub(crate) fn resposta_post(
    status_code: i32,
    mime: &str,
    corpo: Vec<u8>,
    content_length: Option<usize>,
    localizacao: Option<String>,
) -> RespostaDoPost<Cursor<Vec<u8>>> {
    RespostaDoPost {
        resposta: resposta(status_code, mime, corpo, content_length),
        localizacao,
    }
}

/// Registra a tentativa antes de consumir a resposta, inclusive em falhas.
pub(crate) fn atender<P, R>(pedidos: &mut Vec<P>, respostas: &mut VecDeque<R>, pedido: P) -> R {
    pedidos.push(pedido);
    respostas.pop_front().expect("requisição além do roteiro")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn conserva_bytes_charset_e_tamanho_declarado() {
        let bytes = vec![b'a', 0xe7, b'o'];
        let mut r = resposta(200, "text/html; charset=iso-8859-1", bytes.clone(), None);
        assert_eq!(r.content_length, None);
        assert_eq!(r.mime, "text/html; charset=iso-8859-1");
        let mut lido = Vec::new();
        r.corpo.read_to_end(&mut lido).unwrap();
        assert_eq!(lido, bytes);
    }

    #[test]
    fn location_do_302_e_dado_sem_abrir_rede() {
        let r = resposta_post(
            302,
            "text/html",
            Vec::new(),
            Some(0),
            Some("controlador.php?acao=teste".into()),
        );
        assert_eq!(r.resposta.status_code, 302);
        assert_eq!(r.localizacao.as_deref(), Some("controlador.php?acao=teste"));
    }
}
