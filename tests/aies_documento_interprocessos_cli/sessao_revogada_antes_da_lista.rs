//! BDD-60-05: a sessão expira depois da árvore e antes da lista de versões.
//! Transição sintética não registrada na cassette `UnB` de histórico.
use super::*;

#[test]
fn bdd_60_05_cli_recusa_sessao_revogada_antes_da_lista_sem_fallback() {
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
        let (saida, trilha) = apoio::executar_sessao_revogada_antes_da_lista(&args);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "sessão revogada não deve entregar versão: stderr={} trilha={trilha:?}",
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
            "a árvore precisa ter oferecido o histórico: {trilha:?}"
        );
        assert_eq!(
            trilha
                .iter()
                .filter(|pedido| pedido.as_str() == "GET documento_versao_listar")
                .count(),
            1,
            "uma tentativa de lista; sem retry ou segundo caminho: {trilha:?}"
        );
        assert!(
            !trilha
                .iter()
                .any(|pedido| pedido == "GET documento_visualizar"),
            "a recusa do histórico impede baixar o corpo: {trilha:?}"
        );
        assert!(
            trilha.iter().all(|pedido| {
                pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"
            }),
            "não pode haver POST mutador ou fallback: {trilha:?}"
        );
    }
}
