//! Recusa de XML alterado no transporte local, com a CLI real e SQLite compartilhado.
//! Hipótese sintética de atributo repetido; não é observação do SEI.

use super::*;

#[test]
fn autocomplete_com_id_repetido_preserva_causa_e_nao_envia_processo() {
    let previa = args(false);
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&previa],
        false,
        None,
        false,
        EnsaioRemocao::AutocompleteIdRepetido,
    );
    falha(&saidas[0], "Consulta");
    let stderr = String::from_utf8_lossy(&saidas[0].stderr);
    assert!(
        stderr.contains("Xml(Atributo(Duplicated(12, 5)))"),
        "causa precisa no stderr: {stderr}"
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST unidade_auto_completar_envio_processo")
            .count(),
        1,
        "a consulta foi feita uma vez: {pedidos:?}"
    );
    assert!(
        pedidos.iter().all(|p| p != "POST procedimento_enviar"),
        "nenhuma escrita autorizada após XML ambíguo: {pedidos:?}"
    );
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
}
