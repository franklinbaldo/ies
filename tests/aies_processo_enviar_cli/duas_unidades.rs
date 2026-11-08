//! BDD-333-01/02: duas tramitações consecutivas pela CLI de verdade.
//! A simulação em loopback não demonstra compatibilidade com o SEI.
use super::*;

#[test]
fn envio_sucessivo_pela_cli_usa_unidade_ativa_e_mesmo_sqlite() {
    let primeiro = [
        "--json",
        "processo",
        "enviar",
        NUP,
        "--para",
        "DESTINO",
        "--confirmar",
    ];
    let segundo = [
        "--json",
        "processo",
        "enviar",
        NUP,
        "--para",
        "ARQUIVO",
        "--confirmar",
    ];
    let (saidas, pedidos, finalizado) = executar_com_troca_de_unidade(
        &[&primeiro, &segundo],
        false,
        Some("Nota da origem"),
        false,
        EnsaioRemocao::Normal,
        Some("DESTINO"),
    );
    for (indice, saida) in saidas.iter().enumerate() {
        assert_eq!(
            saida.status.code(),
            Some(0),
            "passo {indice}: stdout={} stderr={} pedidos={pedidos:?}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr),
        );
    }
    let inicial = json(&saidas[0]);
    assert_eq!(inicial["unidade"], "ORIGEM");
    assert_eq!(inicial["depois"], serde_json::json!(["DESTINO"]));
    let segundo_resultado = json(&saidas[1]);
    assert_eq!(segundo_resultado["unidade"], "DESTINO");
    assert_eq!(
        segundo_resultado["aberto_em"],
        serde_json::json!(["DESTINO"])
    );
    assert_eq!(segundo_resultado["depois"], serde_json::json!(["ARQUIVO"]));
    assert_eq!(finalizado.aberto_em, ["ARQUIVO"]);
    assert_eq!(
        finalizado.anotacao_origem.as_deref(),
        Some("Nota da origem"),
        "tramitações sucessivas sem remoção preservam a anotação da origem",
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        2,
        "exatamente um POST mutador por envio: {pedidos:?}",
    );
    assert!(
        !pedidos.iter().any(|p| p == "POST anotacao_registrar"),
        "nenhum comando pediu remoção de anotação",
    );
    assert!(
        pedidos
            .iter()
            .filter(|p| *p == "GET procedimento_visualizar")
            .count()
            >= 4,
        "cada envio precisa abrir o processo e depois conferir o efeito: {pedidos:?}",
    );
}

// Unidade recebente: transição observável por duas invocações da CLI.
#[test]
fn unidade_recebente_nao_pode_remover_anotacao_da_origem_fechada() {
    // Primeiro envio muda o estado real do SQLite: ORIGEM fecha, DESTINO abre.
    let enviar = [
        "--json",
        "processo",
        "enviar",
        NUP,
        "--para",
        "DESTINO",
        "--confirmar",
    ];
    // A sessao passa para DESTINO no mesmo banco. A anotacao pertence
    // a ORIGEM, portanto a tela de envio nao deve oferecer sua remocao.
    let remocao = [
        "--json",
        "processo",
        "enviar",
        NUP,
        "--para",
        "ARQUIVO",
        "--remover-anotacoes",
    ];
    let (saidas, pedidos, depois) = executar_com_troca_de_unidade(
        &[&enviar, &remocao],
        false,
        Some("Nota da origem"),
        false,
        EnsaioRemocao::Normal,
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
        !pedidos
            .iter()
            .any(|pedido| pedido == "POST anotacao_registrar"),
        "sem escrita de anotacao: {pedidos:?}",
    );
    assert!(
        pedidos
            .iter()
            .any(|pedido| pedido == "POST unidade_auto_completar_envio_processo")
            || pedidos
                .iter()
                .any(|pedido| pedido == "GET procedimento_enviar"),
        "a recusa deve decorrer da leitura da unidade recebente: {pedidos:?}",
    );
}
