//! BDD-581-01: limpeza confirmada é efeito distinto de envio não realizado.
//! Transição hipotética sobre SQLite efêmero, não observação do SEI ao vivo.
use super::*;

#[test]
fn bdd_581_01_anotacao_removida_formulario_de_envio_invalido() {
    let mut enviar = args(true);
    enviar.retain(|arg| *arg != "--manter-aberto");
    enviar.push("--remover-anotacoes");
    // Releitura só de aberto_em: a fixture não publica o histórico, a
    // consulta de propriedades nem a de marcadores na barra, e a leitura
    // sem esses links agora recusa (não sai mais null). Este cenário afirma
    // aberto_em e a anotação, não essas leituras.
    let consultar = [
        "--json",
        "processo",
        NUP,
        "--sem-andamentos",
        "--sem-propriedades",
        "--sem-marcadores",
    ];
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&enviar, &consultar],
        false,
        Some("Anotação registrada na origem"),
        true,
        EnsaioRemocao::TelaSemCheckboxSemFormularioValido,
    );

    falha(&saidas[0], "FormularioAposLimpezaComprovada");

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

#[test]
fn bdd_581_02_limpeza_comprovada_e_envio_recusado_nao_apagado_do_resultado() {
    let mut enviar = args(true);
    enviar.retain(|arg| *arg != "--manter-aberto");
    enviar.push("--remover-anotacoes");
    // Releitura só de aberto_em: a fixture não publica o histórico, a
    // consulta de propriedades nem a de marcadores na barra, e a leitura
    // sem esses links agora recusa (não sai mais null). Este cenário afirma
    // aberto_em e a anotação, não essas leituras.
    let consultar = [
        "--json",
        "processo",
        NUP,
        "--sem-andamentos",
        "--sem-propriedades",
        "--sem-marcadores",
    ];
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&enviar, &consultar],
        false,
        Some("Anotação registrada na origem"),
        true,
        EnsaioRemocao::TelaSemCheckboxEnvioRecusado,
    );

    falha(&saidas[0], "EnvioNaoAplicado");

    assert_eq!(saidas[1].status.code(), Some(0), "segunda CLI: {pedidos:?}");
    let consulta = json(&saidas[1]);
    assert_eq!(consulta["aberto_em"][0]["sigla"], "ORIGEM");
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some(""));

    let limpar = pedidos
        .iter()
        .position(|p| p == "POST anotacao_registrar")
        .unwrap();
    let enviar_post = pedidos
        .iter()
        .position(|p| p == "POST procedimento_enviar")
        .unwrap();
    assert!(
        pedidos[limpar + 1..enviar_post]
            .iter()
            .any(|p| p == "GET anotacao_registrar"),
        "GET deve comprovar a limpeza antes do envio: {pedidos:?}"
    );
    assert!(
        pedidos[enviar_post + 1..]
            .iter()
            .any(|p| p == "GET procedimento_visualizar"),
        "a recusa do POST exige releitura independente da situação: {pedidos:?}"
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST anotacao_registrar")
            .count(),
        1
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        1
    );
}
