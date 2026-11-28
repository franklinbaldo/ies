use super::*;
use crate::formulario_web::{
    derivar_submissao,
    tests::{instalacao, tela_iso},
};
use crate::propriedades_processo::tests_apoio::{
    link_das_propriedades, tela_das_propriedades, tela_restrita,
};
use crate::tela_html::HtmlServido;

fn corpo(html: &str) -> String {
    let html = HtmlServido::de_teste(html);
    let pares = pares_de_especificacao(&html, "Nova descrição & café").unwrap();
    let refs: Vec<_> = pares.iter().map(|(n, v)| (*n, v.as_str())).collect();
    let url = format!(
        "https://sei.exemplo.gov.br/sei/{}",
        link_das_propriedades(crate::acao_do_sei::AcaoDoSei::ProcedimentoAlterar, "10")
    );
    let sub = derivar_submissao(
        &ALTERAR_ESPECIFICACAO,
        &instalacao(),
        &tela_iso(&url, &html),
        None,
        &refs,
    )
    .unwrap();
    String::from_utf8(sub.corpo().to_vec()).unwrap()
}

#[test]
fn derivacao_nao_usa_projecao_normalizada_da_lupa() {
    let html = tela_das_propriedades("00001.000001/2026-01", "10");
    for (rotulo, codificado) in [
        ("Pessoa Sintetica A", "Pessoa+Sintetica+A"),
        ("Pessoa&nbsp;Sintetica A", "Pessoa%A0Sintetica+A"),
        ("Pessoa  Sintetica A", "Pessoa++Sintetica+A"),
    ] {
        let variante = html.replace("Pessoa Sintetica A", rotulo);
        let corpo = corpo(&variante);
        let esperado = format!(
            "hdnInteressadosProcedimento=301%B1{codificado}%A5302%B1Entidade+Sintetica+B"
        );
        assert!(corpo.split('&').any(|par| par == esperado));
    }
}

#[test]
fn publica_e_restrita_preservam_controles() {
    let nup = "00001.000001/2026-01";
    let publica = corpo(&tela_das_propriedades(nup, "10"));
    for par in [
        "txtDescricao=Nova+descri%E7%E3o+%26+caf%E9",
        "txaObservacoes=Observa%E7%E3o+sint%E9tica",
        "selTipoProcedimento=100000002",
        "hdnAssuntos=2001%B1000.01+-+Assunto+sint%E9tico+um",
        "hdnInteressadosProcedimento=301%B1Pessoa+Sintetica+A%A5302%B1Entidade+Sintetica+B",
        "hdnFlagProcedimentoCadastro=2",
        "rdoNivelAcesso=0",
        "selGrauSigilo=null",
        "txtAssunto=",
        "txtInteressadoProcedimento=",
        "txtInteressadoUsuario=",
    ] {
        assert!(publica.split('&').any(|p| p == par), "par ausente: {par}");
    }
    for nome in [
        "selHipoteseLegal",
        "btnSalvar",
        "selAssuntos",
        "selInteressadosProcedimento",
        "hdnInfraPrefixoCookie",
    ] {
        assert!(
            !publica
                .split('&')
                .any(|p| p.starts_with(&format!("{nome}=")))
        );
    }
    let restrita = corpo(&tela_restrita(nup, "10"));
    assert!(restrita.split('&').any(|p| p == "rdoNivelAcesso=1"));
    assert!(restrita.split('&').any(|p| p == "selHipoteseLegal=7"));
    let placeholder = tela_das_propriedades(nup, "10").replace(
        "<select id=\"selHipoteseLegal\" name=\"selHipoteseLegal\" class=\"infraSelect\">\n</select>",
        "<select id=\"selHipoteseLegal\" name=\"selHipoteseLegal\" class=\"infraSelect\"><option value=\"null\">&nbsp;</option></select>");
    assert!(
        corpo(&placeholder)
            .split('&')
            .any(|p| p == "selHipoteseLegal=null")
    );
}
