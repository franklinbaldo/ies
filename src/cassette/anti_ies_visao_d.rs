//! Inspeciona apenas formas estruturais de cassettes públicas conferidas.
//! Não imprime conteúdo HTML, urls, cookies ou parâmetros de sessão.
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use std::path::{Path, PathBuf};

#[test]
#[ignore = "wip #354: localizar resposta D real no ZIP público UnB do acervo"]
fn localizar_controle_d_unb_no_acervo() {
    let pasta = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes/unb");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("todos-sei-cassettes-unb.zip")).unwrap();
    let mut cards = 0usize;
    let mut paginas = 0usize;
    let mut detalhadas = 0usize;
    for entrada in std::fs::read_dir(pasta).unwrap() {
        let card = entrada.unwrap().path();
        if card.extension().is_none_or(|x| x != "md") {
            continue;
        }
        let texto = std::fs::read_to_string(&card).unwrap();
        if !texto.contains("- \`procedimento_controlar\`")
            || archive_filename(&card).ok().as_deref() != Some("todos-sei-cassettes-unb.zip")
        {
            continue;
        }
        cards += 1;
        let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
        for item in captura.json["interactions"].as_array().unwrap() {
            if item["request"]["operation"] != "procedimento_controlar"
                || item["response"]["status_code"] != 200
            {
                continue;
            }
            let Ok(html) = html_da_resposta(item) else {
                continue;
            };
            paginas += 1;
            if crate::controle_reconhecido::publica_visao_detalhada(&html) {
                detalhadas += 1;
                let lido = crate::caixa_detalhada::ler_paginas(&[html], None);
                println!(
                    "D capturada em card={} metodo={} parse_ok={}",
                    card.file_name().unwrap().to_string_lossy(),
                    item["request"]["method"].as_str().unwrap_or("desconhecido"),
                    lido.is_ok()
                );
            }
        }
    }
    println!("RESUMO_UNB cards={cards} paginas_controle={paginas} paginas_D={detalhadas}");
    assert!(cards > 0 && paginas > 0, "acervo UnB precisa ter GET Controle");
}
