//! Regressão do `tl` 0.7.8 (removido na #676) na derivação: um booleano sem valor
//! antes de `disabled`, `name`, `type` ou `value` não pode abrir o formulário.
use super::tests::{OPERACAO, instalacao, tela_iso, url_servida};
use super::*;
use crate::tela_html::HtmlServido;
use crate::tela_html::teste::com_booleano_antes_dos_atributos;

fn pagina(interior: &str) -> String {
    format!(
        "<html><a id='lnkInfraUnidade'>UNIDADE TESTE</a>\
         <form id='frmPesquisaProtocolo' method='post' \
         action='controlador.php?acao=protocolo_pesquisar&amp;infra_hash={}'>\
         <input type='hidden' name='hdnInfraTipoPagina' value='1'>\
         <button type='submit' name='sbmPesquisar' value='Pesquisar'>Pesquisar</button>\
         <input type='text' name='txtProtocoloPesquisa' value=''>\
         {interior}<input type='hidden' name='hdnInicio' value='0'></form></html>",
        "c".repeat(64)
    )
}

fn derivar(
    html: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    let url = url_servida();
    derivar_submissao(
        &OPERACAO,
        &instalacao(),
        &tela_iso(&url, &HtmlServido::de_teste(html)),
        Some("UNIDADE TESTE"),
        campos,
    )
}

#[test]
fn input_desabilitado_apos_booleano_nao_e_serializado() {
    for extra in [
        "<input hidden disabled type='hidden' name='hdnExtra' value='1'>",
        "<input disabled type='hidden' name='hdnExtra' value='1'>",
        "<input type='hidden' readonly disabled name='hdnExtra' value='1'>",
        "<input type='hidden' name='hdnExtra' value='1' hidden DISABLED='disabled'>",
    ] {
        let s = derivar(&pagina(extra), &[]).unwrap();
        assert!(
            !s.nomes_dos_campos().iter().any(|n| n == "hdnExtra"),
            "{extra}"
        );
    }
}

#[test]
fn select_multiple_desabilitado_com_booleanos_antes_do_name_nao_e_publicado() {
    let select = "<select multiple disabled name='rdoPesquisarEm' id='selX'>\
        <option value='P' selected>P</option></select>";
    assert_eq!(
        derivar(&pagina(select), &[("rdoPesquisarEm", "P")]).unwrap_err(),
        ErroSubmissaoFormulario::CampoNaoPublicadoPeloFormulario {
            campo: "rdoPesquisarEm".to_owned()
        }
    );
}

#[test]
fn booleano_antes_de_name_type_e_value_preserva_o_corpo() {
    let original = pagina("<input type='radio' name='rdoPesquisarEm' value='P'>");
    let campos = [("txtProtocoloPesquisa", "0001"), ("rdoPesquisarEm", "P")];
    let esperado = derivar(&original, &campos).unwrap();
    let com_booleano = derivar(&com_booleano_antes_dos_atributos(&original), &campos).unwrap();
    assert_eq!(com_booleano.corpo(), esperado.corpo());
    assert_eq!(com_booleano.url(), esperado.url());
    assert_eq!(
        String::from_utf8(esperado.corpo().to_vec()).unwrap(),
        "hdnInfraTipoPagina=1&sbmPesquisar=Pesquisar&txtProtocoloPesquisa=0001\
         &rdoPesquisarEm=P&hdnInicio=0"
    );
}

#[test]
fn select_com_multiple_antes_do_name_aceita_repeticao() {
    let select = "<select multiple name='rdoPesquisarEm'>\
        <option value='P' selected>P</option><option value='D' selected>D</option></select>";
    let s = derivar(
        &pagina(select),
        &[("rdoPesquisarEm", "P"), ("rdoPesquisarEm", "D")],
    )
    .unwrap();
    assert_eq!(
        s.nomes_dos_campos()
            .iter()
            .filter(|n| *n == "rdoPesquisarEm")
            .count(),
        2
    );
}

#[test]
fn name_repetido_num_controle_vale_o_primeiro() {
    let repetido = pagina("<input type='hidden' name='hdnExtra' NAME='hdnOutro' value='1'>");
    let unico = pagina("<input type='hidden' name='hdnExtra' value='1'>");
    assert_eq!(
        derivar(&repetido, &[]).unwrap().corpo(),
        derivar(&unico, &[]).unwrap().corpo()
    );
}
