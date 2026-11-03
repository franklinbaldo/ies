use super::*;

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
    assert_eq!(unidade_do_controle(&resumida()).unwrap(), "PPI");
}

#[test]
fn reconhece_a_unidade_na_visao_detalhada() {
    assert_eq!(unidade_do_controle(&detalhada(1)).unwrap(), "PPI");
}

#[test]
fn reconhece_a_visao_detalhada_lembrada_fora_da_primeira_pagina() {
    assert_eq!(unidade_do_controle(&detalhada(101)).unwrap(), "PPI");
}

#[test]
fn detalhada_sem_unidade_publicada_continua_recusada() {
    let sem_unidade = detalhada(1).replace("<a id='lnkInfraUnidade'>PPI</a>", "");
    assert!(unidade_do_controle(&sem_unidade).is_err());
}

#[test]
fn pagina_sem_controle_continua_recusada() {
    assert!(unidade_do_controle("<html><body><p>outra tela</p></body></html>").is_err());
}
