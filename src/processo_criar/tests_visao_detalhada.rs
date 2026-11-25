//! `processo criar` com o Controle na visão detalhada (GO, 20/11/2026).
use super::tests::{
    HASH, arvore_verificadora, cadastro, lista, pedido, propriedades_verificadoras,
};
use super::*;
use crate::processo_concluir::tests_apoio::{UNIDADE, ambiente, ok, r302, rodar_com, trabalhar};

/// GO 20/11/2026: a sessão estava na visão detalhada (deixada por
/// `unidade listar-processos`). Sem as tabelas R não há fotografia dos
/// Gerados, mas ela só serve para reconciliar resposta incerta: o POST
/// segue e o `Location` com a releitura continuam a ser a prova.
#[test]
fn controle_na_visao_detalhada_cria_sem_fotografia_dos_gerados() {
    let amb = ambiente();
    // Forma observada na PEN1 e em GO (`controle_reconhecido::tests`).
    let detalhada = format!(
        "<html><body><a id='lnkInfraUnidade'>{UNIDADE}</a>\
         <input id='hdnTipoVisualizacao' value='D'>\
         <input id='hdnRecebidosHashCriterios' value='r'>\
         <input id='hdnGeradosHashCriterios' value='g'>\
         <table id='tblProcessosDetalhado'><caption>Processos (1 registros - 1 a 1):</caption>\
         <tr><th></th><th>Processo</th><th>Tipo</th></tr>\
         <tr id='P7'><td>linha</td><td><a class='processoVisualizado' \
         href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=7'>N-7</a></td>\
         <td>Ofício</td></tr></table>\
         <a href='controlador.php?acao=procedimento_escolher_tipo&infra_hash={HASH}'>Iniciar Processo</a>\
         </body></html>"
    );
    let respostas = vec![
        ok(detalhada),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento=10&infra_hash={HASH}"
        )),
        ok(trabalhar()),
        ok(arvore_verificadora()),
        ok(propriedades_verificadoras()),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    let feito = resultado.unwrap();
    assert!(feito.confirmado);
    assert_eq!(requests.len(), 7);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}
