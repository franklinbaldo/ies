//! RED/GREEN do passo 1b: o link `procedimento_trabalhar` colhido do
//! Controle (por qualquer comando que o leu) abre o processo com 2 GET até
//! a árvore; link velho é esquecido e o comando segue. E a colheita do GET
//! do Controle do passo 4, sem requisição a mais.
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::tests::NUP;
use crate::colheita_do_controle::Colheita;

/// O que um comando anterior (login, listagem) deixou ao ler o Controle.
fn colhido(amb: &Ambiente, linhas: &[(&str, &str)]) {
    Colheita::com_geracao(&amb.indice, &instalacao(), &geracao(), &url_controle())
        .colher(&controle(linhas), Some("text/html; charset=iso-8859-1"));
}

fn trabalho_guardado(amb: &Ambiente, id: &str) -> Option<String> {
    let (inst, g) = (instalacao(), geracao());
    amb.indice
        .link(&inst, chave(&g, AcaoDoSei::ProcedimentoTrabalhar, id))
        .map(|u| u.url().to_owned())
}

#[test]
fn processo_colhido_abre_pelo_link_de_trabalho_com_dois_get() {
    let amb = ambiente();
    colhido(&amb, &[(ID, NUP), ("11", "OUTRO-NUP")]);
    let (r, pedidos) = rodar(&amb, None, vec![ok(trabalhar(ID)), ok(arvore_de(ID, NUP))]);
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, ResolvidoPor::IndiceTrabalhar);
    assert_eq!(r.resolvido_por.como_texto(), "indice_trabalhar");
    assert_eq!(r.requisicoes, 2);
    assert_eq!(r.unidade.as_deref(), Some(UNIDADE));
    assert_eq!(
        pedidos,
        [
            get(&absoluta(&trabalhar_controle(ID))),
            get(&url_arvore(ID))
        ]
    );
    // A árvore aprendida faz a repetição seguinte custar 1 (passo 1).
    let (r, pedidos) = rodar(&amb, None, vec![ok(arvore_de(ID, NUP))]);
    assert_eq!(r.unwrap().resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!(pedidos, [get(&url_arvore(ID))]);
}

#[test]
fn com_andamentos_o_passo_1b_soma_o_get_do_historico() {
    let amb = ambiente();
    colhido(&amb, &[(ID, NUP)]);
    let (r, pedidos) = rodar_com(
        &amb,
        None,
        PedidoDeAndamentos::VisaoServida,
        vec![
            ok(trabalhar(ID)),
            ok(arvore_com_historico(ID, NUP)),
            ok(historico('R', 2, 2)),
        ],
    );
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, ResolvidoPor::IndiceTrabalhar);
    assert_eq!(r.requisicoes, 3);
    assert!(r.andamentos.lidos().is_some());
    assert_eq!(
        pedidos,
        [
            get(&absoluta(&trabalhar_controle(ID))),
            get(&url_arvore(ID)),
            get(&url_historico(ID)),
        ]
    );
}

#[test]
fn passo_1b_roda_com_unidade_conferida() {
    let amb = ambiente();
    colhido(&amb, &[(ID, NUP)]);
    let (r, pedidos) = rodar(
        &amb,
        Some(UNIDADE),
        vec![ok(trabalhar(ID)), ok(arvore_de(ID, NUP))],
    );
    assert_eq!(r.unwrap().resolvido_por, ResolvidoPor::IndiceTrabalhar);
    assert_eq!(pedidos.len(), 2);
}

#[test]
fn link_de_trabalho_que_responde_login_e_esquecido_e_a_entrada_segue() {
    let amb = ambiente();
    colhido(&amb, &[(ID, NUP)]);
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            r302(LOGIN),
            r302(&trabalhar_entrada(ID)),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, ResolvidoPor::IndiceEntrada);
    assert_eq!(r.requisicoes, 4);
    assert_eq!(pedidos[0], get(&absoluta(&trabalhar_controle(ID))));
    assert_eq!(pedidos[1], get(&url_entrada(ID)));
    // O esquecido foi substituído pelo que a entrada por id confirmou.
    assert_eq!(trabalho_guardado(&amb, ID), Some(trabalhar_entrada(ID)));
}

#[test]
fn link_de_trabalho_com_tela_de_outro_processo_e_esquecido() {
    let amb = ambiente();
    colhido(&amb, &[(ID, NUP)]);
    amb.indice.registrar_entrada_por_id(&instalacao(), false);
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            ok(trabalhar("11")),
            r302(&trabalhar_rapida(ID)),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    let r = r.unwrap();
    // A pesquisa rápida colhida do mesmo Controle é o passo seguinte.
    assert_eq!(r.resolvido_por, ResolvidoPor::PesquisaRapida);
    assert_eq!(r.requisicoes, 4);
    assert_eq!(pedidos[1], post(&absoluta(&action_rapida()), CORPO_RAPIDO));
    assert_eq!(
        trabalho_guardado(&amb, ID),
        Some(absoluta(&trabalhar_rapida(ID)))
    );
}

#[test]
fn link_de_trabalho_com_arvore_de_outro_nup_esquece_link_e_processo() {
    let amb = ambiente();
    colhido(&amb, &[(ID, NUP)]);
    amb.indice.registrar_entrada_por_id(&instalacao(), false);
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            ok(trabalhar(ID)),
            ok(arvore_de(ID, "OUTRO-NUP")),
            r302(&trabalhar_rapida(ID)),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, ResolvidoPor::PesquisaRapida);
    assert_eq!(r.requisicoes, 5);
    assert_eq!(pedidos[2], post(&absoluta(&action_rapida()), CORPO_RAPIDO));
    assert_eq!(
        amb.indice.id_por_nup(&instalacao(), NUP).as_deref(),
        Some(ID)
    );
}

#[test]
fn controle_do_passo_4_e_colhido_sem_requisicao_a_mais() {
    let amb = ambiente();
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP), ("11", "OUTRO-NUP")])),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    assert_eq!(r.unwrap().requisicoes, 3);
    assert_eq!(pedidos.len(), 3);
    // A outra linha da página também foi colhida.
    assert_eq!(
        amb.indice.id_por_nup(&instalacao(), "OUTRO-NUP").as_deref(),
        Some("11")
    );
    assert_eq!(
        trabalho_guardado(&amb, "11"),
        Some(absoluta(&trabalhar_controle("11")))
    );
}

/// BDD-633-02: a modal servida no link colhido é recusa de acesso,
/// não um palpite de índice envelhecido a ser repetido por outra rota.
/// Hipótese de transporte sintética baseada no relato GO, não cassette da modal.
#[test]
fn bdd_633_02_link_colhido_com_modal_recusa_sem_tentar_outro_caminho() {
    let amb = ambiente();
    colhido(&amb, &[(ID, NUP)]);
    let modal = crate::arvore_processo::tests_tela::tela_do_processo(
        UNIDADE,
        &[String::new()],
    )
    .replace(
        "</body>",
        "<script>function inicializar(){infraAbrirJanelaModal('controlador.php?acao=usuario_validar_acesso&acao_origem=procedimento_trabalhar&id_procedimento=10&infra_hash=SINTETICO',500,300,true,'finalizar');}</script></body>",
    );
    let (resultado, pedidos) = rodar(&amb, None, vec![ok(modal)]);
    assert!(matches!(
        resultado,
        Err(ErroProcessoObter::Tela(
            crate::arvore_processo::ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso
        ))
    ));
    assert_eq!(pedidos, [get(&absoluta(&trabalhar_controle(ID)))]);
    assert!(trabalho_guardado(&amb, ID).is_some());
}
