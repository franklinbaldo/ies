use super::{DesfechoDaInclusao, ErroInclusaoProcessual, desfecho_sem_escrita};
use crate::bloco::TipoBloco;
use crate::bloco_detalhe::ler_detalhe;
use crate::bloco_estado::EstadoDoBloco;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use crate::processo::NumeroProcesso;
use crate::tela_html::HtmlServido;
use std::path::Path;

#[test]
#[ignore = "ZIP original PEN1 SHA-verificado no IES_ACERVO_CACHE; replay offline"]
fn cassette_pen1_bloco_interno_distingue_ausencia_de_inclusao_ja_confirmada() {
    let cache = std::env::var_os("IES_ACERVO_CACHE").expect("IES_ACERVO_CACHE");
    let zip = ZipDoAcervo::ler(&Path::new(&cache).join("todos-sei-cassettes-pen1.zip")).unwrap();
    let card = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cassettes/pen1/bloco-interno-processos-pen1--69795bb274d6.md");
    let cassette = conferir_cassette_ate(&card, &zip, 20_000_000).unwrap();
    let interacoes = cassette.json["interactions"].as_array().unwrap();
    let ler = |indice: usize| {
        let interacao = &interacoes[indice];
        assert_eq!(
            interacao["request"]["operation"],
            "rel_bloco_protocolo_listar"
        );
        assert_eq!(interacao["response"]["status_code"], 200);
        let html = HtmlServido::de_teste(&html_da_resposta(interacao).unwrap());
        ler_detalhe(&html, "68", TipoBloco::Interno, EstadoDoBloco::Gerado).unwrap()
    };
    let nup = NumeroProcesso::novo("043430/2026-63").unwrap();
    let antes = ler(11);
    assert_eq!(antes.total_declarado, 0);
    assert!(matches!(
        desfecho_sem_escrita(&antes, &nup, true),
        Err(ErroInclusaoProcessual::InclusaoDeProcessoIndisponivel),
    ));
    let depois = ler(22);
    assert_eq!(depois.total_declarado, 1);
    assert!(matches!(
        desfecho_sem_escrita(&depois, &nup, true),
        Ok(DesfechoDaInclusao::JaEstava {
            confirmacao_pedida: true,
        }),
    ));
}
