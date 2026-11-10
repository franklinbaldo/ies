//! Confirmação BDD-348-02: CLI real, um POST, releitura independente e mensagem humana.
use super::*;

#[test]
fn bdd_348_02_confirmar_retira_so_g_com_um_post_e_segunda_cli() {
    let (saida, eventos, depois, segunda_cli) = executar_com(EstadoInicial::GrupoG, true);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} eventos={eventos:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["operacao"], "deixar");
    assert_eq!(valor["grupo"], "G");
    assert_eq!(valor["id_acompanhamento"], "831");
    assert_eq!(valor["confirmado"], true);
    assert_eq!(valor["alterado"], true);
    assert_eq!(
        eventos
            .iter()
            .filter(|evento| evento.as_str() == "POST acompanhamento_excluir")
            .count(),
        1,
        "a retirada nunca pode repetir o POST: {eventos:?}"
    );
    let restante = depois.unwrap().itens;
    assert_eq!(restante.len(), 1);
    assert_eq!(restante[0].id, "832");
    assert_eq!(restante[0].grupo, "H");
    assert_eq!(restante[0].observacao, "Preservar");
    let segunda = segunda_cli.unwrap();
    assert_eq!(
        segunda.status.code(),
        Some(0),
        "stderr segunda CLI={}",
        String::from_utf8_lossy(&segunda.stderr)
    );
    let relida: serde_json::Value = serde_json::from_slice(&segunda.stdout).unwrap();
    assert_eq!(relida["ja_estava"], true);
    assert_eq!(relida["alterado"], false);
    assert!(relida["id_acompanhamento"].is_null());
}

#[test]
fn bdd_348_02_releitura_503_apos_post_nao_reenvia_e_retorna_saida_1() {
    let (saida, eventos, depois, segunda_cli) =
        executar_com_falha(EstadoInicial::GrupoG, true, true);
    falha(&saida, "NaoConfirmado");
    assert_eq!(
        eventos
            .iter()
            .filter(|v| v.as_str() == "POST acompanhamento_excluir")
            .count(),
        1,
        "não se reenvia exclusão com resultado incerto: {eventos:?}"
    );
    let itens = depois.unwrap().itens;
    assert_eq!(itens.len(), 1);
    assert_eq!(itens[0].id, "832");
    assert_eq!(itens[0].grupo, "H");
    let outra = segunda_cli.unwrap();
    assert_eq!(outra.status.code(), Some(0));
    let relida: serde_json::Value = serde_json::from_slice(&outra.stdout).unwrap();
    assert_eq!(relida["ja_estava"], true);
    assert_eq!(relida["alterado"], false);
}

#[test]
fn bdd_348_02_saida_humana_retirada_confirmada_nao_anuncia_previa() {
    let (saida, eventos, depois, _) =
        executar_com_falha_modo(EstadoInicial::GrupoG, true, false, true);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} eventos={eventos:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let humano = String::from_utf8(saida.stdout).unwrap();
    assert!(
        humano.contains("Retirado acompanhamento do grupo: G"),
        "{humano}"
    );
    assert!(
        humano.contains("Acompanhamento retirado e confirmado por leitura."),
        "{humano}"
    );
    assert!(!humano.contains("Retiraria o acompanhamento"), "{humano}");
    assert_eq!(
        eventos
            .iter()
            .filter(|evento| evento.as_str() == "POST acompanhamento_excluir")
            .count(),
        1
    );
    let restante = depois.unwrap().itens;
    assert_eq!(restante.len(), 1);
    assert_eq!(restante[0].grupo, "H");
}
