//! RED/GREEN da linha do Controle: texto do link igual ao NUP, linha e
//! link únicos. HTML sintético na forma observada (visões R e D).
use super::controle::link_do_processo_no_controle;
use super::tests_apoio::{ID, controle, trabalhar_controle};
use crate::arvore_processo::tests::NUP;
use crate::tela_html::HtmlServido;

#[test]
fn acha_a_linha_do_nup_exato() {
    let html = HtmlServido::de_teste(&controle(&[("7", "OUTRO-1"), (ID, NUP)]));
    assert_eq!(
        link_do_processo_no_controle(&html, NUP).unwrap(),
        Some((ID.to_owned(), trabalhar_controle(ID)))
    );
}

#[test]
fn texto_parecido_nao_e_o_nup() {
    let html = HtmlServido::de_teste(&controle(&[
        (ID, &format!("{NUP}0")),
        ("11", "00001000001202601"),
    ]));
    assert_eq!(link_do_processo_no_controle(&html, NUP).unwrap(), None);
}

#[test]
fn quebra_wbr_e_entidade_amp_como_servidas() {
    let quebrado = NUP.replace('/', "<wbr>/");
    let html = HtmlServido::de_teste(&controle(&[(ID, &quebrado)]).replace('&', "&amp;"));
    assert_eq!(
        link_do_processo_no_controle(&html, NUP).unwrap(),
        Some((ID.to_owned(), trabalhar_controle(ID)))
    );
}

#[test]
fn repeticoes_sao_recusa() {
    let html = HtmlServido::de_teste(&controle(&[(ID, NUP), ("11", NUP)]));
    assert_eq!(
        link_do_processo_no_controle(&html, NUP),
        Err(super::ErroProcessoObter::ProcessoRepetidoNoControle)
    );
    let duas = HtmlServido::de_teste(&controle(&[(ID, NUP)]).replace(
        "</a></td>",
        &format!(
            "</a><a href=\"{}\">{NUP}</a></td>",
            trabalhar_controle(ID).replace("procedimento_controlar", "outra")
        ),
    ));
    assert_eq!(
        link_do_processo_no_controle(&duas, NUP),
        Err(super::ErroProcessoObter::LinksDoProcessoRepetidosNoControle)
    );
}

#[test]
fn link_de_outro_id_na_linha_nao_conta() {
    let html = HtmlServido::de_teste(&controle(&[(ID, NUP)]).replace("id=\"P10\"", "id=\"P12\""));
    assert_eq!(link_do_processo_no_controle(&html, NUP).unwrap(), None);
}
