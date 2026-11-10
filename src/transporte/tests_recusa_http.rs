//! RED/GREEN: toda recusa por status publica o número e o que o SEI disse,
//! com os subcódigos de sempre, um único pedido e 64 KiB lidos no máximo.
use super::corpo_de_recusa::LIMITE;
use super::*;
use std::cell::Cell;
use std::io::{self, Cursor, Read};

/// Corpo sem fim que conta os bytes entregues ao leitor.
pub(super) struct CorpoContado<'a> {
    pub(super) entregues: &'a Cell<usize>,
    pub(super) byte: u8,
}

impl Read for CorpoContado<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        buf.fill(self.byte);
        self.entregues.set(self.entregues.get() + buf.len());
        Ok(buf.len())
    }
}

fn resposta(status_code: i32, mime: &str, corpo: &[u8]) -> RespostaWeb<Cursor<Vec<u8>>> {
    RespostaWeb {
        status_code,
        mime: mime.to_owned(),
        content_length: Some(corpo.len()),
        localizacao: None,
        corpo: Cursor::new(corpo.to_vec()),
    }
}

fn recusa(falha: &FalhaTransporte) -> (u16, Option<&str>) {
    let http = falha.status_http().expect("recusa HTTP");
    (http.status, http.mensagem_do_servidor.as_deref())
}

const EXCECAO_SEI: &[u8] = b"<html><head><title>SEI - Erro</title></head><body>\
    <div id='divInfraExcecao'>Erro na pagina\xe7\xe3o: tempo esgotado.</div></body></html>";

#[test]
fn erro_500_com_tela_de_excecao_do_sei_expoe_status_e_mensagem() {
    let falha = decodificar_resposta(resposta(500, "text/html; charset=iso-8859-1", EXCECAO_SEI))
        .unwrap_err();
    assert_eq!(
        recusa(&falha),
        (500, Some("Erro na paginação: tempo esgotado."))
    );
}

#[test]
fn erro_503_em_texto_puro_e_429_e_404_nao_html() {
    let falha =
        decodificar_resposta(resposta(503, "text/plain", b"Service Unavailable\n")).unwrap_err();
    assert_eq!(recusa(&falha), (503, Some("Service Unavailable")));

    let falha =
        decodificar_resposta(resposta(429, "text/html", b"<title>Too Many</title>")).unwrap_err();
    assert_eq!(recusa(&falha), (429, Some("Too Many")));

    let falha = decodificar_resposta(resposta(
        404,
        "application/json",
        b"{\"erro\":\"nao achou\"}",
    ))
    .unwrap_err();
    assert_eq!(recusa(&falha), (404, Some("{\"erro\":\"nao achou\"}")));
}

#[test]
fn negacao_401_403_continua_sessao_nao_autorizada_com_status_e_mensagem() {
    for status in [401, 403] {
        let falha =
            decodificar_resposta(resposta(status, "text/plain", b"Acesso negado.")).unwrap_err();
        assert_eq!(
            recusa(&falha),
            (u16::try_from(status).unwrap(), Some("Acesso negado."))
        );
    }
}

struct CorpoProibido;

impl Read for CorpoProibido {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("o corpo de um 3xx não pode ser lido")
    }
}

#[test]
fn redirecionamento_expoe_o_status_sem_ler_o_corpo() {
    let falha = decodificar_resposta(RespostaWeb {
        status_code: 307,
        mime: "text/html".to_owned(),
        content_length: None,
        localizacao: None,
        corpo: CorpoProibido,
    })
    .unwrap_err();
    assert_eq!(recusa(&falha), (307, None));
}

#[test]
fn corpo_enorme_da_recusa_consome_so_64_kib_e_nao_e_resposta_excessiva() {
    let entregues = Cell::new(0);
    let falha = decodificar_resposta(RespostaWeb {
        status_code: 502,
        mime: "text/plain".to_owned(),
        content_length: Some(crate::MAX_HTML_BYTES * 2),
        localizacao: None,
        corpo: CorpoContado {
            entregues: &entregues,
            byte: b'y',
        },
    })
    .unwrap_err();
    assert_eq!(entregues.get(), LIMITE);
    let (status, texto) = recusa(&falha);
    assert_eq!(status, 502);
    assert!(texto.unwrap().ends_with('…'));
}

#[test]
fn segredos_e_urls_na_mensagem_sao_redigidos() {
    let hash = "ab".repeat(32);
    let corpo = format!(
        "<div id='divInfraExcecao'>Falhou https://sei.exemplo.gov.br/sei/controlador.php?\
         acao=x&amp;infra_hash={hash} com infra_hash={hash} PHPSESSID=segredo1 \
         chave {hash}</div>"
    );
    let falha = decodificar_resposta(resposta(500, "text/html", corpo.as_bytes())).unwrap_err();
    let (_, texto) = recusa(&falha);
    assert_eq!(
        texto,
        Some("Falhou <url> com infra_hash=<redigido> PHPSESSID=<redigido> chave <hex>")
    );
}

#[test]
fn bytes_invalidos_no_charset_declarado_nao_escondem_a_recusa() {
    let falha = decodificar_resposta(resposta(
        500,
        "text/plain; charset=utf-8",
        b"Erro \xe9 \xff",
    ))
    .unwrap_err();
    assert_eq!(recusa(&falha), (500, Some("Erro é ÿ")));
}

#[test]
fn recusa_com_corpo_vazio_tem_status_sem_mensagem() {
    let falha = decodificar_resposta(resposta(500, "text/html", b"")).unwrap_err();
    assert_eq!(recusa(&falha), (500, None));
}

#[test]
fn get_com_500_nao_repete_o_pedido_e_traz_a_mensagem() {
    let pasta = std::env::temp_dir().join(format!(
        "ies-recusa-http-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&pasta).unwrap();
    let cookie = pasta.join("cookie");
    std::fs::write(&cookie, "PHPSESSID=SINTETICO").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&cookie, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let url = format!(
        "https://sei.treinamento.unb.br/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
        "a".repeat(64)
    );
    let chamadas = Cell::new(0);
    let falha = read_box_via(
        &url,
        &cookie,
        "https://sei.treinamento.unb.br/sei",
        |_, _| {
            chamadas.set(chamadas.get() + 1);
            Ok(resposta(500, "text/html; charset=iso-8859-1", EXCECAO_SEI))
        },
    )
    .unwrap_err();
    assert_eq!(chamadas.get(), 1);
    assert_eq!(recusa(&falha).0, 500);
    assert!(recusa(&falha).1.is_some());
    std::fs::remove_dir_all(pasta).unwrap();
}
