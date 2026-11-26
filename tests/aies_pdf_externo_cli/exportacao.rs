use super::{Leitura, executar, json};

#[test]
fn exportacao_zip_preserva_documento_cadastrado_e_nao_repete_geracao() {
    use sha2::Digest as _;
    use std::io::{Cursor, Read};

    // Hipótese de saída: o ZIP integral contém o PDF previamente cadastrado.
    // O formulário e o link de download têm replay original separado.
    let (saidas, eventos, documentos, arquivo) = executar(false, false, Leitura::Zip);
    assert_eq!(documentos, 1, "exportar não cria novo documento");
    assert_eq!(saidas[0].status.code(), Some(0), "cadastro: {eventos:?}");
    assert_eq!(saidas[1].status.code(), Some(0), "releitura: {eventos:?}");
    assert_eq!(
        saidas[2].status.code(),
        Some(0),
        "exportação: stderr={} HTTP={eventos:?}",
        String::from_utf8_lossy(&saidas[2].stderr)
    );
    let bytes = arquivo.expect("arquivo efetivamente produzido pela CLI");
    let resultado = json(&saidas[2]);
    assert_eq!(resultado["formato"], "zip");
    assert_eq!(resultado["tamanho_bytes"], bytes.len());
    assert_eq!(
        resultado["sha256"],
        hex::encode(sha2::Sha256::digest(&bytes))
    );
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    assert_eq!(zip.len(), 1, "um único documento foi cadastrado");
    let mut conteudo = Vec::new();
    zip.by_index(0).unwrap().read_to_end(&mut conteudo).unwrap();
    assert_eq!(conteudo, b"%PDF-1.4\n1 0 obj\n%%EOF\n");
    for evento in ["POST procedimento_gerar_zip 200", "GET exibir_arquivo 200"] {
        assert_eq!(
            eventos.iter().filter(|e| e.as_str() == evento).count(),
            1,
            "{evento}: {eventos:?}"
        );
    }
}

#[test]
fn exportacao_pdf_de_um_unico_anexo_preserva_bytes_e_nao_repete_geracao() {
    use sha2::Digest as _;

    // Hipótese mínima: um único anexo externo, sem conversão ou composição.
    // Os bytes de teste são sintéticos; não demonstram fidelidade do PDF do SEI.
    let (saidas, eventos, documentos, arquivo) = executar(false, false, Leitura::PdfProcesso);
    assert_eq!(documentos, 1, "exportar não cria novo documento");
    assert_eq!(saidas[0].status.code(), Some(0), "cadastro: {eventos:?}");
    assert_eq!(saidas[1].status.code(), Some(0), "releitura: {eventos:?}");
    assert_eq!(
        saidas[2].status.code(),
        Some(0),
        "exportação: stderr={} HTTP={eventos:?}",
        String::from_utf8_lossy(&saidas[2].stderr)
    );
    let bytes = arquivo.expect("arquivo efetivamente produzido pela CLI");
    assert_eq!(bytes, b"%PDF-1.4\n1 0 obj\n%%EOF\n");
    let resultado = json(&saidas[2]);
    assert_eq!(resultado["formato"], "pdf");
    assert_eq!(resultado["tamanho_bytes"], bytes.len());
    assert_eq!(
        resultado["sha256"],
        hex::encode(sha2::Sha256::digest(&bytes))
    );
    for evento in ["POST procedimento_gerar_pdf 200", "GET exibir_arquivo 200"] {
        assert_eq!(
            eventos.iter().filter(|e| e.as_str() == evento).count(),
            1,
            "{evento}: {eventos:?}"
        );
    }
}

#[test]
fn exportacao_zip_seletiva_inclui_so_o_segundo_anexo() {
    use std::io::{Cursor, Read};

    // Hipótese de saída, não replay de corpo original: dois anexos distintos
    // persistidos; somente o segundo deve chegar ao arquivo.
    let (saidas, eventos, documentos, arquivo) =
        executar(false, false, Leitura::ZipSelecao("050000002"));
    assert_eq!(documentos, 2);
    for saida in &saidas {
        assert!(
            saida.status.success(),
            "stderr={} HTTP={eventos:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
    }
    let mut zip = zip::ZipArchive::new(Cursor::new(arquivo.unwrap())).unwrap();
    assert_eq!(zip.len(), 1);
    let mut membro = zip.by_index(0).unwrap();
    assert_eq!(membro.name(), "050000002.pdf");
    let mut bytes = Vec::new();
    membro.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"%PDF-1.4\n2 0 obj\n%%EOF\n");
    for evento in ["POST procedimento_gerar_zip 200", "GET exibir_arquivo 200"] {
        assert_eq!(eventos.iter().filter(|e| e.as_str() == evento).count(), 1);
    }
}

#[test]
fn exportacao_zip_seletiva_documento_fora_do_processo_nao_gera_arquivo() {
    let (saidas, eventos, documentos, arquivo) =
        executar(false, false, Leitura::ZipSelecao("999999999"));
    assert_eq!(documentos, 2);
    for saida in &saidas[..3] {
        assert!(saida.status.success(), "HTTP={eventos:?}");
    }
    assert_eq!(saidas[3].status.code(), Some(1));
    assert!(arquivo.is_none());
    assert!(!eventos.iter().any(|e| e.starts_with("POST procedimento_gerar_zip")
        || e.starts_with("GET exibir_arquivo")));
}
