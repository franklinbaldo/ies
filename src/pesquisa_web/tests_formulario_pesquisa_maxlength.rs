//! `maxlength` do termo servido pela tela de pesquisa.
use super::*;

#[test]
fn termo_acima_do_maxlength_servido_e_recusado_antes_do_post() {
    let html = com_campo_do_termo(CAMPO_COM_50);
    let erro = regra_do_termo(&html, &"1".repeat(51));
    assert_eq!(
        erro,
        ErroRegraDoFormularioDePesquisa::TermoAcimaDoMaxlengthDaTela { maxlength: 50 }
    );
}

#[test]
fn maxlength_e_o_publicado_e_nao_um_valor_universal() {
    let html = com_campo_do_termo(&CAMPO_COM_50.replace("'50'", "'10'"));
    assert!(derivar_termo(&html, &"1".repeat(10)).is_ok());
    assert_eq!(
        regra_do_termo(&html, &"1".repeat(11)),
        ErroRegraDoFormularioDePesquisa::TermoAcimaDoMaxlengthDaTela { maxlength: 10 }
    );
}

#[test]
fn sem_maxlength_publicado_o_termo_nao_tem_limite_da_tela() {
    assert!(derivar_termo(&observado(), &"1".repeat(51)).is_ok());
}

#[test]
fn maxlength_conta_unidades_utf16_como_o_navegador() {
    let html = com_campo_do_termo(&CAMPO_COM_50.replace("'50'", "'2'"));
    assert!(derivar_termo(&html, "éé").is_ok());
    assert_eq!(
        regra_do_termo(&html, "😀a"),
        ErroRegraDoFormularioDePesquisa::TermoAcimaDoMaxlengthDaTela { maxlength: 2 }
    );
}

#[test]
fn maxlength_ilegivel_ou_campo_repetido_e_recusado() {
    for ruim in ["''", "'-1'", "'5a'", "' 5'"] {
        let html = com_campo_do_termo(&CAMPO_COM_50.replace("'50'", ruim));
        assert_eq!(
            regra_do_termo(&html, "1"),
            ErroRegraDoFormularioDePesquisa::MaxlengthDoTermoIlegivelNaTela,
            "maxlength={ruim}"
        );
    }
    let html = com_campo_do_termo(&format!("{CAMPO_COM_50}{CAMPO_COM_50}"));
    assert_eq!(
        regra_do_termo(&html, "1"),
        ErroRegraDoFormularioDePesquisa::CamposDoTermoRepetidosNaTela
    );
}
