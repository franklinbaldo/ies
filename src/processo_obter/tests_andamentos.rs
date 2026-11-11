//! RED/GREEN dos andamentos depois da árvore nos caminhos 2, 3 e 4: a
//! sequência exata de requisições no padrão, com `--andamentos-completos`
//! e com `--sem-andamentos`, e as falhas tipadas.
use super::tests_apoio::*;
use super::*;
use crate::arvore_processo::tests::NUP;
use crate::historico::ErroHistorico;
use crate::historico_processo::{ErroAndamentos, ErroLinkDoHistorico, TipoDoHistorico};

fn caminho_4a(arvore: String, historico: Vec<Resposta>) -> Vec<Resposta> {
    let mut respostas = vec![ok(controle(&[(ID, NUP)])), ok(trabalhar(ID)), ok(arvore)];
    respostas.extend(historico);
    respostas
}

fn pedidos_4a() -> Vec<Pedido> {
    vec![
        get(&url_controle()),
        get(&absoluta(&trabalhar_controle(ID))),
        get(&url_arvore(ID)),
    ]
}

fn tipo_total_itens(r: &ProcessoAberto) -> (TipoDoHistorico, u64, usize) {
    let a = r.andamentos.lidos().expect("andamentos lidos");
    (a.tipo, a.total_declarado, a.itens.len())
}

#[test]
fn padrao_le_a_visao_servida_com_mais_um_get() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        caminho_4a(
            arvore_com_historico(ID, NUP),
            vec![ok(historico('R', 3, 3))],
        ),
    );
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 4);
    assert_eq!(tipo_total_itens(&r), (TipoDoHistorico::Resumido, 3, 3));
    let mut esperados = pedidos_4a();
    esperados.push(get(&url_historico(ID)));
    assert_eq!(pedidos, esperados);
    let (inst, g) = (instalacao(), geracao());
    let guardado = amb
        .indice
        .link(&inst, chave(&g, "procedimento_consultar_historico", ID))
        .map(|u| u.url().to_owned());
    assert_eq!(guardado, Some(url_historico(ID)));
    let debug = format!("{r:?}");
    assert!(!debug.contains(&"f".repeat(64)), "{debug}");
}

#[test]
fn forma_unb_do_link_tambem_abre_o_historico() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        caminho_4a(
            arvore_com_historico_unb(ID, NUP),
            vec![ok(historico('R', 1, 1))],
        ),
    );
    assert_eq!(r.unwrap().requisicoes, 4);
    assert_eq!(pedidos.last(), Some(&get(&url_historico(ID))));
}

#[test]
fn andamentos_completos_custam_get_e_post_e_publicam_so_a_visao_p() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::Completos,
        caminho_4a(
            arvore_com_historico(ID, NUP),
            vec![ok(historico('R', 2, 2)), ok(historico('P', 5, 5))],
        ),
    );
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 5);
    assert_eq!(tipo_total_itens(&r), (TipoDoHistorico::Completo, 5, 5));
    let mut esperados = pedidos_4a();
    esperados.push(get(&url_historico(ID)));
    esperados.push(post(&url_post_historico(ID), &corpo_ver_completo(2)));
    assert_eq!(pedidos, esperados);
}

#[test]
fn andamentos_completos_sem_post_quando_o_get_ja_serviu_a_visao_p() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::Completos,
        caminho_4a(
            arvore_com_historico(ID, NUP),
            vec![ok(historico('P', 4, 4))],
        ),
    );
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 4);
    assert_eq!(tipo_total_itens(&r), (TipoDoHistorico::Completo, 4, 4));
    assert_eq!(pedidos.len(), 4);
}

#[test]
fn post_que_devolve_a_visao_r_e_recusa() {
    let amb = ambiente();
    let (r, _) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::Completos,
        caminho_4a(
            arvore_com_historico(ID, NUP),
            vec![ok(historico('R', 2, 2)), ok(historico('R', 2, 2))],
        ),
    );
    assert_eq!(
        r.unwrap_err(),
        super::ErroProcessoObter::Andamentos(ErroAndamentos::VisaoDoHistoricoDiferenteDaPedida)
    );
}

#[test]
fn post_que_redireciona_ao_login_e_sessao_encerrada() {
    let amb = ambiente();
    let (r, _) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::Completos,
        caminho_4a(
            arvore_com_historico(ID, NUP),
            vec![ok(historico('R', 2, 2)), r302(LOGIN)],
        ),
    );
    assert_eq!(
        r.unwrap_err(),
        super::ErroProcessoObter::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
    );
}

