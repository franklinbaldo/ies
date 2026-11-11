//! RED/GREEN do vínculo de anexação no fluxo: abrir o próprio anexado
//! (árvore sintética na forma da `UnB` 4.0.9) custa o mesmo, publica o pai
//! e ensina ao índice o par NUP ↔ id e o link `procedimento_trabalhar`
//! dele; `ies processo <NUP do pai>` sai então pelo passo 1b.
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::tests::NUP;
use crate::arvore_processo::tests_anexado_a::{
    ID_PAI, NUP_PAI, arvore_do_anexado, href_do_pai, html_do_anexado,
};

fn arvore_anexada() -> String {
    arvore_do_anexado(ID, NUP, &html_do_anexado(&href_do_pai(ID_PAI, ID), NUP_PAI))
}

#[test]
fn abrir_o_anexado_publica_o_pai_e_aprende_sem_requisicao() {
    let amb = ambiente();
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_anexada()),
        ],
    );
    let r = r.unwrap();
    let vinculo = r.arvore.anexado_a.as_ref().unwrap();
    assert_eq!(vinculo.numero, NUP_PAI);
    assert_eq!(vinculo.id_procedimento, ID_PAI);
    assert_eq!(r.requisicoes, 3);
    assert_eq!(
        pedidos,
        [
            get(&url_controle()),
            get(&absoluta(&trabalhar_controle(ID))),
            get(&url_arvore(ID)),
        ]
    );
    assert!(!format!("{r:?}").contains(&"4".repeat(64)));
    let (inst, g) = (instalacao(), geracao());
    assert_eq!(amb.indice.id_por_nup(&inst, NUP).as_deref(), Some(ID));
    assert_eq!(
        amb.indice.id_por_nup(&inst, NUP_PAI).as_deref(),
        Some(ID_PAI)
    );
    assert_eq!(
        amb.indice
            .link(&inst, chave(&g, AcaoDoSei::ProcedimentoTrabalhar, ID_PAI))
            .map(|u| u.url().to_owned()),
        Some(absoluta(&href_do_pai(ID_PAI, ID)))
    );
}

#[test]
fn depois_o_pai_abre_pelo_passo_1b_em_duas_requisicoes() {
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_anexada()),
        ],
    );
    r.unwrap();
    let (r, pedidos) = rodar_nup(
        &amb,
        NUP_PAI,
        None,
        PedidoDeAndamentos::Omitidos,
        vec![ok(trabalhar(ID_PAI)), ok(arvore_de(ID_PAI, NUP_PAI))],
    );
    let r = r.unwrap();
    assert_eq!(r.resolvido_por, ResolvidoPor::IndiceTrabalhar);
    assert_eq!(r.requisicoes, 2);
    assert_eq!(r.arvore.processo.nup, NUP_PAI);
    assert!(r.arvore.anexado_a.is_none());
    assert_eq!(
        pedidos,
        [
            get(&absoluta(&href_do_pai(ID_PAI, ID))),
            get(&url_arvore(ID_PAI)),
        ]
    );
}

#[test]
fn processo_sem_vinculo_nao_aprende_pai() {
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
    assert!(r.unwrap().arvore.anexado_a.is_none());
    assert!(amb.indice.id_por_nup(&instalacao(), NUP_PAI).is_none());
}

#[test]
fn aberto_em_do_anexado_e_nao_lido_pelo_vinculo() {
    use crate::arvore_processo::{AbertoEm, MotivoAbertoEm};
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_anexada()),
        ],
    );
    assert_eq!(
        r.unwrap().arvore.aberto_em,
        AbertoEm::NaoLido(MotivoAbertoEm::RaizPublicaOVinculoDeAnexacao)
    );
}
