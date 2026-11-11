//! RED/GREEN da página de bloqueio de segurança da instalação: 403 com a
//! estrutura observada em GO (título e frase), página SINTÉTICA (contato,
//! código e textos de apoio inventados). 403 genérico não muda.
use super::bloqueio_de_seguranca::e_bloqueio_de_seguranca;
use super::*;
use crate::tela_html::HtmlServido;
use std::io::Cursor;

const BLOQUEIO: &str = "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Acesso Negado</title>\
    <style>body{font-family:sans-serif}</style></head><body><div class=\"caixa\">\
    <h1>Acesso Negado</h1><p>Sua requisição foi bloqueada por nossa política de segurança. \
    Maiores detalhes sobre este bloqueio e sua possível liberação poderão ser solicitados através do \
    e-mail <a href=\"mailto:seguranca@exemplo.gov.br\">seguranca@exemplo.gov.br</a> informando o \
    endereço acessado e código de bloqueio a seguir:</p><p><b>0000000000000-SINTETICO</b></p>\
    </div></body></html>";

fn resposta(status_code: i32, mime: &str, corpo: &str) -> RespostaWeb<Cursor<Vec<u8>>> {
    RespostaWeb {
        status_code,
        mime: mime.to_owned(),
        content_length: Some(corpo.len()),
        localizacao: None,
        corpo: Cursor::new(corpo.as_bytes().to_vec()),
    }
}

#[test]
fn reconhece_a_pagina_pelo_titulo_e_pela_frase_visivel() {
    assert!(e_bloqueio_de_seguranca(&HtmlServido::de_teste(BLOQUEIO)));
    let com_entidades = BLOQUEIO
        .replace("requisição", "requisi&ccedil;&atilde;o")
        .replace("política", "pol&iacute;tica")
        .replace("bloqueada por nossa", "bloqueada\n   por   nossa");
    assert!(e_bloqueio_de_seguranca(&HtmlServido::de_teste(
        &com_entidades
    )));
}

#[test]
fn outro_titulo_frase_ausente_ou_frase_fora_do_texto_visivel_nao_e_bloqueio() {
    assert!(!e_bloqueio_de_seguranca(&HtmlServido::de_teste(
        &BLOQUEIO.replace("<title>Acesso Negado</title>", "<title>SEI - Erro</title>")
    )));
    assert!(!e_bloqueio_de_seguranca(&HtmlServido::de_teste(
        &BLOQUEIO.replace("bloqueada por nossa política de segurança", "negada")
    )));
    let em_script = HtmlServido::de_teste(
        "<html><head><title>Acesso Negado</title></head><body>\
        <script>var t='bloqueada por nossa política de segurança';</script><p>Negado.</p></body></html>",
    );
    assert!(!e_bloqueio_de_seguranca(&em_script));
}

#[test]
fn bloqueio_403_tem_erro_proprio_com_status_e_mensagem() {
    let falha =
        decodificar_resposta(resposta(403, "text/html; charset=utf-8", BLOQUEIO)).unwrap_err();
    let http = falha.status_http().unwrap();
    assert_eq!(http.status, 403);
    let mensagem = http.mensagem_do_servidor.as_deref().unwrap();
    assert!(
        mensagem.contains("bloqueada por nossa política de segurança"),
        "{mensagem}"
    );
    assert!(mensagem.contains("0000000000000-SINTETICO"), "{mensagem}");
    assert!(!mensagem.contains("mailto:"), "{mensagem}");
}

#[test]
fn status_403_generico_e_401_com_a_pagina_continuam_acesso_negado() {
    let _generico = decodificar_resposta(resposta(
        403,
        "text/html",
        "<html><head><title>Forbidden</title></head><body>Forbidden</body></html>",
    ))
    .unwrap_err();
    let _nao_autorizado = decodificar_resposta(resposta(401, "text/html", BLOQUEIO)).unwrap_err();
    let _texto = decodificar_resposta(resposta(403, "text/plain", BLOQUEIO)).unwrap_err();
}

/// Bloqueio do Cloudflare da PEN2 (Tramita, 04/11/2026), no segundo GET de
/// `ies login`: HTTP 403, título `Attention Required! | Cloudflare` e, no
/// texto visível, "Sorry, you have been blocked" e "You are unable to
/// access processoeletronico.gov.br". Página SINTÉTICA com esses textos
/// observados; marcação e textos de apoio inventados.
const CLOUDFLARE: &str = "<!DOCTYPE html><html lang=\"en-US\"><head><title>Attention Required! | Cloudflare</title>\
    <meta charset=\"UTF-8\"></head><body><div id=\"cf-wrapper\">\
    <div class=\"cf-alert\" id=\"cf-error-details\">Please enable cookies.</div>\
    <h1>Sorry, you have been blocked</h1><h2>You are unable to access processoeletronico.gov.br</h2>\
    <h2>Why have I been blocked?</h2><p>This website is using a security service to protect itself \
    from online attacks.</p><p>Ray ID: <strong>0000000000000000</strong></p></div></body></html>";

#[test]
fn bloqueio_do_cloudflare_observado_na_pen2_e_bloqueio_de_seguranca() {
    assert!(e_bloqueio_de_seguranca(&HtmlServido::de_teste(CLOUDFLARE)));
    let falha =
        decodificar_resposta(resposta(403, "text/html; charset=UTF-8", CLOUDFLARE)).unwrap_err();
    assert_eq!(falha.status_http().unwrap().status, 403);
}

#[test]
fn titulo_do_cloudflare_sem_a_frase_do_bloqueio_nao_e_bloqueio() {
    assert!(!e_bloqueio_de_seguranca(&HtmlServido::de_teste(
        &CLOUDFLARE.replace("Sorry, you have been blocked", "Checking your browser")
    )));
    assert!(!e_bloqueio_de_seguranca(&HtmlServido::de_teste(
        &CLOUDFLARE.replace("Attention Required! | Cloudflare", "Just a moment...")
    )));
    // A frase de um par com o título do outro não conta.
    assert!(!e_bloqueio_de_seguranca(&HtmlServido::de_teste(
        &CLOUDFLARE.replace("Attention Required! | Cloudflare", "Acesso Negado")
    )));
}
