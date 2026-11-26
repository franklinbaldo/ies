//! Saída do documento externo em `ies documento DOC` (Refs #60, fatia 3;
//! contracts/cli/documento-obter-externo.md): o objeto `anexo` do JSON,
//! e a linha humana do anexo. Bytes do
//! arquivo nunca vão para a saída padrão.
use crate::anexo_documento::AnexoLido;
use serde_json::{Value, json};

/// `mime`, `tamanho`, `sha256`, `nome_original` e `disposicao`.
pub(super) fn json_do_anexo(a: &AnexoLido) -> Value {
    let mut valor = json!({
        "mime": a.mime,
        "tamanho": a.tamanho,
        "sha256": a.sha256,
        "nome_original": a.nome_original,
        "disposicao": a.disposicao.como_texto(),
    });
    if let Some(estado) = a.estado_pdf()
        && let Some(objeto) = valor.as_object_mut()
    {
        objeto.insert("extracao_pdf".to_owned(), json!(estado));
    }
    valor
}

/// A linha humana do anexo; sem texto nem arquivo, a dica de `--saida`.
pub(super) fn linha_do_anexo(a: &AnexoLido, gravado: bool) -> String {
    let nome_original = &a.nome_original;
    let nome = format!(", nome original {nome_original:?}");
    let dica = if gravado || a.texto().is_some() {
        ""
    } else {
        " Use --saida DIR para gravar o arquivo."
    };
    let extracao = a.estado_pdf().map_or_else(String::new, |estado| {
        format!(" Extração PDF: {estado}; teor integral não certificado.")
    });
    format!(
        "Anexo {}: {} bytes, sha256 {}{nome}.{extracao}{dica}",
        a.mime, a.tamanho, a.sha256
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anexo_documento::{ConteudoDoAnexo, Disposicao};

    fn anexo(texto: Option<&str>) -> AnexoLido {
        AnexoLido {
            mime: "application/pdf".to_owned(),
            tamanho: 12,
            sha256: "ab".repeat(32),
            nome_original: "Minuta.pdf".to_owned(),
            disposicao: Disposicao::Inline,
            conteudo: texto.map_or(ConteudoDoAnexo::Nenhum, |t| {
                ConteudoDoAnexo::Texto(t.to_owned())
            }),
        }
    }

    #[test]
    fn json_e_linha_do_anexo_sem_bytes_do_arquivo() {
        assert_eq!(
            json_do_anexo(&anexo(None)),
            json!({"mime": "application/pdf", "tamanho": 12, "sha256": "ab".repeat(32),
                   "nome_original": "Minuta.pdf", "disposicao": "inline"})
        );
        let linha = linha_do_anexo(&anexo(None), false);
        assert!(
            linha.ends_with("Use --saida DIR para gravar o arquivo."),
            "{linha}"
        );
        assert!(linha.contains("12 bytes"));
        assert!(!linha_do_anexo(&anexo(Some("t")), false).contains("--saida"));
        assert!(!linha_do_anexo(&anexo(None), true).contains("--saida"));
    }
    #[test]
    fn linha_pdf_declara_estado_sem_negar_ocr_nem_certificar_teor() {
        use crate::anexo_documento::ExtracaoPdf;

        for pdf in [
            ExtracaoPdf::NativoParcial("texto".to_owned()),
            ExtracaoPdf::NativoComDiagnostico("texto".to_owned()),
            ExtracaoPdf::MistoParcial("texto".to_owned()),
            ExtracaoPdf::OcrParcial("texto".to_owned()),
            ExtracaoPdf::MistoComDiagnostico("texto".to_owned()),
            ExtracaoPdf::OcrComDiagnostico("texto".to_owned()),
            ExtracaoPdf::SemTextoNativo,
            ExtracaoPdf::OcrIndisponivel,
            ExtracaoPdf::MotorAusente,
            ExtracaoPdf::Falhou,
        ] {
            let estado = pdf.estado();
            let mut a = anexo(None);
            a.conteudo = ConteudoDoAnexo::Pdf(pdf);
            let linha = linha_do_anexo(&a, false);
            assert!(linha.contains(&format!("Extração PDF: {estado};")));
            assert!(linha.contains("teor integral não certificado."));
            assert!(!linha.contains("OCR não executado"));
            assert_eq!(json_do_anexo(&a)["extracao_pdf"], estado);
        }
    }
}
