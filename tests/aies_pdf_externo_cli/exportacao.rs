use super::{Leitura, executar, falha, json};

#[test]
fn exportacao_zip_preserva_documento_cadastrado_e_nao_repete_geracao() {
    use sha2::Digest as _;
    use std::io::{Cursor, Read};

    // Hipótese de saída: o ZIP integral contém o PDF previamente cadastrado.
    // O formulário e o link de download têm replay original separado.
    let (saidas, eventos, documentos, arquivo) = executar(None, false, Leitura::Zip, &[]);
    assert_eq!(documentos, 1, "exportar não cria novo documento");
    assert_eq!(saidas[0].status.code(), Some(0), "cadastro: {eventos:?}");
    assert_eq!(saidas[1].status.code(), Some(0), "releitura: {eventos:?}");
    assert_eq!(
        saidas[2].status.code(),
        Some(0),
        "exportação: stderr={} HTTP={eventos:?}",
        String::from_utf8_lossy(&saidas[2].stderr)
    );
    let bytes = arquivo
        .into_iter()
        .next()
        .flatten()
        .expect("arquivo efetivamente produzido pela CLI");
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
    let (saidas, eventos, documentos, arquivo) = executar(None, false, Leitura::PdfProcesso, &[]);
    assert_eq!(documentos, 1, "exportar não cria novo documento");
    assert_eq!(saidas[0].status.code(), Some(0), "cadastro: {eventos:?}");
    assert_eq!(saidas[1].status.code(), Some(0), "releitura: {eventos:?}");
    assert_eq!(
        saidas[2].status.code(),
        Some(0),
        "exportação: stderr={} HTTP={eventos:?}",
        String::from_utf8_lossy(&saidas[2].stderr)
    );
    let bytes = arquivo
        .into_iter()
        .next()
        .flatten()
        .expect("arquivo efetivamente produzido pela CLI");
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

    // Hipótese de saída, não replay de corpo original: anexos com o mesmo
    // nome e tipo, mas números e bytes distintos; só o segundo deve sair.
    let (saidas, eventos, documentos, arquivo) = executar(
        None,
        false,
        Leitura::ZipSelecao(&["050000002"]),
        &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
    );
    assert_eq!(documentos, 2);
    for saida in &saidas {
        assert!(
            saida.status.success(),
            "stderr={} HTTP={eventos:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
    }
    let arvore = json(&saidas[2]);
    let docs = arvore["documentos"].as_array().unwrap();
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0]["numero_sei"], "050000001");
    assert_eq!(docs[1]["numero_sei"], "050000002");
    assert_ne!(docs[0]["id_documento"], docs[1]["id_documento"]);
    let bytes = arquivo.into_iter().next().flatten().unwrap();
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    assert_eq!(zip.len(), 1);
    let mut membro = zip.by_index(0).unwrap();
    assert_eq!(membro.name().unwrap().as_ref(), "050000002.pdf");
    let mut bytes = Vec::new();
    membro.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"%PDF-1.4\n2 0 obj\n%%EOF\n");
    for evento in ["POST procedimento_gerar_zip 200", "GET exibir_arquivo 200"] {
        assert_eq!(eventos.iter().filter(|e| e.as_str() == evento).count(), 1);
    }
}

#[test]
fn exportacao_zip_seletiva_documento_fora_do_processo_nao_gera_arquivo() {
    let (saidas, eventos, documentos, arquivo) = executar(
        None,
        false,
        Leitura::ZipSelecao(&["999999999"]),
        &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
    );
    assert_eq!(documentos, 2);
    for saida in &saidas[..3] {
        assert!(saida.status.success(), "HTTP={eventos:?}");
    }
    falha(&saidas[3], "DocumentoSelecionadoForaDaArvore");
    assert_eq!(arquivo, vec![None]);
    assert!(!eventos.iter().any(
        |e| e.starts_with("POST procedimento_gerar_zip") || e.starts_with("GET exibir_arquivo")
    ));
}

#[test]
fn exportacao_zip_seletiva_inclui_dois_de_tres_anexos() {
    use std::io::{Cursor, Read};

    let (saidas, eventos, documentos, arquivo) = executar(
        None,
        false,
        Leitura::ZipSelecao(&["050000003", "050000001"]),
        &[b"%PDF-1.4\n2 0 obj\n%%EOF\n", b"%PDF-1.4\n3 0 obj\n%%EOF\n"],
    );
    assert_eq!(documentos, 3);
    for saida in &saidas {
        assert!(
            saida.status.success(),
            "stderr={} HTTP={eventos:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
    }
    let bytes = arquivo.into_iter().next().flatten().unwrap();
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    assert_eq!(zip.len(), 2);
    for (nome, esperado) in [
        ("050000003.pdf", b"%PDF-1.4\n3 0 obj\n%%EOF\n"),
        ("050000001.pdf", b"%PDF-1.4\n1 0 obj\n%%EOF\n"),
    ] {
        let mut membro = zip.by_name(nome).unwrap();
        let mut bytes = Vec::new();
        membro.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes.as_slice(), esperado);
    }
    for evento in ["POST procedimento_gerar_zip 200", "GET exibir_arquivo 200"] {
        assert_eq!(eventos.iter().filter(|e| e.as_str() == evento).count(), 1);
    }
}

#[test]
fn exportacao_zip_seletiva_valido_seguido_de_inexistente_nao_gera_arquivo() {
    let (saidas, eventos, documentos, arquivo) = executar(
        None,
        false,
        Leitura::ZipSelecao(&["050000001", "999999999"]),
        &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
    );
    assert_eq!(documentos, 2);
    for saida in &saidas[..3] {
        assert!(saida.status.success(), "HTTP={eventos:?}");
    }
    falha(&saidas[3], "DocumentoSelecionadoForaDaArvore");
    assert_eq!(arquivo, vec![None]);
    assert!(!eventos.iter().any(
        |e| e.starts_with("POST procedimento_gerar_zip") || e.starts_with("GET exibir_arquivo")
    ));
}
