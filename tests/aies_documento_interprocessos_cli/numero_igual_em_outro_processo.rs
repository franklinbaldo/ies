//! BDD-60-05: mesmo número SEI em processos distintos não deve causar
//! seleção de corpo alheio quando --processo delimita explicitamente a busca.
//! Hipótese sintética não capturada nos cards UnB de número e busca.
use super::*;

#[test]
fn bdd_60_05_numero_igual_em_processos_distintos_respeita_nup_explicito() {
    for (nup, id, trecho, indevido) in [
        (NUP_A, "42", "primeiro processo", "segundo processo"),
        (NUP_B, "43", "segundo processo", "primeiro processo"),
    ] {
        let (saida, trilha) = apoio::executar_numero_igual_em_outro_processo(&[
            "--json",
            "documento",
            DOC_A,
            "--processo",
            nup,
            "--sem-cache",
        ]);
        assert_eq!(
            saida.status.code(),
            Some(0),
            "o contexto NUP deve selecionar somente sua identidade: stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let valor = json(&saida);
        let documentos = valor["documentos"].as_array().expect("documentos lidos");
        assert_eq!(documentos.len(), 1, "{valor}");
        let doc = &documentos[0];
        assert_eq!(doc["numero"], DOC_A, "{valor}");
        assert_eq!(doc["processo"]["nup"], nup, "{valor}");
        assert_eq!(doc["id_documento"], id, "{valor}");
        let corpo = doc["texto"].as_str().expect("texto da identidade pedida");
        assert!(corpo.contains(trecho), "{valor}");
        assert!(
            !corpo.contains(indevido),
            "não publicar corpo de outro NUP: {valor}"
        );
        assert_eq!(valor["requisicoes"].as_u64(), Some(trilha.len() as u64));
        assert!(
            trilha.iter().any(|p| p == "GET procedimento_visualizar"),
            "a árvore do NUP explícito deve ter sido consultada: {trilha:?}"
        );
        assert!(
            trilha
                .iter()
                .filter(|p| p.as_str() == "GET documento_visualizar")
                .count()
                == 1,
            "exatamente um corpo, sem fallback ou retry: {trilha:?}"
        );
        assert!(
            trilha
                .iter()
                .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
            "nenhum POST mutador: {trilha:?}"
        );
    }
}
