use super::{Pasta, executar, json};
use std::fs;

// #873: PDF sintético válido, com offsets xref conferidos pelo Poppler.
// Diferente do recibo de upload legado, este é um documento interpretável.
const PDF_NATIVO: &[u8] = concat!(
    "%PDF-1.4\n",
    "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
    "2 0 obj\n<< /Type /Pages /Count 1 /Kids [3 0 R] >>\nendobj\n",
    "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>\nendobj\n",
    "4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
    "5 0 obj\n<< /Length 43 >>\nstream\nBT /F1 20 Tf 40 700 Td (NATIVO ALFA) Tj ET\nendstream\nendobj\n",
    "xref\n0 6\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n",
    "0000000115 00000 n \n0000000241 00000 n \n0000000311 00000 n \n",
    "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n403\n%%EOF\n",
)
.as_bytes();

#[test]
fn pdf_nativo_publica_teor_com_e_sem_cache_sem_alterar_original() {
    // Hipótese sintética: o PDF é semeado via bancada já existente. Não é
    // evidência de paridade com uma cassette original do SEI.
    let (saidas, eventos, documentos, arquivo) = executar(false, false, true, Some(PDF_NATIVO), None);
    assert_eq!(documentos, 1);
    assert_eq!(saidas.len(), 5);
    assert_eq!(arquivo.as_deref(), Some(PDF_NATIVO));
    assert_eq!(saidas[2].status.code(), Some(0));
    let original = json(&saidas[2]);
    let hash = "545f1f049a835f19d710b9213bf38b1c0b802bbdedfded624736a878356e46de";
    assert_eq!(original["documentos"][0]["anexo"]["sha256"], hash);
    assert!(original["documentos"][0].get("texto").is_none());
    assert!(
        original["documentos"][0]["anexo"]
            .get("extracao_pdf")
            .is_none()
    );
    for (i, saida) in saidas[3..].iter().enumerate() {
        assert_eq!(saida.status.code(), Some(0), "{:?}", saida.stderr);
        let lido = json(saida);
        let doc = &lido["documentos"][0];
        assert_eq!(doc["anexo"]["sha256"], hash);
        assert!(doc.get("arquivo").is_none());
        assert_eq!(doc["anexo"]["extracao_pdf"], "nativo_parcial");
        assert_eq!(doc.get("cache").is_some(), i == 1);
        assert_eq!(doc["texto"].as_str().map(str::trim), Some("NATIVO ALFA"));
    }
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "GET documento_download_anexo 200")
            .count(),
        3,
        "cada leitura do teor reutiliza o mesmo GET do original"
    );
}

// #873: ausência de texto recuperável não equivale a falha do motor.
#[test]
fn pdf_sem_texto_e_pdf_corrompido_preservam_original_e_estado_distinto() {
    let marcador = b"NATIVO ALFA";
    let mut vazio = PDF_NATIVO.to_vec();
    let inicio = vazio
        .windows(marcador.len())
        .position(|janela| janela == marcador)
        .unwrap();
    vazio
        .get_mut(inicio..inicio + marcador.len())
        .unwrap()
        .fill(b' ');
    for (pdf, hash, estado) in [
        (
            vazio.as_slice(),
            "23500893a3c396e9f3ab00fff6924f21ecf1a1b2ebaadbcd3bd039027648ddeb",
            "sem_texto_nativo",
        ),
        (
            b"%PDF-1.4\n1 0 obj\n%%EOF\n".as_slice(),
            "49286c45c90723857946342e0610a2b28974f176218f77f79efacc34cc4567bc",
            "falhou",
        ),
    ] {
        let (saidas, eventos, documentos, arquivo) = executar(false, false, true, Some(pdf), None);
        assert_eq!(documentos, 1);
        assert_eq!(saidas.len(), 5);
        assert_eq!(arquivo.as_deref(), Some(pdf));
        for (i, saida) in saidas.iter().skip(2).enumerate() {
            assert_eq!(saida.status.code(), Some(0), "{:?}", saida.stderr);
            let lido = json(saida);
            let doc = &lido["documentos"][0];
            assert_eq!(doc["anexo"]["sha256"], hash);
            assert!(doc.get("texto").is_none());
            if i == 0 {
                assert!(doc["anexo"].get("extracao_pdf").is_none());
            } else {
                assert_eq!(doc["anexo"]["extracao_pdf"], estado);
                assert_eq!(doc.get("cache").is_some(), i == 2);
            }
        }
        assert_eq!(
            eventos
                .iter()
                .filter(|e| e.as_str() == "GET documento_download_anexo 200")
                .count(),
            3
        );
    }
}

#[test]
fn pdf_motor_ausente_preserva_original_e_indica_limite() {
    let pasta = Pasta::nova();
    let sem_motor = pasta.0.join("sem-motor");
    fs::create_dir(&sem_motor).unwrap();
    let (saidas, eventos, documentos, arquivo) =
        executar(false, false, true, Some(PDF_NATIVO), Some(&sem_motor));
    assert_eq!(documentos, 1);
    assert_eq!(saidas.len(), 5);
    assert_eq!(arquivo.as_deref(), Some(PDF_NATIVO));
    for (i, saida) in saidas[3..].iter().enumerate() {
        assert_eq!(saida.status.code(), Some(0), "{:?}", saida.stderr);
        let lido = json(saida);
        let doc = &lido["documentos"][0];
        assert_eq!(
            doc["anexo"]["sha256"],
            "545f1f049a835f19d710b9213bf38b1c0b802bbdedfded624736a878356e46de"
        );
        assert_eq!(doc["anexo"]["extracao_pdf"], "motor_ausente");
        assert!(doc.get("texto").is_none());
        assert_eq!(doc.get("cache").is_some(), i == 1);
    }
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "GET documento_download_anexo 200")
            .count(),
        3
    );
}
