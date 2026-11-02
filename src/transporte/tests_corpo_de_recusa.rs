//! RED/GREEN da leitura do corpo da recusa: 64 KiB no máximo, em fluxo,
//! charset declarado com queda para ISO-8859-1, nunca mascarando o status.
use super::{LIMITE, decodificar, ler_ate_o_limite, mensagem};
use crate::transporte::tests_recusa_http::CorpoContado;
use std::cell::Cell;
use std::io::{self, Read};

struct CorpoQuebrado;

impl Read for CorpoQuebrado {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("conexão caiu"))
    }
}

#[test]
fn corpo_maior_que_o_limite_consome_so_64_kib() {
    let entregues = Cell::new(0);
    let lidos = ler_ate_o_limite(CorpoContado {
        entregues: &entregues,
        byte: b'x',
    })
    .unwrap();
    assert_eq!(LIMITE, 65_536);
    assert_eq!(lidos.len(), LIMITE);
    assert_eq!(entregues.get(), LIMITE, "nada além do limite é lido");
}

#[test]
fn corpo_curto_e_lido_inteiro_e_falha_de_leitura_nao_rende_mensagem() {
    assert_eq!(
        ler_ate_o_limite(&b"pouco"[..]).as_deref(),
        Some(&b"pouco"[..])
    );
    assert_eq!(ler_ate_o_limite(CorpoQuebrado), None);
    assert_eq!(mensagem(CorpoQuebrado, "text/plain"), None);
}

#[test]
fn charset_declarado_e_queda_para_iso_8859_1() {
    let latin1 = b"a\xe7\xe3o";
    assert_eq!(decodificar(latin1, "text/html; charset=ISO-8859-1"), "ação");
    assert_eq!(
        decodificar(latin1, "text/plain; charset=windows-1252"),
        "ação"
    );
    assert_eq!(
        decodificar("ação".as_bytes(), "text/plain; charset=utf-8"),
        "ação"
    );
    // Bytes inválidos em UTF-8 (ou sem charset) caem para ISO-8859-1.
    assert_eq!(decodificar(latin1, "text/plain; charset=utf-8"), "ação");
    assert_eq!(decodificar(latin1, ""), "ação");
}

#[test]
fn caractere_utf8_cortado_pelo_limite_no_fim_e_descartado() {
    let mut bytes = "ok ç".as_bytes().to_vec();
    bytes.pop();
    assert_eq!(decodificar(&bytes, "text/plain; charset=utf-8"), "ok ");
}

#[test]
fn texto_puro_e_html_viram_mensagem_saneada() {
    assert_eq!(
        mensagem(&b"  Service\r\nUnavailable  "[..], "text/plain").as_deref(),
        Some("Service Unavailable")
    );
    let html = b"<html><body><div id='divInfraExcecao'>Falha \xe0 toa</div></body></html>";
    assert_eq!(
        mensagem(&html[..], "text/html; charset=iso-8859-1").as_deref(),
        Some("Falha à toa")
    );
    // Fora de HTML, as tags não são interpretadas.
    assert_eq!(
        mensagem(&b"{\"erro\":\"<b>x</b>\"}"[..], "application/json").as_deref(),
        Some("{\"erro\":\"<b>x</b>\"}")
    );
    assert_eq!(mensagem(&b""[..], "text/html"), None);
}
