//! BDD-60-05: documento cancelado entre a oferta do histórico e seu GET.
//! Hipótese sintética; as cassettes UnB não registram esta corrida.
use super::*;

#[test]
fn bdd_60_05_cli_recusa_historico_cancelado_apos_arvore_sem_fallback() {
    for ocultar in [false, true] {
        let mut args = vec![
            "--json",
            "documento",
            DOC_A,
            "--processo",
            NUP_A,
            "--versao",
            "1",
            "--sem-cache",
        ];
        if ocultar {
            args.push("--ocultar-inacessiveis");
        }
        let (saida, trilha) = apoio::executar_cancelado_antes_da_lista(&args);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "a lista cancelada não pode produzir versão: stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let erro = json(&saida);
        assert_eq!(erro["status_http"], 403, "{erro}");
        assert_eq!(erro["documento"], DOC_A, "{erro}");
        assert_eq!(erro["documentos_lidos_antes"], 0, "{erro}");
        assert!(erro.get("documentos").is_none(), "{erro}");
        assert!(
            trilha
                .iter()
                .any(|pedido| pedido == "GET procedimento_visualizar"),
            "a identidade precisa ter vindo da árvore: {trilha:?}"
        );
        assert_eq!(
            trilha
                .iter()
                .filter(|pedido| pedido.as_str() == "GET documento_versao_listar")
                .count(),
            1,
            "uma consulta do histórico oferecido, sem retry: {trilha:?}"
        );
        assert!(
            !trilha
                .iter()
                .any(|pedido| pedido == "GET documento_visualizar"),
            "a recusa da lista impede buscar qualquer versão: {trilha:?}"
        );
        assert!(
            trilha.iter().all(|pedido| {
                pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"
            }),
            "nenhum POST mutador nem tentativa alternativa: {trilha:?}"
        );
    }
}
