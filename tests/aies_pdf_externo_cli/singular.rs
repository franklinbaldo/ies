use super::{Leitura, executar, falha, json};

#[test]
fn bdd_332_03_binario_confirma_upload_cadastro_e_releitura_independente() {
    let (saidas, eventos, total, _) = executar(None, false, Leitura::Nenhuma, &[]);
    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "stdout={} stderr={} eventos={eventos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    let incluido = json(&saidas[0]);
    assert_eq!(incluido["confirmado"], true);
    assert_eq!(incluido["numero_sei"], "050000001");
    assert_eq!(incluido["reconciliado_por_leitura"], true);
    assert_eq!(
        saidas[1].status.code(),
        Some(0),
        "leitura externa: {:?}",
        saidas[1].stderr
    );
    assert!(
        json(&saidas[1])["documentos"]
            .to_string()
            .contains("050000001")
    );
    assert_eq!(total, 1, "upload temporário não duplica o documento");
    for acao in ["documento_upload_anexo", "documento_receber"] {
        assert_eq!(
            eventos
                .iter()
                .filter(|e| *e == &format!("POST {acao} 200") || *e == &format!("POST {acao} 302"))
                .count(),
            if acao == "documento_receber" { 2 } else { 1 },
            "recarga de tipo, upload e cadastro não podem causar retry: {eventos:?}"
        );
    }
    assert!(
        eventos
            .iter()
            .rposition(|e| e.starts_with("GET procedimento_visualizar"))
            > eventos
                .iter()
                .position(|e| e == "POST documento_receber 302")
    );
}

#[test]
fn bdd_332_04_resposta_sem_autoria_falha_sem_repetir_post() {
    let (saidas, eventos, total, _) = executar(Some(1), false, Leitura::Nenhuma, &[]);
    falha(&saidas[0], "TentativaNaoComprovada");
    assert_eq!(total, 1, "efeito existiu mesmo com resposta adulterada");
    assert_eq!(
        eventos
            .iter()
            .filter(|e| *e == "POST documento_upload_anexo 200")
            .count(),
        1
    );
    assert_eq!(
        eventos
            .iter()
            .filter(|e| *e == "POST documento_receber 302")
            .count(),
        1
    );
}

/// BDD-332-02: a prévia passa pelo binário e pelas rotas do anti-IES,
/// mas nenhuma linha de upload nem documento definitivo pode surgir.
#[test]
fn bdd_332_02_binario_consulta_previa_sem_upload_ou_cadastro() {
    let (saidas, eventos, total, _) = executar(None, true, Leitura::Nenhuma, &[]);
    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "stdout={} stderr={} eventos={eventos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    let previa = json(&saidas[0]);
    assert_eq!(previa["escopo"], "previa_web");
    assert_eq!(previa["id_tipo"], "293");
    assert_eq!(previa["tipo_solicitado"], "Anexo");
    assert_eq!(previa["limite_pdf_bytes"], 10 * 1024 * 1024);
    assert_eq!(previa["cadastro_sei_observado"], true);
    assert_eq!(previa["previa_remota"], true);
    assert_eq!(previa["confirmado"], false);
    assert_eq!(previa["escrita_remota"], false);
    assert_eq!(saidas[1].status.code(), Some(0));
    assert_eq!(total, 0, "a prévia não deve registrar documento");
    assert!(
        !json(&saidas[1])["documentos"]
            .to_string()
            .contains("050000001")
    );
    assert_eq!(
        eventos
            .iter()
            .filter(|evento| *evento == "POST documento_receber 200")
            .count(),
        1,
        "a recarga do tipo é navegação, não cadastro: {eventos:?}"
    );
    assert!(
        !eventos
            .iter()
            .any(|evento| evento.starts_with("POST documento_upload_anexo")
                || evento == "POST documento_receber 302"),
        "prévia nunca envia o PDF nem cadastra documento: {eventos:?}"
    );
}

#[test]
fn bdd_60_03_cli_baixa_pdf_original_do_estado_compartilhado() {
    // BDD-60-03 pré-escrito na #60: o mesmo PDF cadastrado deve ser lido
    // pela CLI real e salvo byte a byte, sem transformar conteúdo em HTML.
    let (saidas, eventos, documentos, arquivo) = executar(None, false, Leitura::Pdf, &[]);
    assert_eq!(documentos, 1, "um único documento definitivo");
    assert_eq!(
        saidas[2].status.code(),
        Some(0),
        "saída={} stderr={} HTTP={eventos:?}",
        String::from_utf8_lossy(&saidas[2].stdout),
        String::from_utf8_lossy(&saidas[2].stderr),
    );
    assert_eq!(
        arquivo,
        vec![Some(b"%PDF-1.4\n1 0 obj\n%%EOF\n".to_vec())],
        "o arquivo salvo deve ser o PDF original enviado, sem conversão"
    );
    let lido = json(&saidas[2]);
    assert_eq!(lido["documentos"][0]["numero"], "050000001");
    assert_eq!(lido["documentos"][0]["fonte"], "documento_download_anexo");
    assert_eq!(lido["documentos"][0]["anexo"]["mime"], "application/pdf");
    assert_eq!(lido["documentos"][0]["arquivo"]["nome"], "050000001.pdf");
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "GET documento_download_anexo 200")
            .count(),
        1,
        "o link assinado deve ser seguido uma só vez"
    );
}
