use super::*;
use crate::tela_html::HtmlServido;

fn resumida() -> String {
    "<html><body><a id='lnkInfraUnidade'>PPI</a>\
     <input id='hdnTipoVisualizacao' value='R'>\
     <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
     <table id='tblProcessosGerados'><caption>0 registros:</caption></table></body></html>"
        .to_owned()
}

/// Forma observada na PEN1 e em GO: a visão `D` lembrada pela sessão,
/// inclusive numa página que não é a primeira.
fn detalhada(inicio: usize) -> String {
    format!(
        "<html><body><a id='lnkInfraUnidade'>PPI</a>\
         <input id='hdnTipoVisualizacao' value='D'>\
         <input id='hdnRecebidosHashCriterios' value='r'>\
         <input id='hdnGeradosHashCriterios' value='g'>\
         <table id='tblProcessosDetalhado'><caption>(150 registros - {inicio} a {inicio}):</caption>\
         <tr><th></th><th>Processo</th><th>Tipo</th></tr>\
         <tr id='P7'><td>linha</td><td><a class='processoVisualizado' \
         href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=7'>N-7</a></td>\
         <td>Ofício</td></tr></table></body></html>"
    )
}

#[test]
fn reconhece_a_unidade_na_visao_resumida() {
    assert_eq!(
        unidade_do_controle(&HtmlServido::de_teste(&resumida())).unwrap(),
        "PPI"
    );
}

#[test]
fn reconhece_a_unidade_na_visao_detalhada() {
    assert_eq!(
        unidade_do_controle(&HtmlServido::de_teste(&detalhada(1))).unwrap(),
        "PPI"
    );
}

#[test]
fn reconhece_a_visao_detalhada_lembrada_fora_da_primeira_pagina() {
    assert_eq!(
        unidade_do_controle(&HtmlServido::de_teste(&detalhada(101))).unwrap(),
        "PPI"
    );
}

#[test]
fn detalhada_sem_unidade_publicada_continua_recusada() {
    let sem_unidade =
        HtmlServido::de_teste(&detalhada(1).replace("<a id='lnkInfraUnidade'>PPI</a>", ""));
    assert!(unidade_do_controle(&sem_unidade).is_err());
}

#[test]
fn pagina_sem_controle_continua_recusada() {
    assert!(
        unidade_do_controle(&HtmlServido::de_teste(
            "<html><body><p>outra tela</p></body></html>"
        ))
        .is_err()
    );
}

#[test]
fn metamorfismo_estrutural_do_controle_resumido_sem_mudar_unidade() {
    // Variações determinísticas de atributos cosmeticos, não de contrato.
    // Oráculo: a unidade segue sendo a exibida no Controle completo.
    for mascara in 0..64_u8 {
        let mut html = resumida();
        if mascara & 1 != 0 {
            html = html.replace("<html>", "<html lang='pt-BR'>");
        }
        if mascara & 2 != 0 {
            html = html.replace(
                "<a id='lnkInfraUnidade'>",
                "<a class='titulo' id='lnkInfraUnidade'>",
            );
        }
        if mascara & 4 != 0 {
            html = html.replace(
                "<table id='tblProcessosRecebidos'>",
                "<table data-ensaio='1' id='tblProcessosRecebidos'>",
            );
        }
        if mascara & 8 != 0 {
            html = html.replace(
                "<table id='tblProcessosGerados'>",
                "<table id='tblProcessosGerados' data-ensaio='2'>",
            );
        }
        if mascara & 16 != 0 {
            html = html.replace("<body>", "<body data-ensaio='sintetico'>");
        }
        if mascara & 32 != 0 {
            html = html.replace("</body>", "\n</body>");
        }
        assert_eq!(
            unidade_do_controle(&HtmlServido::de_teste(&html)).unwrap(),
            "PPI",
            "mascara={mascara}"
        );
    }
}

#[test]
fn ablar_componentes_obrigatorios_nunca_gera_sucesso() {
    let html = resumida();
    let sem_gerados = HtmlServido::de_teste(&html.replace(
        "<table id='tblProcessosGerados'>",
        "<div id='tblProcessosGerados'>",
    ));
    assert!(unidade_do_controle(&sem_gerados).is_err());

    let sem_unidade = HtmlServido::de_teste(&html.replace("<a id='lnkInfraUnidade'>PPI</a>", ""));
    assert!(unidade_do_controle(&sem_unidade).is_err());

    let so_marcador = HtmlServido::de_teste(
        "<form id='frmLogin'><input name='txtUsuario'></form>\
                      <a id='lnkInfraUnidade'>PPI</a>",
    );
    assert!(unidade_do_controle(&so_marcador).is_err());
}

#[test]
fn metamorfismo_da_visao_detalhada_nao_altera_unidade() {
    for inicio in [1, 2, 101] {
        let html = detalhada(inicio);
        let cosmetico = HtmlServido::de_teste(&html.replace(
            "<a id='lnkInfraUnidade'>",
            "<a data-ensaio='1' id='lnkInfraUnidade'>",
        ));
        assert_eq!(unidade_do_controle(&cosmetico).unwrap(), "PPI");
    }
}
