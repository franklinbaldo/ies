//! Ensaios técnicos do transporte de #332; auxiliares a BDD-332-03/04.
use super::{multipart::preparar_pdf, recibo::FalhaUpload, upload::enviar_via};
use crate::formulario_web::CodificacaoDaTela;
use crate::transporte::{FalhaTransporte, RespostaWeb};
use std::io::Cursor;

const PDF: &[u8] = b"%PDF-1.4\n%%EOF\n";
const RECIBO: &str = "94e336aa#prova.pdf#application/pdf#15#11/10/2026 11:07:50#";

#[test]
fn upload_temporario_transmite_um_multipart_inteiro_e_aceita_so_recibo_observado() {
    let plano = preparar_pdf(
        "prova.pdf",
        PDF.len() as u64,
        "----ies332",
        CodificacaoDaTela::Utf8,
    )
    .unwrap();
    let mut tentativas = 0;
    let resultado = enviar_via(
        &plano,
        PDF,
        "https://exemplo.gov.br/sei/controlador.php?acao=documento_upload_anexo",
        "PHPSESSID=teste",
        "https://exemplo.gov.br",
        "https://exemplo.gov.br/sei/form",
        |pedido| {
            tentativas += 1;
            assert_eq!(pedido.tipo_conteudo, plano.tipo_conteudo());
            assert_eq!(pedido.tamanho, plano.tamanho_total());
            assert_eq!(pedido.cookie, "PHPSESSID=teste");
            assert_eq!(pedido.origem, "https://exemplo.gov.br");
            assert_eq!(pedido.referer, "https://exemplo.gov.br/sei/form");
            let mut corpo = Vec::new();
            pedido.corpo.read_to_end(&mut corpo).unwrap();
            assert_eq!(corpo.len() as u64, pedido.tamanho);
            assert!(corpo.windows(PDF.len()).any(|p| p == PDF));
            Ok(RespostaWeb {
                status_code: 200,
                mime: "text/plain; charset=iso-8859-1".into(),
                content_length: Some(RECIBO.len()),
                localizacao: None,
                corpo: Cursor::new(RECIBO.as_bytes().to_vec()),
            })
        },
        "prova.pdf",
        PDF.len() as u64,
    );
    assert_eq!(tentativas, 1);
    assert_eq!(resultado.unwrap().nome_upload, "94e336aa");
}

#[test]
fn recibo_do_upload_usa_o_mesmo_descritor_revalidado_na_previa() {
    use super::{FalhaPdf, abrir_pdf_para_envio, inspecionar_pdf};
    use std::fs;
    let caminho =
        std::env::temp_dir().join(format!("ies-332-reabertura-{}.pdf", std::process::id()));
    fs::write(&caminho, PDF).unwrap();
    let previa = inspecionar_pdf(&caminho, None).unwrap();
    let arquivo = abrir_pdf_para_envio(&caminho, &previa).unwrap();
    let plano = preparar_pdf(
        &previa.nome,
        previa.tamanho_bytes,
        "----ies332",
        CodificacaoDaTela::Utf8,
    )
    .unwrap();
    let mut tentativas = 0;
    let recibo_observado = format!(
        "94e336aa#{}#application/pdf#{}#11/10/2026 11:07:50#",
        previa.nome, previa.tamanho_bytes
    );
    let recibo = enviar_via(
        &plano,
        arquivo,
        "https://exemplo.gov.br/sei/controlador.php?acao=documento_upload_anexo",
        "PHPSESSID=teste",
        "https://exemplo.gov.br",
        "https://exemplo.gov.br/sei/form",
        |pedido| {
            tentativas += 1;
            let mut corpo = Vec::new();
            pedido.corpo.read_to_end(&mut corpo).unwrap();
            assert!(corpo.windows(PDF.len()).any(|janela| janela == PDF));
            Ok(RespostaWeb {
                status_code: 200,
                mime: "text/plain; charset=iso-8859-1".into(),
                content_length: Some(recibo_observado.len()),
                localizacao: None,
                corpo: Cursor::new(recibo_observado.as_bytes().to_vec()),
            })
        },
        &previa.nome,
        previa.tamanho_bytes,
    )
    .unwrap();
    assert_eq!(recibo.nome, previa.nome);
    assert_eq!(tentativas, 1);

    let mut adulterado = PDF.to_vec();
    adulterado[6] = b'X';
    fs::write(&caminho, &adulterado).unwrap();
    assert!(matches!(
        abrir_pdf_para_envio(&caminho, &previa),
        Err(FalhaPdf::DivergenteDaPrevia)
    ));
    fs::remove_file(caminho).unwrap();
}

#[test]
fn falha_de_rede_no_upload_temporario_eh_incerta_sem_reenvio() {
    let plano = preparar_pdf(
        "prova.pdf",
        PDF.len() as u64,
        "----ies332",
        CodificacaoDaTela::Utf8,
    )
    .unwrap();
    let mut tentativas = 0;
    let resultado = enviar_via(
        &plano,
        PDF,
        "https://exemplo.gov.br/sei/controlador.php?acao=documento_upload_anexo",
        "PHPSESSID=teste",
        "https://exemplo.gov.br",
        "https://exemplo.gov.br/sei/form",
        |_| -> Result<RespostaWeb<Cursor<Vec<u8>>>, FalhaTransporte> {
            tentativas += 1;
            Err(FalhaTransporte::Rede)
        },
        "prova.pdf",
        PDF.len() as u64,
    );
    assert_eq!(tentativas, 1);
    assert!(matches!(
        resultado,
        Err(FalhaUpload::NaoConfirmado { status_http: None })
    ));
}
