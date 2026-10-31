use super::*;

#[test]
fn historia_e_escolhida_pela_operacao_e_nao_pelo_nome() {
    let json = serde_json::json!({"interactions": [
        {"request": {"operation": "procedimento_controlar", "method": "GET"}},
        {"request": {"operation": "procedimento_consultar_historico", "method": "GET"}}
    ]});
    assert_eq!(
        operation_for_replay(&json, &[]),
        Ok("procedimento_consultar_historico")
    );
    assert_eq!(
        operation_for_replay(&json, &[0]),
        Ok("procedimento_controlar")
    );
}

#[test]
fn caixa_e_padrao_quando_nao_ha_historico() {
    let json = serde_json::json!({"interactions": [
        {"request": {"operation": "procedimento_controlar", "method": "GET"}}
    ]});
    assert_eq!(
        operation_for_replay(&json, &[]),
        Ok("procedimento_controlar")
    );
}
#[test]
fn seleciona_get_e_post_de_uma_unica_sequencia_do_controle() {
    let dados = serde_json::json!({"interactions": [
        {"request":{"operation":"procedimento_controlar","method":"GET"}},
        {"request":{"operation":"procedimento_controlar","method":"POST"}}
    ]});
    assert_eq!(passos_de_todas_as_paginas(&dados), Ok(vec![0, 1]));
}

#[test]
fn recusa_intercalacao_de_outra_operacao_sem_agregar_caixas() {
    let dados = serde_json::json!({"interactions": [
        {"request":{"operation":"procedimento_controlar","method":"GET"}},
        {"request":{"operation":"documento_imprimir_web","method":"GET"}},
        {"request":{"operation":"procedimento_controlar","method":"POST"}}
    ]});
    assert_eq!(
        passos_de_todas_as_paginas(&dados),
        Err(ErroPaginasAutomaticas::PaginasDeBlocosDiferentes)
    );
}

#[test]
fn recusa_controle_sem_get_inicial() {
    let dados = serde_json::json!({"interactions": [
        {"request":{"operation":"procedimento_controlar","method":"POST"}}
    ]});
    assert_eq!(
        passos_de_todas_as_paginas(&dados),
        Err(ErroPaginasAutomaticas::PrimeiraPaginaNaoEGet)
    );
}

#[test]
fn passo_explicito_de_historico_escolhe_a_operacao_observada() {
    let json = serde_json::json!({"interactions": [
        {"request": {"operation": "procedimento_controlar", "method": "GET"}},
        {"request": {"operation": "procedimento_consultar_historico", "method": "GET"}}
    ]});
    assert_eq!(
        operation_for_replay(&json, &[1]),
        Ok("procedimento_consultar_historico")
    );
    assert_eq!(
        operation_for_replay(&json, &[0]),
        Ok("procedimento_controlar")
    );
    assert_eq!(
        operation_for_replay(&json, &[0, 1]),
        Err(ErroSelecaoOperacao::PassosComOperacoesDiferentes)
    );
}

#[test]
fn passo_inexistente_falha_com_erro_tipado() {
    let json = serde_json::json!({"interactions": [
        {"request": {"operation": "procedimento_controlar", "method": "GET"}}
    ]});
    assert_eq!(
        operation_for_replay(&json, &[7]),
        Err(ErroSelecaoOperacao::PassoNaoEncontrado { indice: 7 })
    );
}

#[test]
fn replay_nao_mistura_multiplos_historicos_nem_aceita_acao_aleatoria() {
    let json = serde_json::json!({"interactions": [
        {"request": {"operation": "procedimento_consultar_historico", "method": "GET"}},
        {"request": {"operation": "procedimento_consultar_historico", "method": "GET"}},
        {"request": {"operation": "documento_assinar", "method": "POST"}}
    ]});
    assert_eq!(
        operation_for_replay(&json, &[0, 1]),
        Err(ErroSelecaoOperacao::HistoricoComVariasRespostas)
    );
    assert_eq!(
        operation_for_replay(&json, &[2]),
        Err(ErroSelecaoOperacao::OperacaoDoPassoNaoSuportada { indice: 2 })
    );
}

#[test]
fn selecao_pesquisa_exige_post_200_sem_reinterpretar_get() {
    let corpo = base64::engine::general_purpose::STANDARD
        .encode(b"<table class='pesquisaResultado'></table>");
    let payload = serde_json::json!({"interactions":[
        {"request":{"operation":"protocolo_pesquisar","method":"GET"},
         "response":{"status_code":200,"headers":{"content-type":"text/html"},"body_base64":corpo,"encoding":"utf-8"}},
        {"request":{"operation":"protocolo_pesquisar","method":"POST"},
         "response":{"status_code":200,"headers":{"content-type":"text/html"},"body_base64":corpo,"encoding":"utf-8"}}
    ]});
    assert!(selected_html_pages(&payload, &[0], "protocolo_pesquisar", false, false).is_err());
    assert_eq!(
        selected_html_pages(&payload, &[1], "protocolo_pesquisar", false, false)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn resposta_de_pesquisa_exige_post_200_e_operacao_exata() {
    use base64::Engine as _;
    let html = "<table class='pesquisaResultado'></table>";
    let corpo = base64::engine::general_purpose::STANDARD.encode(html);
    let resposta = serde_json::json!({"interactions":[{
        "request":{"operation":"protocolo_pesquisar","method":"POST"},
        "response":{"status_code":200,"headers":{"content-type":"text/html; charset=utf-8"},"body_base64":corpo,"encoding":"utf-8"}
    }]});
    assert_eq!(
        selected_html_pages(&resposta, &[0], "protocolo_pesquisar", false, false),
        Ok(vec![html.to_owned()])
    );
    for (metodo, status, operacao) in [
        ("GET", 200, "protocolo_pesquisar"),
        ("POST", 403, "protocolo_pesquisar"),
        ("POST", 200, "procedimento_controlar"),
    ] {
        let mut alterado = resposta.clone();
        alterado["interactions"][0]["request"]["method"] = metodo.into();
        alterado["interactions"][0]["request"]["operation"] = operacao.into();
        alterado["interactions"][0]["response"]["status_code"] = status.into();
        assert!(
            selected_html_pages(&alterado, &[0], "protocolo_pesquisar", false, false).is_err(),
            "metodo={metodo}, status={status}, operacao={operacao}"
        );
    }
}
