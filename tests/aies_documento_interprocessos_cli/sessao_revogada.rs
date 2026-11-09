//! BDD-60-05: o link foi publicado, mas a sessão venceu antes do GET.
//! Corrida hipotética: não consta das respostas da cassette UnB.
use super::*;

#[test]
fn bdd_60_05_cli_recusa_sessao_revogada_depois_da_arvore() {
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
        let (saida, trilha) = apoio::executar_sessao_revogada_ao_visualizar(&args);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "negação da sessão não deve virar documento: stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let erro = json(&saida);
        assert_eq!(erro["status_http"], 403, "{erro}");
        assert_eq!(erro["documento"], DOC_A, "{erro}");
        assert_eq!(erro["documentos_lidos_antes"], 0, "{erro}");
        assert!(erro.get("documentos").is_none(), "conteúdo publicado: {erro}");
        assert_eq!(
            trilha
                .iter()
                .filter(|pedido| pedido.as_str() == "GET documento_visualizar")
                .count(),
            1,
            "um GET, sem retry nem fallback: {trilha:?}"
        );
        assert!(
            trilha
                .iter()
                .any(|pedido| pedido == "GET procedimento_visualizar"),
            "a árvore precede a revogação: {trilha:?}"
        );
        assert!(
            trilha.iter().all(|pedido| pedido.starts_with("GET ")
                || pedido == "POST protocolo_pesquisa_rapida"),
            "nenhuma escrita ou nova tentativa: {trilha:?}"
        );
    }
}
