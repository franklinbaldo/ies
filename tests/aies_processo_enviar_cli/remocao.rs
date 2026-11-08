//! BDD-333-03/05: opcoes de remocao vistas pela CLI real em TLS local.
//! O estado e o contrato aqui sao sinteticos, nao comprovam SEI real.
use super::*;

#[test]
fn bdd_333_03_previa_de_remocao_nao_muta_o_processo_ou_a_anotacao() {
    let mut previa = args(false);
    previa.push("--remover-anotacoes");
    let (saidas, pedidos, depois) = executar(&[&previa], false, Some("Nota da origem"));
    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    let plano = json(&saidas[0]);
    assert_eq!(plano["confirmado"], false);
    assert_eq!(plano["alterado"], false);
    assert_eq!(plano["remover_anotacoes"], true);
    assert_eq!(plano["anotacao_antes"], true);
    assert_eq!(plano["metodo_remocao"], "checkbox_no_envio");
    assert_eq!(depois.anotacao_origem.as_deref(), Some("Nota da origem"));
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
    assert!(
        !pedidos.iter().any(|pedido| {
            pedido == "POST procedimento_enviar" || pedido == "POST anotacao_registrar"
        }),
        "a previa jamais grava: {pedidos:?}"
    );
}

#[test]
fn bdd_333_05_remocao_nao_observada_recusa_sem_post_de_escrita() {
    let mut previa = args(false);
    // Comando literal BDD-333-05: sem --manter-aberto.
    previa.retain(|arg| *arg != "--manter-aberto");
    previa.push("--remover-anotacoes");
    let (saidas, pedidos, depois) = executar(&[&previa], false, None);
    assert_eq!(
        saidas[0].status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    assert_eq!(
        json(&saidas[0])["subcodigo"],
        "remocao_de_anotacoes_nao_oferecida"
    );
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
    assert_eq!(depois.anotacao_origem, None);
    assert!(
        !pedidos.iter().any(|pedido| {
            pedido == "POST procedimento_enviar" || pedido == "POST anotacao_registrar"
        }),
        "a opcao nao oferecida deve ser recusada antes da escrita: {pedidos:?}"
    );
}

#[test]
fn bdd_333_04_confirmacao_da_cli_local_comprova_envio_e_remocao() {
    let mut confirmacao = args(true);
    confirmacao.push("--remover-anotacoes");
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&confirmacao],
        false,
        Some("Nota da origem"),
        true,
        EnsaioRemocao::Normal,
    );
    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    let resultado = json(&saidas[0]);
    assert_eq!(resultado["alterado"], true);
    assert_eq!(resultado["confirmado"], true);
    assert_eq!(resultado["anotacao_antes"], true);
    assert_eq!(resultado["anotacao_depois"], false);
    assert_eq!(
        resultado["depois"],
        serde_json::json!(["ORIGEM", "DESTINO"])
    );
    assert_eq!(depois.anotacao_origem.as_deref(), Some(""));
    assert_eq!(depois.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        1,
        "o processo só pode ser enviado uma vez"
    );
    assert!(!pedidos.iter().any(|p| p == "POST anotacao_registrar"));
    assert!(
        pedidos.iter().rposition(|p| p == "GET anotacao_registrar")
            > pedidos.iter().position(|p| p == "POST procedimento_enviar"),
        "a anotação deve ser relida depois da escrita: {pedidos:?}"
    );
}

#[test]
fn bdd_333_04_releitura_indisponivel_retorna_3_e_nao_repete_post() {
    let mut confirmacao = args(true);
    confirmacao.push("--remover-anotacoes");
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&confirmacao],
        true,
        Some("Nota da origem"),
        true,
        EnsaioRemocao::Normal,
    );
    assert_eq!(
        saidas[0].status.code(),
        Some(3),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    assert_eq!(json(&saidas[0])["codigo"], "efeito_incerto");
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        1,
        "a resposta incerta não autoriza repetir escrita"
    );
    assert_eq!(depois.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some(""));
}

#[test]
fn bdd_333_04_envio_aplicado_mas_anotacao_nao_removida_nao_declaro_sucesso() {
    // O proxy local omite somente o checkbox no POST. O anti-IES então
    // tramita no SQLite, mas mantém a anotação: efeitos independentes.
    let mut confirmacao = args(true);
    confirmacao.push("--remover-anotacoes");
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&confirmacao],
        false,
        Some("Nota da origem"),
        true,
        EnsaioRemocao::IgnorarNoPost,
    );
    assert_eq!(
        saidas[0].status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    let resultado = json(&saidas[0]);
    assert_eq!(resultado["codigo"], "escrita_diferente_da_solicitada");
    assert_eq!(resultado["subcodigo"], "anotacoes_nao_removidas_apos_envio");
    assert_eq!(
        resultado["depois"],
        serde_json::json!(["ORIGEM", "DESTINO"]),
        "a falha deve preservar a tramitação comprovada"
    );
    assert_eq!(depois.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some("Nota da origem"));
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        1
    );
    assert!(!pedidos.iter().any(|p| p == "POST anotacao_registrar"));
    assert!(
        pedidos.iter().rposition(|p| p == "GET anotacao_registrar")
            > pedidos.iter().position(|p| p == "POST procedimento_enviar"),
        "a recusa exige releitura independente: {pedidos:?}"
    );
}

