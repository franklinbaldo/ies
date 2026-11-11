//! A CLI real recusa botão de anotação ilegível sobre HTTPS local e SQLite.
//! O atributo repetido é hipótese sintética, não captura de cassette.
use super::*;

#[test]
fn atributo_repetido_no_botao_recusa_antes_do_post_mutador() {
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
    let esperado = [
        "GET procedimento_controlar",
        "POST protocolo_pesquisa_rapida",
        "GET procedimento_trabalhar",
        "GET procedimento_visualizar",
        "GET procedimento_enviar",
        "GET anotacao_registrar",
    ];
    assert_eq!(
        pedidos.iter().map(String::as_str).collect::<Vec<_>>(),
        esperado,
        "consulta HTTP pode usar POST; nenhum POST mutador é autorizado"
    );
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some("Nota da origem"));
}
