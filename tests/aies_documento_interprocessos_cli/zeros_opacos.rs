//! BDD-60-05: o SEI pode localizar numero SEI sem zeros iniciais,
//! mas a CLI nao pode confundir a correspondencia de busca com identidade.
//! A UnB observou 11505 -> 0011505 em 04/11; os IDs aqui sao sinteticos.
//! Fonte: docs/validacao-live/2026-11-04-documento-obter.md.
use super::*;

#[test]
fn bdd_60_05_cli_recusa_numero_sem_zeros_mesmo_com_ocultacao() {
    for ocultar in [false, true] {
        let mut args = vec!["--json", "documento", "11519", "--sem-cache"];
        if ocultar {
            args.push("--ocultar-inacessiveis");
        }
        let (saida, trilha) = executar(true, &args);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "numero da arvore nao pode substituir numero pedido: stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let erro = json(&saida);
        assert_eq!(erro["documento"], "11519", "{erro}");
        assert_eq!(erro["documentos_lidos_antes"], 0, "{erro}");
        assert!(erro.get("documentos").is_none(), "{erro}");
        assert_eq!(
            trilha
                .iter()
                .filter(|pedido| pedido.as_str() == "POST protocolo_pesquisa_rapida")
                .count(),
            1,
            "a pesquisa do SEI deve encontrar o documento com zeros: {trilha:?}"
        );
        assert!(
            trilha
                .iter()
                .any(|pedido| pedido == "GET procedimento_visualizar"),
            "a recusa deve confrontar a arvore servida, nao falhar antes da selecao: {trilha:?}"
        );
        assert!(
            !trilha
                .iter()
                .any(|pedido| pedido == "GET documento_visualizar"),
            "nao baixar conteudo quando o numero SEI da arvore diverge: {trilha:?}"
        );
        assert!(
            trilha
                .iter()
                .all(|pedido| pedido.starts_with("GET ")
                    || pedido == "POST protocolo_pesquisa_rapida"),
            "sem POST mutador, fallback ou retry: {trilha:?}"
        );
    }
}
