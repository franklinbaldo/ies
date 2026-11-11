//! RED/GREEN do documento movido no fluxo: a árvore sintética (forma da
//! PEN1 5.0.4) custa o mesmo que outra, publica o movido fora dos
//! documentos e ensina ao índice o par NUP ↔ id e o link
//! `procedimento_trabalhar` assinado do destino, sem requisição a mais.
use super::tests_apoio::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::tests::NUP;
use crate::arvore_processo::tests_movidos::{
    ID_DESTINO, NUP_DESTINO, arvore_com_movido, href_do_destino, html_do_movido,
};

#[test]
fn abrir_processo_com_movido_aprende_o_destino_sem_requisicao() {
    let amb = ambiente();
    let arvore = arvore_com_movido(
        ID,
        NUP,
        &html_do_movido(&href_do_destino(ID_DESTINO, "31"), NUP_DESTINO),
    );
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![ok(controle(&[(ID, NUP)])), ok(trabalhar(ID)), ok(arvore)],
    );
    let r = r.unwrap();
    assert_eq!(r.requisicoes, 3);
    assert_eq!(
        pedidos,
        [
            get(&url_controle()),
            get(&absoluta(&trabalhar_controle(ID))),
            get(&url_arvore(ID)),
        ]
    );
    assert_eq!(r.arvore.documentos.len(), 2);
    let movido = &r.arvore.documentos_movidos[0];
    assert_eq!(movido.id, "31-41");
    assert_eq!(movido.destino.as_ref().unwrap().numero, NUP_DESTINO);
    let (inst, g) = (instalacao(), geracao());
    assert_eq!(
        amb.indice.id_por_nup(&inst, NUP_DESTINO).as_deref(),
        Some(ID_DESTINO)
    );
    assert_eq!(
        amb.indice
            .link(
                &inst,
                chave(&g, AcaoDoSei::ProcedimentoTrabalhar, ID_DESTINO)
            )
            .map(|u| u.url().to_owned()),
        Some(absoluta(&href_do_destino(ID_DESTINO, "31")))
    );
}

#[test]
fn movido_sem_link_nao_ensina_nada() {
    let amb = ambiente();
    let (r, _) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_com_movido(ID, NUP, "")),
        ],
    );
    assert!(r.unwrap().arvore.documentos_movidos[0].destino.is_none());
    assert!(amb.indice.id_por_nup(&instalacao(), NUP_DESTINO).is_none());
}
