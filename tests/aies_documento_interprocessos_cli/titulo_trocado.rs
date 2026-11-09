//! BDD-60-05: identidade muda entre a árvore e o GET do conteúdo.
//! Hipótese sintética, não observada na cassette original `UnB`.
use super::*;

#[test]
fn bdd_60_05_cli_recusa_titulo_divergente_apos_arvore_sem_fallback() {
    let (saida, trilha) = apoio::executar_titulo_trocado(&[
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
        "a CLI não pode aceitar o título trocado: stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let erro = json(&saida);
    assert_eq!(
        erro["subcodigo"], "titulo_do_conteudo_de_outro_documento",
        "{erro}"
    );
    assert_eq!(erro["documento"], DOC_A, "{erro}");
    assert_eq!(erro["documentos_lidos_antes"], 0, "{erro}");
    assert!(erro.get("documentos").is_none(), "vazou conteúdo: {erro}");
    assert_eq!(
        trilha
            .iter()
            .filter(|pedido| pedido.as_str() == "GET documento_visualizar")
            .count(),
        1,
        "um único GET do conteúdo divergente, sem retry: {trilha:?}"
    );
    assert!(
        trilha
            .iter()
            .any(|pedido| pedido == "GET procedimento_visualizar"),
        "é necessário ler a árvore antes da divergência: {trilha:?}"
    );
    assert!(
        trilha
            .iter()
            .all(|pedido| pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"),
        "nenhum POST mutador nem fallback: {trilha:?}"
    );
}
