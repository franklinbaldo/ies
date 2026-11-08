//! BDD-333-03/05: opcoes de remocao vistas pela CLI real em TLS local.
//! O estado e o contrato aqui sao sinteticos, nao comprovam SEI real.
use super::*;

#[test]
fn bdd_333_03_previa_de_remocao_nao_muta_o_processo_ou_a_anotacao() {
    let mut previa = args(false);
    previa.push("--remover-anotacoes");
    let (saidas, pedidos, depois) = executar(&[&previa], false, Some("Nota da origem"));
    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    let plano = json(&saidas[0]);
    assert_eq!(plano["confirmado"], false);
    assert_eq!(plano["alterado"], false);
    assert_eq!(plano["remover_anotacoes"], true);
    assert_eq!(plano["anotacao_antes"], true);
    assert_eq!(plano["metodo_remocao"], "checkbox_no_envio");
    assert_eq!(depois.anotacao_origem.as_deref(), Some("Nota da origem"));
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
    assert!(
        !pedidos.iter().any(|pedido| {
            pedido == "POST procedimento_enviar" || pedido == "POST anotacao_registrar"
        }),
        "a previa jamais grava: {pedidos:?}"
    );
}

#[test]
fn bdd_333_05_remocao_nao_observada_recusa_sem_post_de_escrita() {
    let mut previa = args(false);
    previa.push("--remover-anotacoes");
    let (saidas, pedidos, depois) = executar(&[&previa], false, None);
    assert_eq!(
        saidas[0].status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    assert_eq!(
        json(&saidas[0])["subcodigo"],
        "remocao_de_anotacoes_nao_oferecida"
    );
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
    assert_eq!(depois.anotacao_origem, None);
    assert!(
        !pedidos.iter().any(|pedido| {
            pedido == "POST procedimento_enviar" || pedido == "POST anotacao_registrar"
        }),
        "a opcao nao oferecida deve ser recusada antes da escrita: {pedidos:?}"
    );
}
