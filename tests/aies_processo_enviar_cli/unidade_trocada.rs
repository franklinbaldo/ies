//! BDD-333-05: opcao de remocao nao oferecida na unidade recebente.
//! Hipotese do anti-IES; o acervo nao prova esta transicao entre sessoes.
use super::*;

#[test]
fn unidade_recebente_nao_pode_remover_anotacao_da_origem_fechada() {
    // Primeiro envio muda o estado real do SQLite: ORIGEM fecha, DESTINO abre.
    let enviar = [
        "--json", "processo", "enviar", NUP, "--para", "DESTINO", "--confirmar",
    ];
    // A sessao passa para DESTINO no mesmo banco. A anotacao pertence
    // a ORIGEM, portanto a tela de envio nao deve oferecer sua remocao.
    let remocao = [
        "--json", "processo", "enviar", NUP, "--para", "ARQUIVO",
        "--remover-anotacoes",
    ];
    let (saidas, pedidos, depois) = executar_com_troca_de_unidade(
        &[&enviar, &remocao],
        false,
        Some("Nota da origem"),
        false,
        false,
        Some("DESTINO"),
    );

    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "primeiro envio: stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr),
    );
    assert_eq!(json(&saidas[0])["depois"], serde_json::json!(["DESTINO"]));
    assert_eq!(
        saidas[1].status.code(),
        Some(1),
        "recusa da remocao: stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[1].stdout),
        String::from_utf8_lossy(&saidas[1].stderr),
    );
    assert_eq!(
        json(&saidas[1])["subcodigo"],
        "remocao_de_anotacoes_nao_oferecida",
    );
    assert_eq!(depois.aberto_em, ["DESTINO"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some("Nota da origem"));
    assert_eq!(
        pedidos
            .iter()
            .filter(|pedido| *pedido == "POST procedimento_enviar")
            .count(),
        1,
        "a recusa nao deve criar uma segunda tramitacao: {pedidos:?}",
    );
    assert!(
        !pedidos.iter().any(|pedido| pedido == "POST anotacao_registrar"),
        "sem escrita de anotacao: {pedidos:?}",
    );
    assert!(
        pedidos
            .iter()
            .any(|pedido| pedido == "POST unidade_auto_completar_envio_processo")
            || pedidos.iter().any(|pedido| pedido == "GET procedimento_enviar"),
        "a recusa deve decorrer da leitura da unidade recebente: {pedidos:?}",
    );
}
