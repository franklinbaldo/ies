//! Replay de telas reais da lupa de unidades da cassette GO (#244).
//! Teste ignorado: exige ZIP público conferido em `IES_ACERVO_CACHE`.
//! Nunca executa HTTP nem revela cookie, URL assinada ou HTML.
use super::ler_pagina;
use crate::acao_do_sei::AcaoDoSei;
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
            let html = html_da_resposta(resposta.interacao)
                .unwrap_or_else(|_| panic!("cadastro ilegível no passo {}", resposta.passo));
            // GO: ação da lupa no literal servido pelo cadastro. Não executar JS,
            // não publicar nem comparar o hash assinado da cassette.
            let candidatos: Vec<_> =
                super::fluxo::candidatos(&html, AcaoDoSei::UnidadeSelecionarEnvioProcesso)
                    .expect("HTML do cadastro fora da forma observada")
                    .into_iter()
                    .filter(|url| url.contains("acao=unidade_selecionar_envio_processo"))
                    .collect();
            assert_eq!(
                candidatos.len(),
                1,
                "cadastro GO passo {} precisa de um literal único da lupa",
                resposta.passo
            );
            let link = &candidatos[0];
            assert!(
                super::fluxo::destino(
                    &html,
                    "https://seitreinamento.go.gov.br/sei/controlador.php",
                    "https://seitreinamento.go.gov.br/sei",
                    AcaoDoSei::UnidadeSelecionarEnvioProcesso
                )
                .is_ok(),
                "cadastro GO passo {} sem link autorizado: relativo={} duplicacao={} fragmento={} escape={}",
                resposta.passo,
                link.starts_with("controlador.php?"),
                link.contains(".."),
                link.contains('#'),
                link.contains('\\'),
            );
            cadastros_com_lupa += 1;
        }
        if resposta.operacao != "unidade_selecionar_envio_processo" || resposta.status != 200 {
            continue;
        }
        let html = html_da_resposta(resposta.interacao)
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

#[test]
#[ignore = "exige ZIP integral GO em IES_ACERVO_CACHE; executado no CI"]
fn replay_274_paginas_go_no_acervo() {
    // Caso Todos test_lupa_unidades_010.py @323eb01: paginas reais,
    // nao simuladas; orcamento gravado 4 GET + 274 POST, 0 HTTP aqui.
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join("cassettes/go/unidade-destinos-no-indice-go--85f61d670bfc.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || {
            Path::new(&std::env::var_os("HOME").expect("HOME ou IES_ACERVO_CACHE"))
                .join(".cache/ies")
        },
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("lote-5b6af062ae69d8c7.zip"))
        .expect("ZIP publico integral GO ausente");
    let conferida = conferir_cassette_ate(&card, &zip, 120_000_000)
        .expect("cassette integral GO nao confere hashes ou origem");
    assert_eq!(conferida.instalacao, "go");
    let mut paginas = Vec::new();
    let (mut get, mut post) = (0usize, 0usize);
    for resposta in gravacao::respostas(&conferida.json) {
        if resposta.operacao != "unidade_selecionar_envio_processo" {
            continue;
        }
        assert_eq!(resposta.status, 200, "passo {}", resposta.passo);
        let html = html_da_resposta(resposta.interacao)
            .unwrap_or_else(|_| panic!("HTML ilegivel no passo {}", resposta.passo));
        let lida = ler_pagina(&html)
            .unwrap_or_else(|e| panic!("pagina no passo {}: {e:?}", resposta.passo));
        if resposta.metodo == "GET" {
            get += 1;
            assert_eq!((lida.total, lida.paginas), (13_671, 274));
        } else {
            assert_eq!(resposta.metodo, "POST");
            super::conferir_filtros_de_lista_integral(&html)
                .unwrap_or_else(|e| panic!("filtros no passo {}: {e:?}", resposta.passo));
            assert_eq!(lida.pagina, post);
            post += 1;
            paginas.push(lida);
        }
    }
    assert_eq!((get, post), (1, 274));
    let reunidas = super::reunir_paginas(paginas)
        .expect("274 paginas reais devem formar snapshot sem identidades duplicadas");
    assert_eq!(reunidas.len(), 13_671);
    println!("replay integral GO: 274 paginas e 13.671 ids, 0 HTTP real");
}

#[path = "acervo/unb.rs"]
mod unb;
