//! RED/GREEN do caminho 1 com andamentos: o link do histórico guardado é
//! tentado primeiro (repetição quente = árvore 1 + histórico 1); guardado
//! que responde login, redirect ou outro NUP é esquecido e o link da
//! árvore recém-lida o substitui.
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::tests::NUP;
use crate::historico_processo::TipoDoHistorico;

fn quente(amb: &Ambiente, historico_guardado: bool) {
    let (inst, g) = (instalacao(), geracao());
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, AcaoDoSei::ProcedimentoVisualizar, ID),
            &url_arvore(ID),
        )
        .unwrap();
    if historico_guardado {
        amb.indice
            .registrar_link(
                &inst,
                chave(&g, AcaoDoSei::ProcedimentoConsultarHistorico, ID),
                &url_historico_guardada(ID),
            )
            .unwrap();
    }
}

fn guardado(amb: &Ambiente) -> Option<String> {
    let (inst, g) = (instalacao(), geracao());
    amb.indice
        .link(
            &inst,
            chave(&g, AcaoDoSei::ProcedimentoConsultarHistorico, ID),
        )
        .map(|u| u.url().to_owned())
}

#[test]
fn repeticao_quente_custa_arvore_e_historico_guardado() {
    let amb = ambiente();
    quente(&amb, true);
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        vec![ok(arvore_com_historico(ID, NUP)), ok(historico('R', 2, 2))],
    );
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!(r.requisicoes, 2);
    assert_eq!(
        pedidos,
        [get(&url_arvore(ID)), get(&url_historico_guardada(ID))]
    );
    assert_eq!(guardado(&amb), Some(url_historico_guardada(ID)));
}

#[test]
fn repeticao_quente_com_andamentos_completos_custa_tres() {
    let amb = ambiente();
    quente(&amb, true);
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::Completos,
        vec![
            ok(arvore_com_historico(ID, NUP)),
            ok(historico('R', 2, 2)),
            ok(historico('P', 6, 6)),
        ],
    );
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 3);
    assert_eq!(
        r.andamentos.lidos().unwrap().tipo,
        TipoDoHistorico::Completo
    );
    assert_eq!(
        pedidos,
        [
            get(&url_arvore(ID)),
            get(&url_historico_guardada(ID)),
            post(&url_post_historico(ID), &corpo_ver_completo(2)),
        ]
    );
}

#[test]
fn sem_historico_guardado_usa_o_link_da_arvore_e_o_aprende() {
    let amb = ambiente();
    quente(&amb, false);
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        vec![ok(arvore_com_historico(ID, NUP)), ok(historico('R', 1, 1))],
    );
    assert_eq!(r.unwrap().requisicoes, 2);
    assert_eq!(pedidos, [get(&url_arvore(ID)), get(&url_historico(ID))]);
    assert_eq!(guardado(&amb), Some(url_historico(ID)));
}

#[test]
fn guardado_que_responde_login_ou_redirect_e_esquecido_e_o_da_arvore_segue() {
    for resposta in [r302(LOGIN), r302(&absoluta("controlador.php?acao=outra"))] {
        let amb = ambiente();
        quente(&amb, true);
        let (r, pedidos) = rodar_com(
            &amb,
            None,
            PedidoDeAndamentos::VisaoServida,
            vec![
                ok(arvore_com_historico(ID, NUP)),
                resposta,
                ok(historico('R', 1, 1)),
            ],
        );
        assert_eq!(r.unwrap().requisicoes, 3);
        assert_eq!(
            pedidos,
            [
                get(&url_arvore(ID)),
                get(&url_historico_guardada(ID)),
                get(&url_historico(ID)),
            ]
        );
        assert_eq!(guardado(&amb), Some(url_historico(ID)));
    }
}

#[test]
fn guardado_que_serve_outro_nup_e_esquecido_e_o_da_arvore_segue() {
    let amb = ambiente();
    quente(&amb, true);
    let outro = crate::historico_processo::tests_apoio::pagina_do_historico(
        "00009.000009/2026-09",
        ID,
        'R',
        1,
        1,
    );
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        vec![
            ok(arvore_com_historico(ID, NUP)),
            ok(outro),
            ok(historico('R', 1, 1)),
        ],
    );
    assert_eq!(r.unwrap().requisicoes, 3);
    assert_eq!(pedidos[2], get(&url_historico(ID)));
    assert_eq!(guardado(&amb), Some(url_historico(ID)));
}

#[test]
fn guardado_que_falha_sem_link_na_arvore_publica_o_motivo() {
    let amb = ambiente();
    quente(&amb, true);
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        vec![ok(arvore_de(ID, NUP)), r302(LOGIN)],
    );
    let r = r.unwrap();
    assert_eq!(r.andamentos, Andamentos::LinkNaoPublicadoNaArvore);
    assert_eq!(r.requisicoes, 2);
    assert_eq!(pedidos.len(), 2);
    assert_eq!(guardado(&amb), None);
}

#[test]
fn guardado_ainda_vale_quando_a_arvore_nao_publica_o_link() {
    let amb = ambiente();
    quente(&amb, true);
    let (r, _) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        vec![ok(arvore_de(ID, NUP)), ok(historico('R', 1, 1))],
    );
    assert!(r.unwrap().andamentos.lidos().is_some());
}

#[test]
fn guardado_que_responde_503_falha_sem_tentar_o_da_arvore() {
    let amb = ambiente();
    quente(&amb, true);
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        vec![
            ok(arvore_com_historico(ID, NUP)),
            (503, String::new(), None),
        ],
    );
    assert!(matches!(
        r.unwrap_err(),
        super::ErroProcessoObter::Transporte(_)
    ));
    assert_eq!(pedidos.len(), 2);
}

#[test]
fn sem_andamentos_no_caminho_1_custa_so_a_arvore() {
    let amb = ambiente();
    quente(&amb, true);
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::Omitidos,
        vec![ok(arvore_com_historico(ID, NUP))],
    );
    assert_eq!(r.unwrap().requisicoes, 1);
    assert_eq!(pedidos, [get(&url_arvore(ID))]);
}
