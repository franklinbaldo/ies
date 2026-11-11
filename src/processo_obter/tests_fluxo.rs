//! RED/GREEN da sequência exata de requisições de cada caminho (1, 2, 3,
//! 4a e 4b) e do aprendizado no índice local.
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::tests::NUP;

fn conferir_aberto(r: &ProcessoAberto, por: ResolvidoPor, requisicoes: u8, unidade: Option<&str>) {
    assert_eq!(r.resolvido_por, por);
    assert_eq!(r.requisicoes, requisicoes);
    assert_eq!(r.unidade.as_deref(), unidade);
    assert_eq!(r.arvore.processo.nup, NUP);
    assert_eq!(r.arvore.processo.id, ID);
    assert_eq!(r.arvore.documentos.len(), 1);
    let debug = format!("{r:?}");
    for c in ['a', 'b', 'c', 'd', 'e', '9'] {
        assert!(!debug.contains(&hash(c)), "{debug}");
    }
}

#[test]
fn caminho_4a_linha_do_controle_e_aprende_tudo() {
    let amb = ambiente();
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[("7", "OUTRO-1"), (ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    conferir_aberto(&r.unwrap(), ResolvidoPor::Caixa, 3, Some(UNIDADE));
    assert_eq!(
        pedidos,
        [
            get(&url_controle()),
            get(&absoluta(&trabalhar_controle(ID))),
            get(&url_arvore(ID)),
        ]
    );
    let (inst, g) = (instalacao(), geracao());
    assert_eq!(amb.indice.id_por_nup(&inst, NUP).as_deref(), Some(ID));
    let link = |acao, objeto| {
        amb.indice
            .link(&inst, chave(&g, acao, objeto))
            .map(|u| u.url().to_owned())
    };
    assert_eq!(
        link(AcaoDoSei::ProcedimentoVisualizar, ID),
        Some(url_arvore(ID))
    );
    assert_eq!(
        link(AcaoDoSei::ProcedimentoTrabalhar, ID),
        Some(absoluta(&trabalhar_controle(ID)))
    );
    assert_eq!(
        link(AcaoDoSei::ProtocoloPesquisaRapida, ""),
        Some(absoluta(&action_rapida()))
    );
}

#[test]
fn caminho_4b_pesquisa_rapida_do_formulario() {
    let amb = ambiente();
    let (r, pedidos) = rodar(
        &amb,
        Some(UNIDADE),
        vec![
            ok(controle(&[("7", "OUTRO-1")])),
            r302(&trabalhar_rapida(ID)),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    conferir_aberto(&r.unwrap(), ResolvidoPor::PesquisaRapida, 4, Some(UNIDADE));
    assert_eq!(
        pedidos,
        [
            get(&url_controle()),
            post(&absoluta(&action_rapida()), CORPO_RAPIDO),
            get(&absoluta(&trabalhar_rapida(ID))),
            get(&url_arvore(ID)),
        ]
    );
    assert_eq!(
        amb.indice.id_por_nup(&instalacao(), NUP).as_deref(),
        Some(ID)
    );
}

#[test]
fn caminho_3_pesquisa_rapida_guardada() {
    let amb = ambiente();
    let (inst, g) = (instalacao(), geracao());
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, AcaoDoSei::ProtocoloPesquisaRapida, ""),
            &absoluta(&action_rapida()),
        )
        .unwrap();
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            r302(&trabalhar_rapida(ID)),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    conferir_aberto(&r.unwrap(), ResolvidoPor::PesquisaRapida, 3, Some(UNIDADE));
    assert_eq!(
        pedidos,
        [
            post(&absoluta(&action_rapida()), CORPO_RAPIDO),
            get(&absoluta(&trabalhar_rapida(ID))),
            get(&url_arvore(ID)),
        ]
    );
}

#[test]
fn caminho_2_entrada_por_id_registra_o_aceite() {
    let amb = ambiente();
    let inst = instalacao();
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            r302(&trabalhar_entrada(ID)),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    conferir_aberto(&r.unwrap(), ResolvidoPor::IndiceEntrada, 3, Some(UNIDADE));
    assert_eq!(
        pedidos,
        [
            get(&url_entrada(ID)),
            get(&trabalhar_entrada(ID)),
            get(&url_arvore(ID))
        ]
    );
    assert_eq!(amb.indice.entrada_por_id(&inst), Some(true));
}

#[test]
fn caminho_1_link_da_arvore_numa_requisicao_sem_unidade() {
    let amb = ambiente();
    let (inst, g) = (instalacao(), geracao());
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, AcaoDoSei::ProcedimentoVisualizar, ID),
            &url_arvore(ID),
        )
        .unwrap();
    let (r, pedidos) = rodar(&amb, None, vec![ok(arvore_de(ID, NUP))]);
    conferir_aberto(&r.unwrap(), ResolvidoPor::IndiceLink, 1, None);
    assert_eq!(pedidos, [get(&url_arvore(ID))]);
}

#[test]
fn segunda_execucao_na_mesma_sessao_custa_uma_requisicao() {
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    assert_eq!(r.unwrap().requisicoes, 3);
    let (r, pedidos) = rodar(&amb, None, vec![ok(arvore_de(ID, NUP))]);
    assert_eq!(r.unwrap().resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!(pedidos, [get(&url_arvore(ID))]);
}

#[test]
fn com_unidade_o_link_da_arvore_e_pulado() {
    let amb = ambiente();
    let (inst, g) = (instalacao(), geracao());
    amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
    amb.indice
        .registrar_link(
            &inst,
            chave(&g, AcaoDoSei::ProcedimentoVisualizar, ID),
            &url_arvore(ID),
        )
        .unwrap();
    let (r, pedidos) = rodar(
        &amb,
        Some(UNIDADE),
        vec![
            r302(&trabalhar_entrada(ID)),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    assert_eq!(r.unwrap().resolvido_por, ResolvidoPor::IndiceEntrada);
    assert_eq!(pedidos[0], get(&url_entrada(ID)));
}

#[test]
fn indice_desligado_vai_direto_ao_controle() {
    let mut amb = ambiente();
    amb.indice = crate::indice_local::IndiceLocal::desligado();
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_de(ID, NUP)),
        ],
    );
    assert_eq!(r.unwrap().resolvido_por, ResolvidoPor::Caixa);
    assert_eq!(pedidos.len(), 3);
}
