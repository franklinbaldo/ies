//! Regressões sintéticas: função de navegação inerte não autoriza POST (#246).
//! Oráculo positivo: Todos `tests/test_escolha_na_lista_010.py` @323eb01.
//! Os casos negativos são adversariais HTML, não capturas de treinamento.
use super::*;

const ESCOLHER: &str = r"function escolher(id) {
    document.getElementById('hdnIdSerie').value = id;
    document.getElementById('frmDocumentoEscolherTipo').submit();
}";

const EXIBIR: &str = r"function exibirSeries(tipo) {
    document.getElementById('hdnFiltroSerie').value = tipo;
    document.getElementById('frmDocumentoEscolherTipo').submit();
}";

fn script_inerte(codigo: &str, variante: usize) -> String {
    match variante {
        0 => format!("<template><script>{codigo}</script></template>"),
        1 => format!("<script type='application/json'>{codigo}</script>"),
        2 => format!("<!-- <script>{codigo}</script> -->"),
        _ => unreachable!(),
    }
}

#[test]
fn escolher_ignora_codigo_em_template_json_ou_comentario_html() {
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let tabela = r##"<table id="tblSeries"><tr><td>
        <a href="#" onclick="escolher(291)">Despacho</a>
        </td></tr></table>"##;
    for variante in 0..3 {
        let tela = format!("{}{}", script_inerte(ESCOLHER, variante), tabela);
        assert_eq!(
            escolha_do_tipo(&instalacao, &tela, "4864", "Despacho"),
            Err(ErroCriar::LinkDoTipoNaoNavegavel),
            "variante inerte {variante} não pode autorizar navegação POST"
        );
    }
    let ativa = format!("<script>{ESCOLHER}</script>{tabela}");
    assert_eq!(
        escolha_do_tipo(&instalacao, &ativa, "4864", "Despacho"),
        Ok(EscolhaDoDocumento::Formulario("291".to_owned()))
    );
}

#[test]
fn filtro_exibir_todos_ignora_script_inerte() {
    let botoes = r#"<a id="ancExibirSeries" href="javascript:void(0);"
        onclick="exibirSeries('T');">Todos</a>
        <table id="tblSeries"></table>"#;
    for variante in 0..3 {
        let tela = format!("{}{}", script_inerte(EXIBIR, variante), botoes);
        assert_eq!(
            filtro_de_todos(&tela),
            Err(ErroCriar::LinkDoTipoNaoNavegavel),
            "variante inerte {variante} não pode autorizar POST do filtro"
        );
    }
    let ativa = format!("<script>{EXIBIR}</script>{botoes}");
    assert_eq!(filtro_de_todos(&ativa), Ok(true));
}
