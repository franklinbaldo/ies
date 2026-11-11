//! RED/GREEN dos anexados no fluxo: a árvore com `PROCESSO_ANEXADO`
//! (forma da `UnB` 4.0.9, sintética) custa as mesmas requisições, e o
//! índice aprende o par NUP ↔ id do anexado sem requisição a mais.
use super::tests_apoio::*;
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
use crate::arvore_processo::tests_anexados::{NUP_ANEXADO, no_anexado};

const ID_ANEXADO: &str = "20";

fn arvore_com_anexado(id: &str, nup: &str) -> String {
    arvore(&[
        no_processo(0, id, nup),
        no_anexado(1, ID_ANEXADO, id, NUP_ANEXADO),
        no_documento(2, "21", id, "0000101"),
    ])
}

fn conferir_anexado(r: &ProcessoAberto) {
    assert_eq!(r.arvore.processo.id, ID);
    assert_eq!(r.arvore.anexados.len(), 1);
    assert_eq!(r.arvore.anexados[0].id, ID_ANEXADO);
    assert_eq!(r.arvore.anexados[0].nup, NUP_ANEXADO);
    assert_eq!(r.arvore.documentos.len(), 1);
}

#[test]
fn caminho_4a_com_anexado_custa_o_mesmo_e_aprende_o_anexado() {
    let amb = ambiente();
    let (r, pedidos) = rodar(
        &amb,
        None,
        vec![
            ok(controle(&[(ID, NUP)])),
            ok(trabalhar(ID)),
            ok(arvore_com_anexado(ID, NUP)),
        ],
    );
    let r = r.unwrap();
    conferir_anexado(&r);
    assert_eq!(r.requisicoes, 3);
    assert_eq!(
        pedidos,
        [
            get(&url_controle()),
            get(&absoluta(&trabalhar_controle(ID))),
            get(&url_arvore(ID)),
        ]
    );
    let inst = instalacao();
    assert_eq!(amb.indice.id_por_nup(&inst, NUP).as_deref(), Some(ID));
    assert_eq!(
        amb.indice.id_por_nup(&inst, NUP_ANEXADO).as_deref(),
        Some(ID_ANEXADO)
    );
}

#[test]
fn caminho_1_com_anexado_numa_requisicao() {
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
    let (r, pedidos) = rodar(&amb, None, vec![ok(arvore_com_anexado(ID, NUP))]);
    let r = r.unwrap();
    conferir_anexado(&r);
    assert_eq!(r.resolvido_por, ResolvidoPor::IndiceLink);
    assert_eq!(pedidos, [get(&url_arvore(ID))]);
    assert_eq!(
        amb.indice.id_por_nup(&inst, NUP_ANEXADO).as_deref(),
        Some(ID_ANEXADO)
    );
    // O anexado não ganha link: nenhum endereço dele foi aberto.
    assert!(
        amb.indice
            .link(
                &inst,
                chave(&g, AcaoDoSei::ProcedimentoVisualizar, ID_ANEXADO)
            )
            .is_none()
    );
}
