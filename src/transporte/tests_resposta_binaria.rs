//! RED/GREEN do GET binário (Refs #60, fatia 3): `chunked` acima de 8 MB
//! passa quando o limite do anexo é maior e é recusado acima dele; nome de
//! arquivo em ISO-8859-1 no cabeçalho não derruba a resposta; só os
//! cabeçalhos conferidos ficam (nunca `Set-Cookie`). Respostas sintéticas.
use super::*;
use crate::transporte::resposta_http::Resposta;
use std::io::{Cursor, Read};

fn ler(bruto: Vec<u8>, limite: usize) -> RespostaBinaria<Corpo<Cursor<Vec<u8>>>> {
    binaria(Resposta::ler(Cursor::new(bruto)).unwrap(), limite)
}

/// Corpo `chunked` com um pedaço de `n` bytes.
fn em_pedacos(n: usize) -> Vec<u8> {
    let mut bruto = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/pdf\r\nTransfer-Encoding: chunked\r\n\r\n{n:x}\r\n"
    )
    .into_bytes();
    bruto.extend(std::iter::repeat_n(b'x', n));
    bruto.extend_from_slice(b"\r\n0\r\n\r\n");
    bruto
}

#[test]
fn chunked_acima_de_8_mb_vale_ate_o_limite_do_anexo() {
    let n = crate::MAX_HTML_BYTES + 10;
    let mut lido = Vec::new();
    ler(em_pedacos(n), n)
        .corpo
        .read_to_end(&mut lido)
        .expect("dentro do limite do anexo");
    assert_eq!(lido.len(), n);
    let erro = ler(em_pedacos(n), n - 1)
        .corpo
        .read_to_end(&mut Vec::new())
        .unwrap_err();
    assert_eq!(
        crate::transporte::resposta_http::falha_de_leitura(&erro),
        FalhaTransporte::RespostaExcessiva
    );
}

#[test]
fn nome_em_iso_8859_1_no_cabecalho_e_lido_e_set_cookie_e_descartado() {
    let mut bruto = b"HTTP/1.1 200 OK\r\nContent-Type: application/pdf\r\n\
        Set-Cookie: PHPSESSID=SEGREDO\r\nContent-Disposition: inline; filename=\"Of"
        .to_vec();
    bruto.push(0xED); // í em ISO-8859-1
    bruto.extend_from_slice(b"cio.pdf\"\r\nContent-Length: 4\r\n\r\n%PDF");
    let r = ler(bruto, 100);
    assert_eq!(r.status, 200);
    assert_eq!(
        r.valores("Content-Disposition"),
        ["inline; filename=\"Ofício.pdf\""]
    );
    assert_eq!(r.valores("content-length"), ["4"]);
    assert!(r.valores("set-cookie").is_empty());
    assert!(
        !r.cabecalhos
            .iter()
            .any(|(n, v)| n.contains("cookie") || v.contains("SEGREDO"))
    );
}

#[test]
fn cabecalho_repetido_fica_com_todas_as_ocorrencias() {
    let bruto = b"HTTP/1.1 200 OK\r\nContent-Type: application/pdf\r\n\
        content-type: text/html\r\nContent-Length: 0\r\n\r\n"
        .to_vec();
    assert_eq!(
        ler(bruto, 100).valores("content-type"),
        ["application/pdf", "text/html"]
    );
}
