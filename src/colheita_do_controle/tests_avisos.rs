//! #786: a recusa do índice na colheita (caminho de produção) não para o
//! comando e fica nos avisos, em vez de ser descartada.
use super::tests::{
    MIME, href_da_linha, indice, instalacao, link, links_do_menu, pagina_r, processo, trabalho,
};
use super::*;
use crate::indice_local::FalhaDoIndice;
use crate::tela_html::HtmlServido;

#[test]
fn nup_invalido_na_linha_vira_aviso_e_nao_grava_o_par() {
    let (_pasta, indice) = indice();
    // NUP opaco acima do limite do índice (128 bytes): a linha é conferida,
    // o link grava e o par NUP ↔ id é recusado.
    let longo = "N".repeat(129);
    let html = HtmlServido::de_teste(&pagina_r(
        &[
            ("10", "NUP-A", Some(href_da_linha("10", 'c'))),
            ("11", longo.as_str(), Some(href_da_linha("11", 'd'))),
        ],
        &[],
    ));
    Colheita::do_cookie(
        &indice,
        &instalacao(),
        super::tests::COOKIE,
        &super::tests::url_controle(),
    )
    .colher(&html, Some(MIME));

    let (processos, links) = indice.conteudo();
    assert_eq!(processos, [processo("NUP-A", "10")]);
    let [rapida, pesquisa] = links_do_menu();
    let acao = AcaoDoSei::ProcedimentoTrabalhar.como_na_url();
    assert_eq!(
        links,
        [
            link(acao, "10", &trabalho("10", 'c')),
            link(acao, "11", &trabalho("11", 'd')),
            rapida,
            pesquisa,
        ]
    );
    let avisos = indice.avisos();
    assert!(
        matches!(
            avisos.as_slice(),
            [FalhaDoIndice::RegistroRecusado(
                crate::indice_local::RegistroRecusado::NupInvalido
            )]
        ),
        "{avisos:?}"
    );
    assert!(indice.ligado());
}
