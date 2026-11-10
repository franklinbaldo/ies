//! BDD-60-05: pesquisa sem --processo não pode escolher um de dois
//! documentos homônimos em processos distintos. Hipótese sintética:
//! o card UnB registra a pesquisa, não essa colisão de identidades.
use super::*;

#[test]
fn bdd_60_05_cli_sem_nup_recusa_numero_ambiguo_sem_publicar_corpos() {
    for args in [
        vec!["--json", "documento", DOC_A, "--sem-cache"],
        vec![
            "--json",
            "documento",
            DOC_A,
            "--ocultar-inacessiveis",
            "--sem-cache",
        ],
    ] {
        let (saida, trilha) = apoio::executar_numero_igual_em_outro_processo(&args);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "duas identidades não podem virar um documento: stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let erro = json(&saida);
        assert_eq!(erro["documento"], DOC_A, "{erro}");
        assert_eq!(erro["documentos_lidos_antes"], 0, "{erro}");
        assert!(
            erro["codigo"].as_str().is_some_and(|codigo| !codigo.is_empty()),
            "a recusa precisa ser classificada: {erro}"
        );
        assert!(erro.get("documentos").is_none(), "{erro}");
        assert_eq!(
            trilha
                .iter()
                .filter(|pedido| pedido.as_str() == "POST protocolo_pesquisa_rapida")
                .count(),
            1,
            "uma pesquisa, sem repetição: {trilha:?}"
        );
        assert!(
            !trilha.iter().any(|pedido| {
                pedido == "GET documento_visualizar"
                    || pedido == "GET documento_download_anexo"
            }),
            "nenhum corpo deve ser lido quando a busca é ambígua: {trilha:?}"
        );
        assert!(
            trilha.iter().all(|pedido| {
                pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"
            }),
            "nenhuma escrita ou nova busca de alternativa: {trilha:?}"
        );
    }
}

#[test]
fn bdd_60_05_humano_sem_nup_nao_exibe_corpo_ambiguo() {
    let (saida, trilha) = apoio::executar_numero_igual_em_outro_processo(&[
        "documento", DOC_A, "--sem-cache",
    ]);
    assert_eq!(
        saida.status.code(),
        Some(1),
        "a recusa também vale no modo humano: trilha={trilha:?}"
    );
    let apresentacao = format!(
        "{}\n{}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    assert!(
        !apresentacao.contains("primeiro processo")
            && !apresentacao.contains("segundo processo"),
        "não publicar conteúdo de nenhum candidato: {apresentacao}"
    );
    assert!(
        !trilha.iter().any(|pedido| pedido == "GET documento_visualizar"),
        "o erro humano não pode consultar um corpo arbitrário: {trilha:?}"
    );
}
