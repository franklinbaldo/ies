//! RED/GREEN do GET binário (Refs #60, fatia 3): `chunked` acima de 8 MB
//! passa quando o limite do anexo é maior e é recusado acima dele; nome de
//! arquivo em ISO-8859-1 no cabeçalho não derruba a resposta; só os
//! cabeçalhos conferidos ficam (nunca `Set-Cookie`). Respostas sintéticas.
use super::*;
use crate::transporte::resposta_http::sintetica;
use std::io::{Cursor, Read};

fn ler(
    cabecalhos: &[(&str, &str)],
    corpo: &[u8],
    limite: usize,
) -> RespostaBinaria<Corpo<Cursor<Vec<u8>>>> {
    binaria(sintetica(200, cabecalhos, corpo), limite)
}

/// Corpo sem `Content-Length` com `n` bytes.
fn em_pedacos(n: usize) -> Vec<u8> {
    vec![b'x'; n]
}

#[test]
fn chunked_acima_de_8_mb_vale_ate_o_limite_do_anexo() {
    let n = crate::MAX_HTML_BYTES + 10;
    let mut lido = Vec::new();
    ler(&[("Content-Type", "application/pdf")], &em_pedacos(n), n)
        .corpo
        .read_to_end(&mut lido)
        .expect("dentro do limite do anexo");
    assert_eq!(lido.len(), n);
    let erro = ler(
        &[("Content-Type", "application/pdf")],
        &em_pedacos(n),
        n - 1,
    )
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
    let mut nome = b"inline; filename=\"Of".to_vec();
    nome.push(0xED); // í em ISO-8859-1
    nome.extend_from_slice(b"cio.pdf\"");
    let disposicao = crate::transporte::resposta_http::texto_do_cabecalho(&nome);
    let r = ler(
        &[
            ("Content-Type", "application/pdf"),
            ("Set-Cookie", "PHPSESSID=SEGREDO"),
            ("Content-Disposition", &disposicao),
            ("Content-Length", "4"),
        ],
        b"%PDF",
        100,
    );
    assert_eq!(r.status, 200);
    assert_eq!(
        r.valores("Content-Disposition"),
        ["inline; filename=\"Ofício.pdf\""]
    );
    assert_eq!(r.valores("content-length"), ["4"]);
    assert_eq!(r.valores("set-cookie"), Vec::<&str>::new());
    assert!(
        !r.cabecalhos
            .iter()
            .any(|(n, v)| n.contains("cookie") || v.contains("SEGREDO"))
    );
}

#[test]
fn cabecalho_repetido_fica_com_todas_as_ocorrencias() {
    assert_eq!(
        ler(
            &[
                ("Content-Type", "application/pdf"),
                ("content-type", "text/html"),
                ("Content-Length", "0")
            ],
            b"",
            100
        )
        .valores("content-type"),
        ["application/pdf", "text/html"]
    );
}
