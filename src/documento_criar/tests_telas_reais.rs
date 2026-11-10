//! Telas de escolha do tipo observadas ao vivo em 10/11/2026, sem URLs
//! assinadas: `UnB` 4.0.9 e as instalações PEN, UFRJ e GO de treinamento.
use super::*;
use crate::arvore_processo::NaturezaDoDocumento;

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

#[test]
fn rotulo_com_numeracao_do_tipo_comprova_o_novo_documento() {
    // GO 5.0.3 ao vivo (10/11/2026): "Despacho 1407 (8446460)".
    let doc = |id: &str, numero: &str, rotulo: &str| DocumentoNaArvore {
        id: id.into(),
        rotulo: rotulo.into(),
        numero_sei: Some(numero.into()),
        pasta: None,
        natureza: NaturezaDoDocumento {
            icone: None,
            cancelado: None,
            motivo_do_cancelamento: None,
            dica: None,
        },
    };
    let antes = [doc("1", "8446442", "Anexo (8446442)")];
    let depois = [
        antes[0].clone(),
        doc("2", "8446460", "Despacho 1407 (8446460)"),
    ];
    assert_eq!(
        conferir_criacao(&antes, &depois, "Despacho"),
        Ok("8446460".into())
    );
    let outro = [
        antes[0].clone(),
        doc("2", "8446460", "Ofício 1407 (8446460)"),
    ];
    assert!(conferir_criacao(&antes, &outro, "Despacho").is_err());
}
