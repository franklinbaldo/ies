//! Controle negativo com respostas originais: o editor nativo nao pode
//! satisfazer o seletor exclusivo do ensaio sintetico sei.test.
use super::{editor_observado, unb};
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::tela_html::HtmlServido;
use std::path::{Path, PathBuf};

fn conferir_recusa_no_original(card_relativo: &str) {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join(card_relativo);
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        PathBuf::from,
    );
    let nome_zip = archive_filename(&card).expect("ZIP do card");
    let zip = ZipDoAcervo::ler(&cache.join(nome_zip)).expect("ZIP original presente");
    let captura = conferir_cassette_ate(&card, &zip, 120_000_000)
        .expect("SHA-1 ZIP, SHA-256 membro, schema e origem conferidos");
    let mut observados = 0;
    for resposta in gravacao::respostas(&captura.json) {
        if resposta.operacao != "editor_montar"
            || resposta.metodo != "GET"
            || resposta.status != 200
        {
            continue;
        }
        let pagina = html_da_resposta(resposta.interacao).expect("HTML original legivel");
        let html = HtmlServido::de_teste(&pagina);
        assert!(
            !editor_observado(&html, 1) && !editor_observado(&html, 2),
            "{}: editor_montar no passo {} nao pode ser aceito como sei.test",
            captura.instalacao,
            resposta.passo
        );
        if captura.instalacao == "unb" {
            let versao = unb::versao_do_editor(&html).expect("versão do editor clássico");
            assert!(matches!(versao.numero(), 1 | 2));
            assert!(unb::principal_publicado(&html));
        }
        observados += 1;
    }
    assert_eq!(observados, 3, "tres editores montados na captura original");
}

/// Testa fronteira de escrita em corpos originais, nao replay da CLI inteira.
/// Nao confundir com paridade IES/anti-IES ou live.
#[test]
#[ignore = "requer ZIPs originais conferidos em IES_ACERVO_CACHE"]
fn replay_editor_nativo_no_acervo() {
    conferir_recusa_no_original("cassettes/unb/documento-escritas-unb--36a904749e20.md");
    conferir_recusa_no_original("cassettes/ufrj/documento-escritas-ufrj--2241d2a8f71b.md");
    conferir_recusa_no_original("cassettes/go/documento-escritas-go--16f83179ecaa.md");
}
