//! Telas de escolha do tipo observadas ao vivo em 10/11/2026, sem URLs
//! assinadas: `UnB` 4.0.9 e as instalações PEN, UFRJ e GO de treinamento.
use super::*;

const EXIBIR: &str = "function exibirSeries(tipo) {
        document.getElementById('hdnFiltroSerie').value = tipo;
        document.getElementById('frmDocumentoEscolherTipo').submit();
    }";

#[test]
fn botao_de_utilizados_indica_lista_ja_completa() {
    // Filtro da sessão já em "todos": o botão passa a oferecer
    // exibirSeries('U') ("Exibir apenas os tipos já utilizados pela unidade").
    let tela = format!(
        r#"<html><head><script>{EXIBIR}</script></head><body>
    <form id="frmDocumentoEscolherTipo" method="post"><input type="hidden" id="hdnFiltroSerie" name="hdnFiltroSerie" value=""></form>
    <a id="ancExibirSeries" href="javascript:void(0);" onclick="exibirSeries('U');">Utilizados</a>
    <table id="tblSeries"><tr><td><a href="controlador.php?acao=documento_gerar">Despacho</a></td></tr></table></body></html>"#
    );
    assert_eq!(filtro_de_todos(&tela), Ok(false));
}

#[test]
fn sei5_escolher_id_serie_e_nome_com_nbsp() {
    // PEN, UFRJ e GO: `function escolher(idSerie)` e tipos de formulário
    // publicados como "Nome&nbsp;(Formulário)".
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let html = r##"<html><head><script>
function escolher(idSerie){
  document.getElementById('hdnIdSerie').value = idSerie;
  document.getElementById('frmDocumentoEscolherTipo').submit();
}
</script></head><body>
<form id="frmDocumentoEscolherTipo" method="post"><input type="hidden" id="hdnIdSerie" name="hdnIdSerie" value=""></form>
<table id="tblSeries"><tr><td>
<a href="#" onclick="escolher(5)" class="ancoraOpcao">Despacho</a>
<a href="#" onclick="escolher(7)" class="ancoraOpcao">Termo de Efetivo Exercicio&nbsp;(Formulário)</a>
</td></tr></table></body></html>"##;
    assert_eq!(
        tipos_oferecidos(html).unwrap(),
        ["Despacho", "Termo de Efetivo Exercicio\u{a0}(Formulário)"]
    );
    assert_eq!(
        escolha_do_tipo(&instalacao, html, "1", "Despacho"),
        Ok(EscolhaDoDocumento::Formulario("5".into()))
    );
}
