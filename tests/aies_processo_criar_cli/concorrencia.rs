//! Perda de resposta e concorrência no E2E sintético de criação.
use super::*;

#[test]
fn resposta_perdida_preserva_residuo_sem_confirmar_nem_repetir() {
    let (saida, _, pedidos, contagens, tipos) = executar_com_releitura(
        &[
            "--json",
            "processo",
            "criar",
            "--tipo",
            TIPO,
            "--especificacao",
            ESPECIFICACAO_PERDIDA,
            "--confirmar",
        ],
        false,
        true,
        false,
    );
    assert_eq!(
        saida.status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    assert_eq!(contagens, (0, 1), "a escrita chegou ao SQLite");
    assert_eq!(tipos, 1);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.starts_with("POST procedimento_gerar "))
            .count(),
        1,
        "a perda da resposta não permite retry",
    );
    let depois = pedidos
        .iter()
        .skip_while(|p| *p != "POST procedimento_gerar 302")
        .skip(1)
        .collect::<Vec<_>>();
    assert!(
        depois
            .iter()
            .any(|p| *p == "GET procedimento_controlar 200")
    );
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(erro.contains("PosEscrita"), "{erro}");
    assert!(erro.contains("causa_post: Transporte(Rede)"), "{erro}");
    assert!(
        erro.contains("correspondentes: Some([\"00000.000001/2026-01\"])"),
        "{erro}"
    );
    // O processo recém-criado precisa aparecer em Gerados com link servido.
    // A leitura das propriedades não pode transformar coincidência em autoria.
    assert!(depois.iter().any(|p| *p == "GET procedimento_alterar 200"));
}

#[test]
fn resposta_perdida_com_dois_candidatos_nao_atribui_autoria() {
    let (saida, _, pedidos, contagens, tipos) = executar_com_releitura(
        &[
            "--json",
            "processo",
            "criar",
            "--tipo",
            TIPO,
            "--especificacao",
            ESPECIFICACAO_PERDIDA,
            "--confirmar",
        ],
        false,
        true,
        true,
    );
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert_eq!(saida.status.code(), Some(1), "{erro} {pedidos:?}");
    assert_eq!(contagens, (0, 2));
    assert_eq!(tipos, 1);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.starts_with("POST procedimento_gerar "))
            .count(),
        1
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "GET procedimento_alterar 200")
            .count(),
        2,
        "cada candidato precisa de propriedades relidas"
    );
    assert!(erro.contains("PosEscrita"), "{erro}");
    assert!(erro.contains("causa_post: Transporte(Rede)"), "{erro}");
    assert!(
        erro.contains(
            "correspondentes: Some([\"00000.000001/2026-01\", \"00000.000002/2026-01\"])"
        ),
        "{erro}"
    );
}
