//! O termo da pesquisa de protocolo contra o `maxlength` servido.
use super::tests::observado;
use super::*;
use crate::formulario_web::tests::{instalacao, tela_iso, url_servida};
use ErroRegraDoFormularioDePesquisa as R;

/// Troca o campo do termo por `campo` (atributo `maxlength` variado).
fn com_campo_do_termo(campo: &str) -> String {
    let servido =
        "<input type='text' id='txtProtocoloPesquisa' name='txtProtocoloPesquisa' value=''>";
    let html = observado();
    assert!(html.contains(servido), "fixture sem o campo do termo");
    html.replace(servido, campo)
}

fn derivar_termo(html: &str, termo: &str) -> Result<SubmissaoDerivada, ErroPesquisaProtocolo> {
    let url = url_servida();
    derivar_pesquisa_de_protocolo(&instalacao(), &tela_iso(&url, html), "UNIDADE TESTE", termo)
}

fn regra_do_termo(html: &str, termo: &str) -> R {
    match derivar_termo(html, termo).unwrap_err() {
        ErroPesquisaProtocolo::RegraDoFormulario(erro) => erro,
        outro => panic!("esperada regra do formulário, veio {outro:?}"),
    }
}

const CAMPO_COM_50: &str = "<input type='text' id='txtProtocoloPesquisa' \
    name='txtProtocoloPesquisa' class='infraText' maxlength='50' value=''>";

#[test]
fn termo_no_limite_do_maxlength_servido_e_enviado() {
    let html = com_campo_do_termo(CAMPO_COM_50);
    assert!(derivar_termo(&html, &"1".repeat(50)).is_ok());
}

#[test]
fn termo_acima_do_maxlength_servido_e_recusado_antes_do_post() {
    let html = com_campo_do_termo(CAMPO_COM_50);
    let erro = regra_do_termo(&html, &"1".repeat(51));
    assert_eq!(erro, R::TermoAcimaDoMaxlengthDaTela { maxlength: 50 });
    assert_eq!(
        crate::erro_publico::folha(&erro),
        "termo_acima_do_maxlength_da_tela"
    );
}

#[test]
fn maxlength_e_o_publicado_e_nao_um_valor_universal() {
    let html = com_campo_do_termo(&CAMPO_COM_50.replace("'50'", "'10'"));
    assert!(derivar_termo(&html, &"1".repeat(10)).is_ok());
    assert_eq!(
        regra_do_termo(&html, &"1".repeat(11)),
        R::TermoAcimaDoMaxlengthDaTela { maxlength: 10 }
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
        R::TermoAcimaDoMaxlengthDaTela { maxlength: 2 }
    );
}

#[test]
fn maxlength_ilegivel_ou_campo_repetido_e_recusado() {
    for ruim in ["''", "'-1'", "'5a'", "' 5'"] {
        let html = com_campo_do_termo(&CAMPO_COM_50.replace("'50'", ruim));
        assert_eq!(
            regra_do_termo(&html, "1"),
            R::MaxlengthDoTermoIlegivelNaTela,
            "maxlength={ruim}"
        );
    }
    let html = com_campo_do_termo(&format!("{CAMPO_COM_50}{CAMPO_COM_50}"));
    assert_eq!(
        regra_do_termo(&html, "1"),
        R::Html(crate::erro_html::ErroHtml::repetido(
            "campos_do_termo_repetidos_na_tela",
            "o elemento aparece mais de uma vez"
        ))
    );
    assert_eq!(
        crate::erro_publico::folha(&R::MaxlengthDoTermoIlegivelNaTela),
        "maxlength_do_termo_ilegivel_na_tela"
    );
    assert_eq!(
        crate::erro_publico::folha(&R::Html(crate::erro_html::ErroHtml::repetido(
            "campos_do_termo_repetidos_na_tela",
            "o elemento aparece mais de uma vez"
        ))),
        "campos_do_termo_repetidos_na_tela"
    );
}

#[test]
fn filtro_de_tipo_exato_e_periodo_usam_mesmo_post() {
    let html = observado()
        .replace("<option value='null'>-</option></select>",
                 "<option value='null'>-</option><option value='73'>Administrativo</option></select>")
        .replace("<input type='hidden' id='hdnInicio'",
                 "<input type='text' id='txtDataInicio' name='txtDataInicio' value=''><input type='text' id='txtDataFim' name='txtDataFim' value=''><input type='hidden' id='hdnInicio'");
    let url = url_servida();
    let filtros = FiltrosDaPesquisa {
        tipo: Some("Administrativo"),
        de: Some("01/01/2026"),
        ate: Some("31/12/2026"),
        ..Default::default()
    };
    let envio = derivar_pesquisa_em_filtrada(
        PesquisarEm::Processos,
        &instalacao(),
        &tela_iso(&url, &html),
        ("UNIDADE TESTE", "0001"),
        filtros,
    )
    .unwrap();
    let corpo = String::from_utf8(envio.corpo().to_vec()).unwrap();
    assert!(corpo.contains("selTipoProcedimentoPesquisa=73"), "{corpo}");
    assert!(corpo.contains("txtDataInicio=01%2F01%2F2026"), "{corpo}");
    assert!(corpo.contains("txtDataFim=31%2F12%2F2026"), "{corpo}");
}
#[test]
fn filtro_ausente_na_tela_falha_antes_do_post() {
    let url = url_servida();
    let erro = derivar_pesquisa_em_filtrada(
        PesquisarEm::Processos,
        &instalacao(),
        &tela_iso(&url, &observado()),
        ("UNIDADE TESTE", "0001"),
        FiltrosDaPesquisa {
            de: Some("01/01/2026"),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(
        crate::erro_publico::folha(&erro),
        "criterio_nao_oferecido_pela_tela"
    );
}
#[test]
fn assunto_nao_vira_texto_sem_id_de_autocompletar() {
    let url = url_servida();
    let erro = derivar_pesquisa_em_filtrada(
        PesquisarEm::Processos,
        &instalacao(),
        &tela_iso(&url, &observado()),
        ("UNIDADE TESTE", "0001"),
        FiltrosDaPesquisa {
            assunto: Some("023.1"),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(
        crate::erro_publico::folha(&erro),
        "criterio_nao_oferecido_pela_tela"
    );
}
