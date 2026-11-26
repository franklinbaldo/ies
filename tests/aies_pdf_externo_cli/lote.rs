use super::*;

#[test]
fn lote_previa_de_dois_pdfs_nao_escreve() {
    let (saidas, eventos, total, _) = executar(None, true, false, &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"]);
    assert_eq!(saidas[0].status.code(), Some(0), "{:?}", saidas[0].stderr);
    let previa = json(&saidas[0]);
    assert_eq!(previa["escopo"], "previa_web_lote");
    assert_eq!(previa["itens"][0]["arquivo"]["nome"], "peticao.pdf");
    assert_eq!(previa["itens"][1]["arquivo"]["nome"], "segundo.pdf");
    assert_ne!(
        previa["itens"][0]["arquivo"]["sha256"],
        previa["itens"][1]["arquivo"]["sha256"]
    );
    assert_eq!(total, 0);
    assert!(
        !eventos
            .iter()
            .any(|evento| evento.starts_with("POST documento_upload_anexo")
                || evento == "POST documento_receber 302")
    );
}

#[test]
fn lote_segundo_pdf_invalido_nao_abre_previa() {
    let (saidas, eventos, total, _) = executar(None, true, false, &[b"nao e pdf"]);
    falha(&saidas[0], "Conteudo");
    assert_eq!(total, 0);
    assert!(
        !eventos
            .iter()
            .any(|evento| evento.starts_with("POST documento_receber")
                || evento.starts_with("POST documento_upload_anexo"))
    );
}

#[test]
fn lote_confirma_dois_pdfs_em_ordem_sem_retry() {
    let segundo_pdf = b"%PDF-1.4\n2 0 obj\n%%EOF\n";
    let (saidas, eventos, total, arquivos) = executar(None, false, true, &[segundo_pdf]);
    assert_eq!(saidas[0].status.code(), Some(0), "{:?}", saidas[0].stderr);
    let lote = json(&saidas[0]);
    assert_eq!(lote["escopo"], "confirmacao_web_lote");
    assert_eq!(lote["itens"][0]["arquivo"]["nome"], "peticao.pdf");
    assert_eq!(lote["itens"][1]["arquivo"]["nome"], "segundo.pdf");
    assert_eq!(lote["itens"][0]["numero_sei"], "050000001");
    let segundo = lote["itens"][1]["numero_sei"].as_str().unwrap();
    assert_eq!(segundo, "050000002");
    assert_eq!(total, 2);
    let primeira_escrita = eventos
        .iter()
        .position(|e| e == "POST documento_upload_anexo 200")
        .expect("primeiro upload");
    assert!(
        eventos[..primeira_escrita]
            .iter()
            .filter(|e| *e == "POST documento_receber 200")
            .count()
            >= 2,
        "todas as prévias precedem o primeiro upload"
    );
    for evento in [
        "POST documento_upload_anexo 200",
        "POST documento_receber 302",
    ] {
        assert_eq!(eventos.iter().filter(|e| e.as_str() == evento).count(), 2);
    }
    let arvore = json(&saidas[1])["documentos"].to_string();
    assert!(arvore.contains("050000001"));
    assert!(arvore.contains(segundo));
    assert_eq!(saidas.len(), 4);
    for (saida, numero) in saidas.iter().skip(2).zip(["050000001", "050000002"]) {
        assert_eq!(saida.status.code(), Some(0), "{:?}", saida.stderr);
        let lido = json(saida);
        assert_eq!(lido["documentos"][0]["numero"], numero);
        assert_eq!(lido["documentos"][0]["fonte"], "documento_download_anexo");
    }
    assert_eq!(
        arquivos,
        vec![
            Some(b"%PDF-1.4\n1 0 obj\n%%EOF\n".to_vec()),
            Some(segundo_pdf.to_vec()),
        ],
        "cada número SEI deve devolver seu próprio PDF original"
    );
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "GET documento_download_anexo 200")
            .count(),
        2
    );
}

#[test]
fn lote_segundo_cadastro_incerto_preserva_primeiro_e_nao_tenta_terceiro() {
    let (saidas, eventos, total, _) = executar(
        Some(2),
        false,
        false,
        &[b"%PDF-1.4\n2 0 obj\n%%EOF\n", b"%PDF-1.4\n3 0 obj\n%%EOF\n"],
    );
    falha(&saidas[0], "LoteInterrompido");
    let stderr = String::from_utf8_lossy(&saidas[0].stderr);
    let linha = stderr.lines().next().unwrap();
    let lote: Value =
        serde_json::from_str(linha.strip_prefix("Error: LoteInterrompido: ").unwrap()).unwrap();
    assert!(stderr.contains("TentativaNaoComprovada"));
    assert!(stderr.contains("Confirmacao("));
    assert_eq!(lote["confirmado"], false);
    assert_eq!(lote["itens"][0]["estado"], "confirmado");
    assert_eq!(lote["itens"][0]["numero_sei"], "050000001");
    assert_eq!(lote["itens"][1]["estado"], "nao_comprovado");
    assert_eq!(lote["itens"][1]["upload_tentado"], true);
    assert_eq!(lote["itens"][2]["estado"], "nao_tentado");
    assert_eq!(total, 2, "segundo efeito existiu, terceiro não foi tentado");
    for evento in [
        "POST documento_upload_anexo 200",
        "POST documento_receber 302",
    ] {
        assert_eq!(eventos.iter().filter(|e| e.as_str() == evento).count(), 2);
    }
}

#[test]
fn lote_segunda_previa_remota_recusada_impede_toda_escrita() {
    // 1 MiB ainda permite o primeiro PDF, mas não o segundo.
    // Uma prévia só do primeiro seguida da sua confirmação não basta.
    let mut segundo = b"%PDF-1.4\n".to_vec();
    segundo.resize(1024 * 1024 + 1, b' ');
    segundo.extend_from_slice(b"\n%%EOF\n");
    let (saidas, eventos, total, _) = executar_com_falha_previa(
        None,
        Some(FalhaPrevia::LimiteMenor(2)),
        false,
        false,
        &[&segundo],
    );
    falha(&saidas[0], "CadastroNaoConferido");
    let stderr = String::from_utf8_lossy(&saidas[0].stderr);
    assert!(
        stderr
            .lines()
            .any(|linha| linha.trim() == "PdfAcimaDoLimite"),
        "a causa tipada deve ser o limite observado: {stderr}"
    );
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "POST documento_receber 200")
            .count(),
        2
    );
    assert_eq!(total, 0);
    assert!(
        !eventos
            .iter()
            .any(|e| e.starts_with("POST documento_upload_anexo")
                || e == "POST documento_receber 302"),
        "falha na segunda prévia não permite escrita: {eventos:?}"
    );
}

#[test]
fn lote_segunda_previa_sem_hdn_anexos_impede_toda_escrita() {
    let (saidas, eventos, total, _) = executar_com_falha_previa(
        None,
        Some(FalhaPrevia::SemAnexos(2)),
        false,
        false,
        &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
    );
    falha(&saidas[0], "CampoOcultoAusenteNoFormulario");
    let stderr = String::from_utf8_lossy(&saidas[0].stderr);
    assert!(stderr.contains("campo: \"hdnAnexos\""), "{stderr}");
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "POST documento_receber 200")
            .count(),
        2
    );
    assert_eq!(total, 0);
    assert!(
        !eventos
            .iter()
            .any(|e| e.starts_with("POST documento_upload_anexo")
                || e == "POST documento_receber 302"),
        "falha na segunda prévia não permite escrita: {eventos:?}"
    );
}
