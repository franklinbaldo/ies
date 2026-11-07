//! Enquadramento do corpo em fluxo: truncamento é falha de rede, pedaço que
//! passaria de 8 MB é `RespostaExcessiva` antes de ser lido, 3xx sem corpo.
use super::*;
use crate::transporte::ler_limitado;
use std::io::Cursor;

fn corpo(bruto: &str) -> Result<Vec<u8>, FalhaTransporte> {
    let resposta = Resposta::ler(Cursor::new(bruto.as_bytes().to_vec()))?;
    ler_limitado(resposta.corpo, crate::MAX_HTML_BYTES)
}

#[test]
fn enquadramentos_do_corpo() {
    assert_eq!(
        corpo("HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabcdef").unwrap(),
        b"abc"
    );
    assert_eq!(
        corpo("HTTP/1.1 200 OK\r\n\r\nate o fim").unwrap(),
        b"ate o fim"
    );
    let ambos = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Length: 1\r\n\r\n\
                 2\r\nok\r\n0\r\n\r\n";
    assert_eq!(corpo(ambos).unwrap(), b"ok");
    assert_eq!(
        corpo("HTTP/1.1 301 X\r\nContent-Length: zz\r\n\r\nnao lido").unwrap(),
        b""
    );
}

#[test]
fn truncamento_e_pedaco_malformado_sao_falha_de_rede() {
    for ruim in [
        "HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\ncurto",
        "HTTP/1.1 200 OK\r\nContent-Length: x\r\n\r\n",
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nzz\r\nab\r\n0\r\n\r\n",
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nabX\r\n0\r\n\r\n",
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nab",
    ] {
        assert_eq!(corpo(ruim), Err(FalhaTransporte::Rede), "{ruim:?}");
    }
}

/// RED: após zero chunk, EOF não substitui a linha vazia que encerra trailers.
/// Um proxy/servidor que trunca o trailer não comprovou fim íntegro do HTTP.
#[test]
fn trailer_chunked_truncado_nao_pode_parecer_resposta_completa() {
    let prefixo = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n1\r\na\r\n0\r\n";
    for sufixo in ["", "X-Teste: valor\r\n", "X-Teste: valor\r"] {
        let bruto = format!("{prefixo}{sufixo}");
        assert_eq!(corpo(&bruto), Err(FalhaTransporte::Rede), "{sufixo:?}");
    }
    assert_eq!(
        corpo(&format!("{prefixo}X-Teste: valor\r\n\r\n")),
        Ok(b"a".to_vec())
    );
}

#[test]
fn pedaco_acima_de_8_mb_e_recusado_antes_da_leitura() {
    let bruto = format!(
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\nab",
        crate::MAX_HTML_BYTES + 1
    );
    assert_eq!(corpo(&bruto), Err(FalhaTransporte::RespostaExcessiva));
    let resposta = Resposta::ler(Cursor::new(
        b"HTTP/1.1 200 OK\r\nContent-Length: 9000000\r\n\r\n".to_vec(),
    ))
    .unwrap();
    assert_eq!(resposta.corpo.tamanho_declarado(), Some(9_000_000));
}
