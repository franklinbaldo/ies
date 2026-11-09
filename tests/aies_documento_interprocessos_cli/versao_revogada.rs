//! BDD-60-05: a versão escolhida pode perder acesso depois do histórico.
//! Hipótese temporal sintética; nenhuma cassette observa essa corrida.
use super::*;

#[test]
fn bdd_60_05_cli_recusa_versao_revogada_entre_lista_e_corpo() {
    let (saida, trilha) = apoio::executar_versao_revogada_ao_visualizar(&[
        "--json",
        "documento",
        DOC_A,
        "--processo",
        NUP_A,
        "--versao",
        "1",
        "--sem-cache",
    ]);
    assert_eq!(
        saida.status.code(),
        Some(1),
        "a versão recusada não pode ser substituída: stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let erro = json(&saida);
    assert_eq!(erro["status_http"], 403, "{erro}");
    assert_eq!(erro["documento"], DOC_A, "{erro}");
    assert_eq!(erro["documentos_lidos_antes"], 0, "{erro}");
    assert!(erro.get("documentos").is_none(), "{erro}");
    assert_eq!(
        trilha
            .iter()
            .filter(|pedido| pedido.as_str() == "GET documento_versao_listar")
            .count(),
        1,
        "a lista deve ser publicada antes da recusa: {trilha:?}"
    );
    assert_eq!(
        trilha
            .iter()
            .filter(|pedido| pedido.as_str() == "GET documento_visualizar")
            .count(),
        1,
        "um GET da versão 1, sem retry/fallback para a versão 2: {trilha:?}"
    );
    assert!(
        trilha.iter().all(|pedido| {
            pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"
        }),
        "a recusa não autoriza escrita: {trilha:?}"
    );
}
