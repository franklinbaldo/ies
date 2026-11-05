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
    let mut vazias = 0usize;
    let mut cadastros_com_lupa = 0usize;
    for resposta in gravacao::respostas(&conferida.json) {
        if resposta.operacao == "grupo_unidade_cadastrar" && resposta.status == 200 {
            let html = html_da_resposta(resposta.item)
                .unwrap_or_else(|_| panic!("cadastro ilegível no passo {}", resposta.passo));
            // GO: ação da lupa no literal servido pelo cadastro. Não executar JS,
            // não publicar nem comparar o hash assinado da cassette.
            assert!(
                super::fluxo::destino(
                    &html,
                    "https://seitreinamento.go.gov.br/sei/controlador.php",
                    "https://seitreinamento.go.gov.br/sei",
                    "unidade_selecionar_envio_processo"
                )
                .is_ok(),
                "cadastro GO passo {} sem link único autorizado da lupa",
                resposta.passo
            );
            cadastros_com_lupa += 1;
        }
        if resposta.operacao != "unidade_selecionar_envio_processo" || resposta.status != 200 {
            continue;
        }
        let html = html_da_resposta(resposta.item)
            .unwrap_or_else(|_| panic!("corpo HTML inválido no passo {}", resposta.passo));
        let pagina = ler_pagina(&html).unwrap_or_else(|e| {
            panic!(
                "cassette GO passo {} metodo {}: {e:?}",
                resposta.passo, resposta.metodo
            )
        });
        assert!(pagina.paginas >= 1);
        assert!(pagina.pagina < pagina.paginas);
        assert!(pagina.unidades.len() <= pagina.total);
        if pagina.total == 0 {
            assert_eq!(pagina.paginas, 1);
            assert_eq!(pagina.unidades, Vec::new());
            vazias += 1;
        }
        if resposta.metodo == "GET" {
            get += 1;
            // O primeiro GET da cassette abre os 13.671 destinos.
            // Um GET posterior pode mostrar o resultado já filtrado.
            if get == 1 {
                assert_eq!(pagina.total, 13_671);
                assert_eq!(pagina.paginas, 274);
                assert_eq!(pagina.unidades.len(), 50);
            }
        } else if resposta.metodo == "POST" {
            post += 1;
        }
        lidas += 1;
    }
    assert_eq!((get, post, lidas), (3, 3, 6));
    assert_eq!(
        cadastros_com_lupa, 3,
        "GO deve publicar 3 cadastros com lupa"
    );
    assert!(vazias >= 1, "a cassette GO deve ter uma lista vazia");
    println!("replay GO: {lidas} telas lidas ({get} GET, {post} POST), 0 requisições reais");
}
