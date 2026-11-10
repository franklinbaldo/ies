//! BDD-60-05 preexistente: --processo NUP deve recusar identidade
//! ambígua ou ausente no processo, também em saída humana.
//! O cenário é hipótese sintética; não é confronto com cassette.
use super::*;

#[test]
fn bdd_60_05_cli_humana_recusa_duplicidade_na_arvore_do_processo() {
    for ocultar in [false, true] {
        let mut args = vec!["documento", DOC_A, "--processo", NUP_A, "--sem-cache"];
        if ocultar {
            args.push("--ocultar-inacessiveis");
        }
        let (saida, trilha) = apoio::executar_numero_repetido(&args);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "número repetido não é documento acessível: trilha={trilha:?}"
        );
        let mensagem = format!(
            "{}\n{}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        );
        assert!(
            mensagem.contains("NumeroSeiRepetidoNaArvore"),
            "a pessoa deve ver a causa específica: {mensagem}"
        );
        assert!(
            !mensagem.contains("Conteudo exclusivo") && !mensagem.contains("Outro corpo"),
            "a ambiguidade não publica documento escolhido arbitrariamente: {mensagem}"
        );
        assert!(
            trilha
                .iter()
                .any(|pedido| pedido == "GET procedimento_visualizar"),
            "a recusa deve partir da árvore oferecida: {trilha:?}"
        );
        assert!(
            !trilha.iter().any(|pedido| {
                pedido == "GET documento_visualizar" || pedido == "GET documento_download_anexo"
            }),
            "nenhum corpo de candidato pode ser baixado: {trilha:?}"
        );
        assert!(
            trilha.iter().all(|pedido| {
                pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"
            }),
            "a pesquisa rápida é navegação, sem POST mutador: {trilha:?}"
        );
    }
}

#[test]
fn bdd_60_05_cli_humana_nao_oferece_documento_de_outro_nup() {
    let (saida, trilha) = executar(
        true,
        &["documento", DOC_B, "--processo", NUP_A, "--sem-cache"],
    );
    assert_eq!(
        saida.status.code(),
        Some(1),
        "um NUP explícito não aceita substituto: trilha={trilha:?}"
    );
    let mensagem = format!(
        "{}\n{}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    assert!(
        mensagem.contains("NumeroSeiAusenteDaArvoreDoProcesso"),
        "a recusa humana deve indicar o contexto: {mensagem}"
    );
    assert!(
        !mensagem.contains("segundo processo"),
        "não publicar corpo do documento alheio: {mensagem}"
    );
    assert!(
        !trilha
            .iter()
            .any(|pedido| pedido == "GET documento_visualizar"),
        "nenhum GET de corpo de DOC_B: {trilha:?}"
    );
    assert!(
        trilha.iter().all(|pedido| {
            pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"
        }),
        "somente navegação da origem, sem POST mutador: {trilha:?}"
    );
}
