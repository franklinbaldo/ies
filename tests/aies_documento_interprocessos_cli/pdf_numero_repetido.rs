//! BDD-60-05: documento interno e PDF externo com o mesmo número SEI.
//! A colisão é sintética; os cards UnB não a registram. O binário é real,
//! passando por TLS local, HTTP anti-IES e um único estado SQLite.
use super::*;

#[test]
fn bdd_60_05_cli_numero_repetido_entre_pdf_e_interno_recusa_sem_fallback() {
    for ocultar in [false, true] {
        let mut args = vec![
            "--json",
            "documento",
            DOC_A,
            "--processo",
            NUP_A,
            "--sem-cache",
        ];
        if ocultar {
            args.push("--ocultar-inacessiveis");
        }
        let (saida, trilha) = apoio::executar_pdf_numero_repetido(&args);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "a CLI não pode escolher corpo interno ou PDF arbitrariamente: \
             stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let erro = json(&saida);
        assert_eq!(erro["documento"], DOC_A, "{erro}");
        assert_eq!(erro["subcodigo"], "numero_sei_repetido_na_arvore", "{erro}");
        assert_eq!(erro["documentos_lidos_antes"], 0, "{erro}");
        assert!(erro.get("documentos").is_none(), "{erro}");
        assert!(
            trilha.iter().any(|p| p == "GET procedimento_visualizar"),
            "a ambiguidade deve surgir da árvore: {trilha:?}"
        );
        assert!(
            !trilha.iter().any(|p| {
                p == "GET documento_visualizar" || p == "GET documento_download_anexo"
            }),
            "não buscar nenhum dos dois corpos em conflito: {trilha:?}"
        );
        assert!(
            trilha
                .iter()
                .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
            "sem escrita ou retry: {trilha:?}"
        );
    }
}
