//! BDD-ANOT-05 (#638): anotação registrada pela CLI e retirada no envio.
//! Prova sintética de sequência de comandos sobre o mesmo SQLite, não live.
use super::*;

#[test]
fn bdd_anot_05_anotacao_preexistente_desaparece_apos_envio() {
    // A anotação não é injetada como estado final: a primeira CLI a grava.
    let registrar = [
        "--json",
        "processo",
        "anotar",
        NUP,
        "--texto",
        "Nota anterior à tramitação",
        "--confirmar",
    ];
    let consultar = ["--json", "processo", "anotar", NUP];
    let mut enviar = args(true);
    enviar.push("--remover-anotacoes");

    // Quatro comandos reais, uma sessão e um banco derivado descartável:
    // registrar -> observar -> enviar removendo -> observar de novo.
    let comandos: &[&[&str]] = &[&registrar, &consultar, &enviar, &consultar];
    let (saidas, pedidos, depois) =
        executar_com_ensaio_local(comandos, false, Some(""), true, EnsaioRemocao::Normal);

    for (passo, saida) in saidas.iter().enumerate() {
        assert_eq!(
            saida.status.code(),
            Some(0),
            "passo {passo}: stdout={} stderr={} pedidos={pedidos:?}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr),
        );
    }

    let cadastro = json(&saidas[0]);
    assert_eq!(cadastro["confirmado"], true);
    assert_eq!(cadastro["alterado"], true);
    assert_eq!(cadastro["texto"], "Nota anterior à tramitação");

    let antes = json(&saidas[1]);
    assert_eq!(antes["anotacao"], "Nota anterior à tramitação");
    assert_eq!(antes["alterado"], false);

    let resultado = json(&saidas[2]);
    assert_eq!(resultado["confirmado"], true);
    assert_eq!(resultado["alterado"], true);
    assert_eq!(resultado["remover_anotacoes"], true);
    assert_eq!(resultado["anotacao_antes"], true);
    assert_eq!(resultado["anotacao_depois"], false);
    assert_eq!(
        resultado["depois"],
        serde_json::json!(["ORIGEM", "DESTINO"])
    );

    let releitura = json(&saidas[3]);
    assert_eq!(releitura["anotacao"], serde_json::Value::Null);
    assert_eq!(releitura["alterado"], false);

    // A expectativa é independente do JSON e do renderizador do anti-IES.
    assert_eq!(depois.anotacao_origem.as_deref(), Some(""));
    assert_eq!(depois.aberto_em, ["ORIGEM", "DESTINO"]);

    let registro = pedidos
        .iter()
        .position(|p| p == "POST anotacao_registrar")
        .expect("a primeira CLI deve registrar a anotação");
    let tramitacao = pedidos
        .iter()
        .position(|p| p == "POST procedimento_enviar")
        .expect("a terceira CLI deve tramitar");
    assert!(registro < tramitacao, "ordem causal: {pedidos:?}");
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST anotacao_registrar")
            .count(),
        1,
        "o envio via checkbox não deve salvar anotação separadamente"
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        1,
        "nenhum envio automático de repetição"
    );
    assert!(
        pedidos[tramitacao + 1..]
            .iter()
            .filter(|p| *p == "GET anotacao_registrar")
            .count()
            >= 2,
        "envio e consulta final devem reler a anotação: {pedidos:?}"
    );
}
