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
    let (saidas, eventos, documentos, arquivo) =
        executar(false, false, true, Some(PDF_NATIVO), None);
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

// Fixture sintética: página nativa, imagem monocromática sem camada textual,
// página em branco. PDF ASCII com xref e stream FlateDecode verificados.
const PDF_MISTO: &[u8] = concat!(
    "%PDF-1.4\n",
    "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
    "2 0 obj\n<< /Type /Pages /Count 3 /Kids [3 0 R 5 0 R 7 0 R] >>\nendobj\n",
    "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 9 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    "4 0 obj\n<< /Length 43 >>\nstream\nBT /F1 20 Tf 40 700 Td (NATIVO ALFA) Tj ET\nendstream\nendobj\n",
    "5 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /XObject << /Im0 10 0 R >> >> /Contents 6 0 R >>\nendobj\n",
    "6 0 obj\n<< /Length 33 >>\nstream\nq 520 0 0 80 35 660 cm /Im0 Do Q\nendstream\nendobj\n",
    "7 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << >> /Contents 8 0 R >>\nendobj\n",
    "8 0 obj\n<< /Length 0 >>\nstream\nendstream\nendobj\n",
    "9 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
    "10 0 obj\n<< /Type /XObject /Subtype /Image /Width 520 /Height 80 /ColorSpace /DeviceGray /BitsPerComponent 1 /Filter [/ASCIIHexDecode /FlateDecode] /Length 998 >>\nstream\n",
    "78DAEDD4B16EDB301006600A1AB404E0DAA1F00D7D894EBD87E91364CB1084DEB404E5235580068D7D050B7C01125D3810BCDCD1B62CA94EBA66E0BF18B4C4CFE25977443535359F24A78E22D06020786C4FCAD317D5264D5919478D2C66B2F455B5196243B9235238344A3DE3022831145C01C37B7CCB807649990DA0900D527A0BB868C631832D8023CF8F42F34C197B179F51160C00F12D3183F7446027BE9A6E40461B92F9C3C04100D93345BEC5B940B2180AD0271DC9BA64082601F8D857800062643884C3B70B6093C9189CA3DF0BA00598A68CFF05600DF0B774340240EC3680D737E017A6C7023C301004808CFC1B6E2AC0AB005C013E972B540182BD03BCFE64C0DE0512DE00FB01706A42BF071E90B745E3C70BA0606833CEFD1DE028C0B4070E02780A6E03C8C57F8081CB16C615502ABA07A09723BC0FB81D901178DB4C2E6C8A38B93BC02CC0BC3EC2028C2BC07D0C841D9094E66D3D0D6A031C9B3B2F5201FCFA455A005B80F38BF40EA0E7F05780B8078C6540737F6C001BF6CD64B977AEC0D24C0C44D35B3A7037C5A74B338D0268BF6FE77E3C0369D5CE2F02D0C4F300F9A99E2EED5CFE4688FB8132740C9C54019681820538763C5018B80D940DD09E479A144F80CC9F65A4A12325A5255700C8DFD532D2F20FE4C3D4D4D4D4D4D4D4D4D4D4D4D47CEABC0125FDBC92>\n",
    "endstream\nendobj\n",
    "xref\n0 11\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n",
    "0000000127 00000 n \n0000000253 00000 n \n0000000345 00000 n \n0000000476 00000 n \n",
    "0000000558 00000 n \n0000000662 00000 n \n0000000710 00000 n \n0000000780 00000 n \n",
    "trailer\n<< /Size 11 /Root 1 0 R >>\nstartxref\n1965\n%%EOF\n",
)
.as_bytes();

