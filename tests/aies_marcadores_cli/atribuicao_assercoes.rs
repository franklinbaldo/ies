//! Asserções causais do BDD-255-01, executadas pela CLI Rust real.
use super::*;

pub(super) fn conferir_atribuicao(
    saida: &Output,
    releitura: &Output,
    persistido: &Observacao,
    pedidos: &[String],
    json: &serde_json::Value,
    nao_confirmado: bool,
) {
    assert_eq!(
        saida.status.code(),
        Some(i32::from(nao_confirmado)),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    assert_eq!(releitura.status.code(), Some(0));
    let processo: serde_json::Value = serde_json::from_slice(&releitura.stdout).unwrap();
    assert_eq!(processo["marcadores_total_declarado"], 1);
    assert_eq!(processo["marcadores"][0]["rotulo"], "501#Revisão");
    assert_eq!(processo["marcadores"][0]["texto"], "Nova revisão");
    assert_eq!(persistido.total_declarado, 1);
    assert_eq!(persistido.itens[0].texto.as_deref(), Some("Nova revisão"));
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.as_str() == "POST andamento_marcador_cadastrar")
            .count(),
        1,
        "atribuição precisa de um POST, sem retry: {pedidos:?}"
    );
    if nao_confirmado {
        super::falha(saida, "AtribuicaoNaoConfirmada");
    } else {
        assert_eq!(json["acao"], "atribuir");
        assert_eq!(json["confirmado"], true);
        assert_eq!(json["alterado"], true);
        assert_eq!(json["texto"], "Nova revisão");
    }
}

/// A interface observada muda depois da atribuição, não antes da tentativa.
pub(super) fn conferir_controles(controles: &[(bool, bool, bool)], sem_tela_gerenciamento: bool) {
    if sem_tela_gerenciamento {
        assert!(
            controles.is_empty(),
            "zero marcadores usa 302 sem formulário de gerenciamento"
        );
    } else {
        assert!(!controles.is_empty(), "gerenciamento não foi servido");
        assert!(
            controles.iter().all(|vistos| *vistos == (true, true, true)),
            "GET de gerenciamento deve oferecer adicionar, remover e remoção individual"
        );
    }
}
