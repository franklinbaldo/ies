//! A CLI real recusa botão de anotação ilegível sobre HTTPS local e SQLite.
//! O atributo repetido é hipótese sintética, não captura de cassette.
use super::*;

#[test]
fn atributo_repetido_no_botao_recusa_antes_de_qualquer_post() {
    let mut previa = args(false);
    previa.push("--remover-anotacoes");
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&previa],
        false,
        Some("Nota da origem"),
        false,
        EnsaioRemocao::BotaoAnotacaoComNomeRepetido,
    );
    falha(&saidas[0], "Anotacoes");
    let stderr = String::from_utf8_lossy(&saidas[0].stderr);
    assert!(
        stderr.contains(
            "TelaDeAnotacoesComAtributoIlegivel(AtributoRepetidoNaTag { atributo: \"name\" })"
        ),
        "causa precisa no stderr: {stderr}"
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "GET anotacao_registrar")
            .count(),
        1,
        "uma única leitura da anotação: {pedidos:?}"
    );
    assert!(
        pedidos.iter().all(|p| !p.starts_with("POST ")),
        "nenhum POST autorizado após o HTML ambíguo: {pedidos:?}"
    );
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some("Nota da origem"));
}
