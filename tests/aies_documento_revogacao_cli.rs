//! #60 BDD-60-05: cancelamento concorrente entre árvore e corpo da CLI.
//! Prova sintética pelo binário real, HTTPS de loopback e SQLite compartilhado.
#![cfg(unix)]
#[path = "support/documento_interprocessos.rs"]
mod apoio;
use apoio::{DOC_A, NUP_A, executar_revogado_ao_visualizar, json};

#[test]
fn bdd_60_05_cli_recusa_url_revogada_apos_arvore_sem_tentar_alternativa() {
    // A árvore do processo ainda publica o href. Antes do GET de conteúdo,
    // o simulador muda a situação do documento no mesmo banco.
    // Não é uma transição capturada na cassette GO: é hipótese funcional.
    let (saida, trilha) = executar_revogado_ao_visualizar(&[
        "--json",
        "documento",
        DOC_A,
        "--processo",
        NUP_A,
        "--sem-cache",
    ]);

    assert_eq!(
        saida.status.code(),
        Some(1),
        "a URL revogada não deve produzir documento: stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let erro = json(&saida);
    assert_eq!(erro["status_http"], 403, "{erro}");
    assert_eq!(erro["documento"], DOC_A, "{erro}");
    assert!(erro.get("documentos").is_none(), "{erro}");
    assert_eq!(
        trilha
            .iter()
            .filter(|pedido| pedido.as_str() == "GET documento_visualizar")
            .count(),
        1,
        "uma única tentativa de conteúdo, sem retry/fallback: {trilha:?}"
    );
    assert!(
        trilha
            .iter()
            .any(|pedido| pedido == "GET procedimento_visualizar"),
        "a navegação deveria obter uma árvore antes da revogação: {trilha:?}"
    );
    assert!(
        trilha
            .iter()
            .all(|pedido| pedido.starts_with("GET ")
                || pedido == "POST protocolo_pesquisa_rapida"),
        "recusa não autoriza POST mutador: {trilha:?}"
    );
}
