//! BDD-60-05: processo fora da unidade da sessão não oferece histórico.
//! A cassette UnB registra a barra sem lista após conclusão; o ensaio
//! de unidade diferente é uma hipótese funcional alimentada por SQLite.
use super::*;

#[test]
fn bdd_60_05_cli_recusa_versao_sem_link_na_unidade_mas_le_conteudo_atual() {
    let (saida, trilha) = apoio::executar_sem_lista_na_unidade(&[
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
        "histórico não oferecido não pode virar uma versão: stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let erro = json(&saida);
    assert_eq!(
        erro["subcodigo"], "lista_de_versoes_nao_publicada_na_arvore",
        "{erro}"
    );
    assert_eq!(erro["documento"], DOC_A);
    assert!(erro.get("documentos").is_none(), "{erro}");
    assert!(
        trilha.iter().any(|pedido| pedido == "GET procedimento_visualizar"),
        "a recusa precisa decorrer da árvore lida: {trilha:?}"
    );
    assert!(
        !trilha.iter().any(|pedido| {
            pedido == "GET documento_versao_listar" || pedido == "GET documento_visualizar"
        }),
        "sem href publicado, nenhum GET da lista ou de versão: {trilha:?}"
    );
    assert!(
        trilha
            .iter()
            .all(|pedido| pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"),
        "sem escrita ou retry mutador: {trilha:?}"
    );

    // Fechar a unidade não cancela o conteúdo atual de um documento.
    let (atual, trilha_atual) = apoio::executar_sem_lista_na_unidade(&[
        "--json",
        "documento",
        DOC_A,
        "--processo",
        NUP_A,
        "--sem-cache",
    ]);
    assert_eq!(
        atual.status.code(),
        Some(0),
        "a recusa da lista não revoga o corpo atual: stderr={} trilha={trilha_atual:?}",
        String::from_utf8_lossy(&atual.stderr)
    );
    let lido = json(&atual);
    assert_eq!(lido["documentos"][0]["numero"], DOC_A);
    assert_eq!(lido["documentos"][0]["processo"]["nup"], NUP_A);
    assert_eq!(
        trilha_atual
            .iter()
            .filter(|pedido| pedido.as_str() == "GET documento_visualizar")
            .count(),
        1,
        "conteúdo presente e uma única leitura: {trilha_atual:?}"
    );
    assert!(
        !trilha_atual.iter().any(|p| p == "GET documento_versao_listar"),
        "leitura atual não solicita lista: {trilha_atual:?}"
    );
}
