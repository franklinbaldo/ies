//! Cada recusa de resposta do GET tem variante própria: status 5xx, 429,
//! outro status, tipo não HTML e corpo não decodificável no charset.
use super::*;
use std::io::Cursor;

fn resposta(status_code: i32, mime: &str, corpo: &[u8]) -> RespostaWeb<Cursor<Vec<u8>>> {
    RespostaWeb {
        status_code,
        mime: mime.to_owned(),
        content_length: Some(corpo.len()),
        corpo: Cursor::new(corpo.to_vec()),
    }
}

#[test]
fn status_http_inesperado_e_classificado_pela_faixa_observavel() {
    for (status, esperado) in [
        (500, FalhaTransporte::StatusHttpDeErroDoServidor),
        (503, FalhaTransporte::StatusHttpDeErroDoServidor),
        (429, FalhaTransporte::StatusHttpDeLimiteDeRequisicoes),
        (404, FalhaTransporte::StatusHttpDiferenteDe200),
        (204, FalhaTransporte::StatusHttpDiferenteDe200),
    ] {
        assert_eq!(
            decodificar_resposta(resposta(status, "text/html", b"<html>")),
            Err(esperado),
            "status {status}"
        );
    }
}

#[test]
fn tipo_de_conteudo_nao_html_tem_variante_propria() {
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
fn corpo_invalido_no_charset_declarado_tem_variante_propria() {
    // 0xE7 isolado não é UTF-8 válido; sem charset, o corpo é lido como UTF-8.
    assert_eq!(
        decodificar_resposta(resposta(200, "text/html", b"<html>a\xe7\xe3o</html>")),
        Err(FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado)
    );
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
fn subcodigos_das_recusas_de_resposta_sao_distintos() {
    let variantes = [
        FalhaTransporte::StatusHttpDeErroDoServidor,
        FalhaTransporte::StatusHttpDeLimiteDeRequisicoes,
        FalhaTransporte::StatusHttpDiferenteDe200,
        FalhaTransporte::TipoDeConteudoNaoHtml,
        FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado,
    ];
    let mut subcodigos: Vec<_> = variantes.iter().map(|v| v.subcodigo()).collect();
    subcodigos.sort_unstable();
    subcodigos.dedup();
    assert_eq!(subcodigos.len(), variantes.len());
}
