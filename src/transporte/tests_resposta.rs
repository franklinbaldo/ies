//! Cada recusa de resposta do GET tem erro próprio: status 5xx, 429,
//! outro status, tipo não HTML e corpo não decodificável no charset.
use super::*;
use std::io::Cursor;

fn resposta(status_code: i32, mime: &str, corpo: &[u8]) -> RespostaWeb<Cursor<Vec<u8>>> {
    testes_http::resposta(status_code, mime, corpo.to_vec(), Some(corpo.len()))
}

#[test]
fn status_http_inesperado_e_classificado_pela_faixa_observavel() {
    for (status, _esperado) in [
        (500, "status_http_de_erro_do_servidor"),
        (503, "status_http_de_erro_do_servidor"),
        (429, "status_http_de_limite_de_requisicoes"),
        (404, "status_http_diferente_de_200"),
        (204, "status_http_diferente_de_200"),
        (401, "acesso_negado"),
        (403, "acesso_negado"),
        (301, "redirecionamento"),
    ] {
        let falha = decodificar_resposta(resposta(status, "text/html", b"<html>")).unwrap_err();
        assert_eq!(
            falha.status_http().map(|h| i32::from(h.status)),
            Some(status),
            "status {status}"
        );
    }
}

#[test]
fn tipo_de_conteudo_nao_html_tem_erro_proprio() {
    for mime in [
        "application/json",
        "text/plain",
        "",
        "application/json; profile=text/html",
    ] {
        assert_eq!(
            decodificar_resposta(resposta(200, mime, b"<html>")),
            Err(FalhaTransporte::TipoDeConteudoNaoHtml),
            "mime {mime:?}"
        );
    }
}

#[test]
fn corpo_invalido_no_charset_declarado_tem_erro_proprio() {
    // 0xE7 isolado não é UTF-8 válido; sem charset, o corpo é lido como UTF-8.
    assert!(matches!(
        decodificar_resposta(resposta(200, "text/html", b"<html>a\xe7\xe3o</html>")),
        Err(FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado(
            crate::transporte::corpo::ErroCorpoHttp::Utf8Invalido(_)
        ))
    ));
    // O mesmo corpo é válido quando o charset ISO-8859-1 é declarado.
    assert!(
        decodificar_resposta(resposta(
            200,
            "text/html; charset=iso-8859-1",
            b"<html>a\xe7\xe3o</html>"
        ))
        .is_ok()
    );
}

#[test]
fn consulta_ajax_aceita_so_xml_com_o_mesmo_status_e_charset() {
    // UnB 4.0.9 ao vivo (10/11/2026): protocolo_RI1023 da anexação é XML.
    let xml = b"<?xml version=\"1.0\" encoding=\"iso-8859-1\"?><complementos/>";
    for mime in ["application/xml; charset=iso-8859-1", "text/xml"] {
        assert!(
            decodificar_resposta_xml(resposta(200, mime, xml)).is_ok(),
            "{mime}"
        );
    }
    assert_eq!(
        decodificar_resposta_xml(resposta(200, "text/html", xml)),
        Err(FalhaTransporte::TipoDeConteudoNaoXml)
    );
    assert!(matches!(
        decodificar_resposta_xml(resposta(500, "application/xml", xml)),
        Err(FalhaTransporte::Status(
            crate::transporte::StatusHttpRecusado {
                status: 500..=599,
                ..
            }
        ))
    ));
}
