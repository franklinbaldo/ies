//! BDD-60-05: identidade cancelada não publica histórico na navegação.
//! Ensaio sintético da CLI real, TLS loopback, anti-IES e SQLite.
use super::*;

#[test]
fn bdd_60_05_cli_recusa_versao_de_documento_cancelado_sem_consultar_historico() {
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
        let (saida, trilha) = executar_cancelado(&args);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "versão de DOC cancelado foi oferecida: stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let erro = json(&saida);
        assert_eq!(erro["documento"], DOC_A, "{erro}");
        assert_eq!(
            erro["subcodigo"],
            "documento_cancelado_sem_conteudo_na_arvore",
            "{erro}"
        );
        assert!(erro.get("documentos").is_none(), "{erro}");
        assert!(
            trilha
                .iter()
                .any(|pedido| pedido == "GET procedimento_visualizar"),
            "é preciso observar a árvore antes da recusa: {trilha:?}"
        );
        assert!(
            !trilha.iter().any(|pedido| {
                pedido == "GET documento_versao_listar"
                    || pedido == "GET documento_visualizar"
            }),
            "não acessar histórico nem corpo cancelados: {trilha:?}"
        );
        assert!(
            trilha.iter().all(|pedido| {
                pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"
            }),
            "não repetir acesso nem fazer POST mutador: {trilha:?}"
        );
    }
}