#[test]
fn bdd_333_05_destino_nao_oferecido_recusa_sem_tramitacao() {
    // O destino consultado nao consta do mesmo SQLite que sustenta o
    // formulario; a recusa da CLI real nao pode alterar o processo.
    let mut previa = args(false);
    // Comando literal BDD-333-05: sem --manter-aberto.
    previa.retain(|arg| *arg != "--manter-aberto");
    previa[5] = "INEXISTENTE";
    previa.push("--remover-anotacoes");
    let (saidas, pedidos, depois) = executar(&[&previa], false, Some("Nota da origem"));
    assert_eq!(
        saidas[0].status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr),
    );
    let falha = json(&saidas[0]);
    assert_eq!(falha["subcodigo"], "destino_nao_oferecido_pelo_sei");
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some("Nota da origem"));
    assert!(
        pedidos
            .iter()
            .any(|p| p == "POST unidade_auto_completar_envio_processo"),
        "a escolha deve vir do autocomplete do servidor: {pedidos:?}",
    );
    assert!(
        !pedidos
            .iter()
            .any(|p| p == "POST procedimento_enviar" || p == "POST anotacao_registrar"),
        "destino invalido nao pode produzir escrita: {pedidos:?}",
    );
}

#[test]
fn bdd_333_05_recusa_da_remocao_nao_bloqueia_envio_basico_posterior() {
    // A primeira CLI recusa a opcao nao oferecida. Uma segunda CLI,
    // sem essa opcao, deve conseguir tramitar pelo MESMO SQLite.
    let mut recusada = args(false);
    recusada.retain(|arg| *arg != "--manter-aberto");
    recusada.push("--remover-anotacoes");
    let mut envio_basico = args(true);
    envio_basico.retain(|arg| *arg != "--manter-aberto");
    let (saidas, pedidos, depois) = executar(&[&recusada, &envio_basico], false, None);
    assert_eq!(saidas[0].status.code(), Some(1), "{pedidos:?}");
    assert_eq!(
        json(&saidas[0])["subcodigo"],
        "remocao_de_anotacoes_nao_oferecida"
    );
    assert_eq!(
        saidas[1].status.code(),
        Some(0),
        "envio basico apos recusa: stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[1].stdout),
        String::from_utf8_lossy(&saidas[1].stderr)
    );
    assert_eq!(json(&saidas[1])["depois"], serde_json::json!(["DESTINO"]));
    assert_eq!(depois.aberto_em, ["DESTINO"]);
    assert_eq!(depois.anotacao_origem, None);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        1,
        "a recusa da opcao nao pode disparar a escrita"
    );
    assert!(
        !pedidos.iter().any(|p| p == "POST anotacao_registrar"),
        "a anotacao nao foi observada: {pedidos:?}"
    );
}

#[test]
fn bdd_333_03_previa_fallback_e_confirmacao_releem_anotacao_no_mesmo_sqlite() {
    // Hipótese revisável: o SEI não oferece checkbox na tela de envio,
    // mas oferece o formulário de anotação da própria unidade.
    let mut previa = args(false);
    previa.push("--remover-anotacoes");
    let mut confirmacao = args(true);
    confirmacao.push("--remover-anotacoes");
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&previa, &confirmacao],
        false,
        Some("Nota da origem"),
        true,
        EnsaioRemocao::TelaSemCheckbox,
    );

    for (indice, saida) in saidas.iter().enumerate() {
        assert_eq!(
            saida.status.code(),
            Some(0),
            "passo {indice}: stdout={} stderr={} pedidos={pedidos:?}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        );
    }
    let plano = json(&saidas[0]);
    assert_eq!(plano["confirmado"], false);
    assert_eq!(plano["alterado"], false);
    assert_eq!(plano["remover_anotacoes"], true);
    assert_eq!(plano["anotacao_antes"], true);
    assert_eq!(plano["metodo_remocao"], "limpar_anotacao");
    assert_eq!(plano["aberto_em"], serde_json::json!(["ORIGEM"]));

    let resultado = json(&saidas[1]);
    assert_eq!(resultado["confirmado"], true);
    assert_eq!(resultado["alterado"], true);
    assert_eq!(resultado["metodo_remocao"], "limpar_anotacao");
    assert_eq!(resultado["anotacao_antes"], true);
    assert_eq!(resultado["anotacao_depois"], false);
    assert_eq!(
        resultado["depois"],
        serde_json::json!(["ORIGEM", "DESTINO"])
    );
    assert_eq!(depois.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some(""));

    let anotacao_post = pedidos
        .iter()
        .position(|p| p == "POST anotacao_registrar")
        .unwrap();
    let envio_post = pedidos
        .iter()
        .position(|p| p == "POST procedimento_enviar")
        .unwrap();
    assert!(
        pedidos[..anotacao_post]
            .iter()
            .filter(|p| *p == "GET procedimento_enviar")
            .count()
            >= 2,
        "a prévia não deve gravar: {pedidos:?}"
    );
    assert!(
        pedidos[anotacao_post + 1..envio_post]
            .iter()
            .any(|p| p == "GET anotacao_registrar"),
        "a limpeza exige releitura antes de tramitar: {pedidos:?}"
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
    assert!(
        pedidos[envio_post + 1..]
            .iter()
            .any(|p| p == "GET procedimento_visualizar"),
        "a tramitação exige leitura posterior: {pedidos:?}"
    );
}
