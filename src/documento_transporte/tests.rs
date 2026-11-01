use super::*;
use crate::transporte::FalhaTransporte;
use std::io::Cursor;
fn url(acao: &str, id: &str, hash: &str) -> String {
    format!(
        "https://sei.exemplo/sei/controlador.php?acao={acao}&id_documento={id}&infra_hash={hash}"
    )
}
fn legitima() -> String {
    url("documento_visualizar", "42", &"a".repeat(64))
}
const ORIGEM: &str = "https://sei.exemplo/sei";

#[test]
fn aceita_so_visualizacao_exata_da_origem_declarada() {
    assert_eq!(validar_link(&legitima(), ORIGEM), Ok(()));
    assert_eq!(
        validar_link(&legitima(), "https://sei.exemplo/outro"),
        Err(ErroDocumentoTransporte::OrigemDoDocumentoNaoAutorizada)
    );
}
#[test]
fn recusa_acao_de_efeito_incerto_hash_falso_duplicatas_e_url_externa() {
    let boa = legitima();
    for ruim in [
        url("editor_montar", "42", &"a".repeat(64)),
        url("documento_visualizar", "42", "falso"),
        url("documento_visualizar", "0", &"a".repeat(64)),
        url("documento_visualizar", "4%32", &"a".repeat(64)),
        format!("{boa}&acao=editor_montar"),
        format!("{boa}&id_documento=7"),
        format!("{boa}&token=desconhecido"),
        format!("{boa}#frag"),
        boa.replace("sei.exemplo", "sei.exemplo.evil"),
    ] {
        assert!(validar_link(&ruim, ORIGEM).is_err(), "{ruim}");
    }
}
#[test]
fn http_200_html_de_documento_aceito_e_login_nao() {
    let corpo = "<html><head><title>SEI/UnB - 0007 - Ofício</title></head><body><p>Texto</p></body></html>";
    let resultado = decodificar(Resposta {
        status_code: 200,
        mime: "text/html; charset=utf-8".to_owned(),
        content_length: Some(corpo.len()),
        corpo: Cursor::new(corpo.as_bytes()),
    });
    assert_eq!(resultado, Ok(corpo.to_owned()));
    let login = b"<form id='frmLogin'><input name='txtUsuario'></form>";
    assert_eq!(
        decodificar(Resposta {
            status_code: 200, mime: "text/html".to_owned(),
            content_length: None, corpo: Cursor::new(login),
        }),
        Err(ErroDocumentoTransporte::Fonte(
            FalhaTransporte::PaginaDeLoginRecebidaNoControle
        ))
    );
}
#[test]
fn redirect_e_tipo_mime_divergente_recusados_antes_do_corpo() {
    for (status, mime, esperado) in [
        (302, "text/html", FalhaTransporte::Redirecionamento),
        (403, "text/html", FalhaTransporte::AcessoNegado),
        (200, "application/json", FalhaTransporte::RespostaInesperada),
    ] {
        assert_eq!(
            decodificar(Resposta {
                status_code: status, mime: mime.to_owned(),
                content_length: None, corpo: Cursor::new(b"qualquer"),
            }),
            Err(ErroDocumentoTransporte::Fonte(esperado))
        );
    }
}
#[cfg(unix)]
#[test]
fn origem_errada_impede_leitura_de_cookie_ou_chamada_get() {
    let nenhum = std::path::Path::new("/diretorio-inexistente/cookie");
    let mut pedidos = 0;
    let resultado = ler_via(&legitima(), nenhum, "https://outro.exemplo/sei", |_, _| {
        pedidos += 1;
        Ok(Resposta {
            status_code: 200, mime: "text/html".to_owned(),
            content_length: None, corpo: Cursor::new(Vec::<u8>::new()),
        })
    });
    assert_eq!(resultado, Err(ErroDocumentoTransporte::OrigemDoDocumentoNaoAutorizada));
    assert_eq!(pedidos, 0);
}