#[test]
fn pdf_misto_recupera_texto_em_ordem_sem_alterar_original() {
    let (saidas, eventos, documentos, arquivo) =
        executar(false, false, true, Some(PDF_MISTO), None);
    assert_eq!(documentos, 1);
    assert_eq!(saidas.len(), 5);
    assert_eq!(arquivo.as_deref(), Some(PDF_MISTO));
    let hash = "9955c0dd918f9823cbc75e2bc9c60b1851f66bdd2554e53ee7c16589bfef67b3";
    for (i, saida) in saidas.iter().skip(2).enumerate() {
        assert_eq!(saida.status.code(), Some(0), "{:?}", saida.stderr);
        let lido = json(saida);
        let doc = &lido["documentos"][0];
        assert_eq!(doc["anexo"]["sha256"], hash);
        if i == 0 {
            assert!(doc.get("texto").is_none());
            assert!(doc["anexo"].get("extracao_pdf").is_none());
        } else {
            let texto = doc["texto"].as_str().unwrap();
            assert_eq!(
                texto.split_whitespace().collect::<Vec<_>>(),
                ["NATIVO", "ALFA", "DOCUMENTO", "DIGITALIZADO", "BETA"]
            );
            assert_eq!(doc["anexo"]["extracao_pdf"], "misto_parcial");
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

// Retira apenas os glifos nativos, sem deslocar objetos nem a tabela xref.
// Restam uma página digitalizada e duas páginas sem texto.
fn pdf_apenas_ocr() -> Vec<u8> {
    let marcador = b"NATIVO ALFA";
    let mut pdf = PDF_MISTO.to_vec();
    let inicio = pdf
        .windows(marcador.len())
        .position(|janela| janela == marcador)
        .unwrap();
    pdf[inicio..inicio + marcador.len()].fill(b' ');
    pdf
}

fn conferir_pdf_ocr(
    pdf: &[u8],
    caminho: Option<&std::path::Path>,
    estado: &str,
    texto: Option<&str>,
) {
    use sha2::{Digest, Sha256};

    let (saidas, eventos, documentos, arquivo) = executar(false, false, true, Some(pdf), caminho);
    assert_eq!(documentos, 1);
    assert_eq!(saidas.len(), 5);
    assert_eq!(arquivo.as_deref(), Some(pdf));
    let hash = hex::encode(Sha256::digest(pdf));
    for (i, saida) in saidas.iter().skip(2).enumerate() {
        assert_eq!(saida.status.code(), Some(0), "{:?}", saida.stderr);
        let lido = json(saida);
        let doc = &lido["documentos"][0];
        assert_eq!(doc["anexo"]["sha256"], hash);
        if i == 0 {
            assert!(doc.get("texto").is_none());
            assert!(doc["anexo"].get("extracao_pdf").is_none());
        } else {
            assert_eq!(doc["anexo"]["extracao_pdf"], estado);
            assert_eq!(doc.get("cache").is_some(), i == 2);
            assert_eq!(
                doc.get("texto")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim),
                texto
            );
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

#[test]
fn pdf_sem_texto_nativo_recupera_apenas_ocr() {
    conferir_pdf_ocr(
        &pdf_apenas_ocr(),
        None,
        "ocr_parcial",
        Some("DOCUMENTO DIGITALIZADO BETA"),
    );
}

#[test]
fn pdf_ocr_ausente_distingue_misto_de_digitalizado() {
    use std::os::unix::fs::symlink;

    let pasta = Pasta::nova();
    let motores = pasta.0.join("somente-poppler");
    fs::create_dir(&motores).unwrap();
    let path = std::env::var_os("PATH").unwrap();
    for nome in ["pdftotext", "pdftoppm"] {
        let motor = std::env::split_paths(&path)
            .map(|diretorio| diretorio.join(nome))
            .find(|arquivo| arquivo.is_file())
            .unwrap();
        symlink(fs::canonicalize(motor).unwrap(), motores.join(nome)).unwrap();
    }
    conferir_pdf_ocr(
        PDF_MISTO,
        Some(&motores),
        "nativo_com_diagnostico",
        Some("NATIVO ALFA"),
    );
    conferir_pdf_ocr(&pdf_apenas_ocr(), Some(&motores), "ocr_indisponivel", None);
}