#[test]
fn sem_andamentos_nao_pede_o_historico() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::Omitidos,
        caminho_4a(arvore_com_historico(ID, NUP), vec![]),
    );
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 3);
    assert_eq!(r.andamentos, Andamentos::Omitidos);
    assert_eq!(r.andamentos.motivo(), Some("omitidos_por_sem_andamentos"));
    assert_eq!(pedidos, pedidos_4a());
}

#[test]
fn sem_andamentos_aceita_arvore_sem_o_link() {
    let amb = ambiente();
    let (r, _) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::Omitidos,
        caminho_4a(arvore_de(ID, NUP), vec![]),
    );
    assert_eq!(r.unwrap().andamentos, Andamentos::Omitidos);
}

#[test]
fn arvore_sem_o_link_publica_o_motivo_sem_falhar() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::Completos,
        caminho_4a(arvore_de(ID, NUP), vec![]),
    );
    let r = r.unwrap();
    assert_eq!(r.andamentos, Andamentos::LinkNaoPublicadoNaArvore);
    assert_eq!(
        r.andamentos.motivo(),
        Some("link_do_historico_nao_publicado_na_arvore")
    );
    assert_eq!(r.requisicoes, 3);
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn link_presente_mas_repetido_continua_recusa() {
    use crate::historico_processo::tests_apoio::{com_link, forma_go, link_do_historico};
    let amb = ambiente();
    let url = link_do_historico(ID);
    let arvore = com_link(&arvore_de(ID, NUP), &[forma_go(&url), forma_go(&url)]);
    let (r, _) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        caminho_4a(arvore, vec![]),
    );
    assert_eq!(
        r.unwrap_err(),
        super::ErroProcessoObter::LinkDoHistorico(
            ErroLinkDoHistorico::LinksDoHistoricoRepetidosNaArvore
        )
    );
}

#[test]
fn historico_que_responde_503_falha_com_o_erro_de_transporte_sem_retry() {
    let amb = ambiente();
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        caminho_4a(
            arvore_com_historico(ID, NUP),
            vec![(503, String::new(), None)],
        ),
    );
    let erro = r.unwrap_err();
    assert!(
        matches!(
            &erro,
            super::ErroProcessoObter::Transporte(transporte)
                if transporte.status_http().map(|s| s.status) == Some(503)
        ),
        "{erro:?}"
    );
    assert_eq!(pedidos.len(), 4);
}

#[test]
fn historico_que_responde_login_e_sessao_encerrada() {
    let amb = ambiente();
    let (r, _) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        caminho_4a(arvore_com_historico(ID, NUP), vec![r302(LOGIN)]),
    );
    assert_eq!(
        r.unwrap_err(),
        super::ErroProcessoObter::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
    );
}

#[test]
fn historico_de_outro_nup_pelo_link_da_arvore_e_recusa() {
    let amb = ambiente();
    let outro = crate::historico_processo::tests_apoio::pagina_do_historico(
        "00009.000009/2026-09",
        ID,
        'R',
        1,
        1,
    );
    let (r, _) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        caminho_4a(arvore_com_historico(ID, NUP), vec![ok(outro)]),
    );
    let erro = r.unwrap_err();
    assert_eq!(
        erro,
        super::ErroProcessoObter::Andamentos(ErroAndamentos::Historico(
            ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado
        ))
    );
}

#[test]
fn caminhos_2_e_3_somam_um_get_do_historico() {
    let amb = ambiente();
    let inst = instalacao();
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        vec![
            r302(&trabalhar_entrada(ID)),
            ok(trabalhar(ID)),
            ok(arvore_com_historico(ID, NUP)),
            ok(historico('R', 1, 1)),
        ],
    );
    assert_eq!(r.unwrap().requisicoes, 4);
    assert_eq!(pedidos.last(), Some(&get(&url_historico(ID))));

    let amb = ambiente();
    let g = geracao();
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, "protocolo_pesquisa_rapida", ""),
            &absoluta(&action_rapida()),
        )
        .unwrap();
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        vec![
            r302(&trabalhar_rapida(ID)),
            ok(trabalhar(ID)),
            ok(arvore_com_historico(ID, NUP)),
            ok(historico('R', 1, 1)),
        ],
    );
    assert_eq!(r.unwrap().requisicoes, 4);
    assert_eq!(pedidos[0], post(&absoluta(&action_rapida()), CORPO_RAPIDO));
    assert_eq!(pedidos.last(), Some(&get(&url_historico(ID))));
}
