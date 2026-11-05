//! Replay de telas reais da lupa de unidades da cassette GO (#244).
//! Teste ignorado: exige ZIP público conferido em `IES_ACERVO_CACHE`.
//! Nunca executa HTTP nem revela cookie, URL assinada ou HTML.
use super::ler_pagina;
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use std::path::Path;

#[test]
#[ignore = "exige lote da cassette GO em IES_ACERVO_CACHE; sem rede"]
fn replay_da_lupa_go_no_acervo() {
    // Caso de Todos tests/test_lupa_unidades_010.py @323eb01,
    // cassette unidade-listar-destinos-go (SEI 5.0.3).
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join("cassettes/go/unidade-listar-destinos-go--965da599b3e4.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || {
            Path::new(&std::env::var_os("HOME").expect("HOME ou IES_ACERVO_CACHE"))
                .join(".cache/ies")
        },
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("lote-e01c45d573ff1591.zip"))
        .expect("cassette GO ausente; baixar ZIP público e conferir SHA-1");
    let conferida = conferir_cassette_ate(&card, &zip, 120_000_000)
        .expect("cassette GO não confere hashes, schema ou origem");
    assert_eq!(conferida.instalacao, "go");

    let mut lidas = 0usize;
    let mut get = 0usize;
    let mut post = 0usize;
    for resposta in gravacao::respostas(&conferida.json) {
        if resposta.operacao != "unidade_selecionar_envio_processo" || resposta.status != 200 {
            continue;
        }
        let html = html_da_resposta(resposta.item)
            .unwrap_or_else(|_| panic!("corpo HTML inválido no passo {}", resposta.passo));
        let pagina = ler_pagina(&html).unwrap_or_else(|e| {
            let dom = tl::parse(&html, tl::ParserOptions::default()).expect("HTML da cassette");
            let legends: Vec<_> = dom
                .query_selector("caption")
                .into_iter()
                .flatten()
                .filter_map(|h| h.get(dom.parser()).and_then(tl::Node::as_tag))
                .map(|tag| super::texto_da_tag(tag, dom.parser()))
                .collect();
            panic!("lupa GO passo {}: {e:?}; legendas públicas: {legends:?}", resposta.passo)
        });
        assert!(pagina.paginas >= 1);
        assert!(pagina.pagina < pagina.paginas);
        assert!(pagina.unidades.len() <= pagina.total);
        if resposta.metodo == "GET" {
            get += 1;
            assert_eq!(pagina.total, 13_671);
            assert_eq!(pagina.paginas, 274);
            assert_eq!(pagina.unidades.len(), 50);
        } else if resposta.metodo == "POST" {
            post += 1;
        }
        lidas += 1;
    }
    assert_eq!(get, 1, "GO deve gravar o GET inicial da lupa");
    assert!(post >= 3, "GO deve gravar páginas filtradas em POST");
    assert!(lidas >= 4);
    println!("replay GO: {lidas} telas lidas ({get} GET, {post} POST), 0 requisições reais");
}
