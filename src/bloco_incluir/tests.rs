//! `frmBlocoEscolher` da cassette `bloco-incluir-anotar-retirar-unb`
//! (interação 24), com `infra_hash` e hash dos itens sintéticos.
use super::*;

const TELA_UNB: &str = include_str!("tela_unb.html");

#[test]
fn tela_da_unb_oferece_blocos_e_documentos_do_processo() {
    let tela = ler_tela_de_escolha(TELA_UNB).unwrap();
    assert_eq!(tela.blocos, ["2195", "2135", "2093", "2092", "1485"]);
    assert_eq!(tela.documentos, ["0011392", "0011393"]);
}

#[test]
fn sem_formulario_ou_com_selecao_repetida_e_recusa() {
    assert_eq!(
        ler_tela_de_escolha("<html><body></body></html>"),
        Err(ErroHtml::ausente("formulario_de_escolher_bloco_ausente"))
    );
    let repetida = TELA_UNB.replacen(
        "</form>",
        "<select id=\"selBloco\" name=\"selBloco\"></select></form>",
        1,
    );
    assert_eq!(
        ler_tela_de_escolha(&repetida),
        Err(ErroHtml::repetido("selecoes_de_bloco_repetidas"))
    );
}

#[test]
fn opcao_de_bloco_fora_da_forma_e_recusa() {
    let tela = TELA_UNB.replacen("value=\"2195\"", "value=\"2195a\"", 1);
    assert!(ler_tela_de_escolha(&tela).is_err());
}
