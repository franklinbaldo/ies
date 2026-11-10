//! BDD-581-01: limpeza confirmada é efeito distinto de envio não realizado.
//! Transição hipotética sobre SQLite efêmero, não observação do SEI ao vivo.
use super::*;

#[test]
fn bdd_581_01_anotacao_removida_formulario_de_envio_invalido() {
    let mut enviar = args(true);
    enviar.retain(|arg| *arg != "--manter-aberto");
    enviar.push("--remover-anotacoes");
    let consultar = ["--json", "processo", NUP];
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&enviar, &consultar],
        false,
        Some("Anotação registrada na origem"),
        true,
        EnsaioRemocao::TelaSemCheckboxSemFormularioValido,
    );

    assert_eq!(
        saidas[0].status.code(),
        Some(1),
        "falha: stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    let falha = json(&saidas[0]);
    assert_eq!(falha["codigo"], "escrita_diferente_da_solicitada");
    assert_eq!(
        falha["anotacao_depois"], false,
        "limpeza comprovada é efeito público"
    );
    let mensagem = falha["erro"].as_str().unwrap();
    assert!(
        mensagem.contains("anotação foi removida") && mensagem.contains("processo não foi enviado"),
        "a mensagem precisa distinguir os dois efeitos: {mensagem}"
    );
    assert_eq!(falha["aberto_em"], serde_json::json!(["ORIGEM"]));

    assert_eq!(
        saidas[1].status.code(),
        Some(0),
        "releitura: stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[1].stdout),
        String::from_utf8_lossy(&saidas[1].stderr)
    );
    assert_eq!(json(&saidas[1])["aberto_em"][0]["sigla"], "ORIGEM");
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some(""));

    let limpeza = pedidos
        .iter()
        .position(|pedido| pedido == "POST anotacao_registrar")
        .expect("a CLI efetua a limpeza");
    assert!(
        pedidos[limpeza + 1..]
            .iter()
            .any(|pedido| pedido == "GET anotacao_registrar"),
        "a confirmação da limpeza depende da releitura: {pedidos:?}"
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|pedido| *pedido == "POST anotacao_registrar")
            .count(),
        1,
        "sem repetição da limpeza"
    );
    assert!(
        !pedidos
            .iter()
            .any(|pedido| pedido == "POST procedimento_enviar"),
        "o formulário inválido impede a tentativa de tramitação: {pedidos:?}"
    );
}
