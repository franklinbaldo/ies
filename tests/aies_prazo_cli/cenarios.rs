//! Cenários BDD da CLI: teste real sobre o anti-IES e SQLite compartilhado.
use super::*;

#[test]
fn bdd_272_01_cli_real_le_tres_estados_sem_escrita() {
    for (caso, tipo, data) in [
        ("sem", "sem_prazo", None),
        ("data", "data", Some("20/11/2026")),
        ("concluido", "concluido", None),
    ] {
        let (saida, pedidos, estado_final) = executar(caso);
        assert_eq!(
            saida.status.code(),
            Some(0),
            "caso={caso} stdout={} stderr={} pedidos={pedidos:?}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
        assert_eq!(json["nup"], PEDIDO);
        assert_eq!(json["atual"]["tipo"], tipo, "caso={caso}");
        match data {
            Some(data) => assert_eq!(json["atual"]["data"], data),
            None => assert!(json["atual"].get("data").is_none()),
        }
        assert_eq!(json["confirmado"], false);
        assert_eq!(estado_final, (data.map(str::to_owned), caso == "concluido"));
        assert!(
            pedidos.iter().any(|p| p == "GET controle_prazo_definir"),
            "formulário de prazo deve vir do servidor: {pedidos:?}"
        );
        assert!(
            pedidos.iter().any(|p| p == "GET procedimento_visualizar"),
            "a CLI deve abrir a árvore do processo: {pedidos:?}"
        );
        assert!(
            pedidos.iter().all(|p| p.starts_with("GET ")),
            "leitura não pode modificar o simulador: {pedidos:?}"
        );
        assert_eq!(json["requisicoes"], pedidos.len());
    }
}

#[test]
fn bdd_272_02_cli_real_previsualiza_dias_sem_post_e_sem_mutacao() {
    let (saida, pedidos, estado_final) = executar_com_argumentos("sem", &["--dias", "15"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["confirmado"], false);
    assert_eq!(json["solicitado"]["dias"], 15);
    assert_eq!(json["atual"]["tipo"], "sem_prazo");
    assert_eq!(estado_final, (None, false));
    assert!(pedidos.iter().all(|p| p.starts_with("GET ")));
    assert_eq!(json["requisicoes"], pedidos.len());
}

#[test]
fn bdd_272_02_cli_real_confirma_data_por_post_unico_e_releitura() {
    let (saida, pedidos, estado_final) =
        executar_com_argumentos("sem", &["--ate", "20/11/2026", "--confirmar"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["confirmado"], true);
    assert_eq!(json["solicitado"]["ate"], "20/11/2026");
    assert_eq!(estado_final, (Some("20/11/2026".into()), false));
    let escrita = pedidos
        .iter()
        .position(|p| p == "POST controle_prazo_definir")
        .unwrap();
    assert_eq!(pedidos.iter().filter(|p| p.starts_with("POST ")).count(), 1);
    assert!(
        pedidos[escrita + 1..]
            .iter()
            .any(|p| p == "GET controle_prazo_definir"),
        "a releitura independente precisa ocorrer depois do POST: {pedidos:?}"
    );
    assert_eq!(json["requisicoes"], pedidos.len());
}

#[test]
fn bdd_272_02_cli_real_confirma_dias_por_post_unico_e_releitura() {
    let (saida, pedidos, _estado_final) =
        executar_com_argumentos("sem", &["--dias", "15", "--confirmar"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["confirmado"], true);
    assert_eq!(json["solicitado"]["dias"], 15);
    assert_eq!(json["atual"]["tipo"], "dias");
    assert_eq!(json["atual"]["dias"], 15);
    let escrita = pedidos
        .iter()
        .position(|p| p == "POST controle_prazo_definir")
        .unwrap();
    assert_eq!(pedidos.iter().filter(|p| p.starts_with("POST ")).count(), 1);
    assert!(
        pedidos[escrita + 1..]
            .iter()
            .any(|p| p == "GET controle_prazo_definir"),
        "confirmar dias exige leitura nova: {pedidos:?}"
    );
    assert_eq!(json["requisicoes"], pedidos.len());
}

#[test]
fn bdd_272_02_post_aplicado_mas_recusado_retorna_efeito_incerto_sem_retry() {
    let (saida, pedidos, estado_final) =
        executar_com_resposta("sem", &["--ate", "20/11/2026", "--confirmar"], true);
    assert_eq!(
        saida.status.code(),
        Some(3),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["codigo"], "efeito_incerto");
    assert_eq!(json["retentavel"], false);
    assert_eq!(estado_final, (Some("20/11/2026".into()), false));
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p == &"POST controle_prazo_definir")
            .count(),
        1,
        "não repetir o envio inconclusivo: {pedidos:?}"
    );
}

#[test]
fn bdd_272_03_previa_de_exclusao_nao_altera_prazo() {
    let (saida, pedidos, estado_final) = executar_com_argumentos("data", &["--excluir"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["confirmado"], false);
    assert_eq!(estado_final, (Some("20/11/2026".into()), false));
    assert!(
        pedidos.iter().any(|p| p == "GET controle_prazo_listar"),
        "exclusão nasce da lista de prazos da unidade: {pedidos:?}"
    );
    assert!(pedidos.iter().all(|p| p.starts_with("GET ")));
}

#[test]
fn bdd_272_03_exclui_pela_lista_com_post_unico_e_releitura() {
    let (saida, pedidos, estado_final) =
        executar_com_argumentos("data", &["--excluir", "--confirmar"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["confirmado"], true);
    assert_eq!(json["atual"]["tipo"], "sem_prazo");
    assert_eq!(estado_final, (None, false));
    let escrita = pedidos
        .iter()
        .position(|p| p == "POST controle_prazo_excluir")
        .expect("exclusão da lista, não definição");
    assert_eq!(pedidos.iter().filter(|p| p.starts_with("POST ")).count(), 1);
    assert!(
        pedidos[escrita + 1..]
            .iter()
            .any(|p| p == "GET controle_prazo_listar"),
        "verificação da remoção é a lista relida: {pedidos:?}"
    );
}

#[test]
fn bdd_272_03_post_exclusao_persistido_sem_aceite_retorna_3_sem_retry() {
    let (saida, pedidos, estado_final) =
        executar_com_resposta("data", &["--excluir", "--confirmar"], true);
    assert_eq!(
        saida.status.code(),
        Some(3),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["codigo"], "efeito_incerto");
    assert_eq!(json["retentavel"], false);
    assert_eq!(estado_final, (None, false));
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p == &"POST controle_prazo_excluir")
            .count(),
        1,
        "não repetir exclusão inconclusiva: {pedidos:?}"
    );
}

#[test]
fn bdd_272_02_outra_invocacao_da_cli_le_prazo_persistido() {
    for (argumentos, tipo, campo, valor) in [
        (
            &["--ate", "20/11/2026", "--confirmar"][..],
            "data",
            "data",
            serde_json::json!("20/11/2026"),
        ),
        (
            &["--dias", "15", "--confirmar"][..],
            "dias",
            "dias",
            serde_json::json!(15),
        ),
    ] {
        let comandos: [&[&str]; 2] = [argumentos, &[]];
        let (saidas, pedidos, estado_final) = executar_sequencia("sem", &comandos, false);
        assert_eq!(saidas.len(), 2);
        for saida in &saidas {
            assert_eq!(
                saida.status.code(),
                Some(0),
                "stdout={} stderr={} pedidos={pedidos:?}",
                String::from_utf8_lossy(&saida.stdout),
                String::from_utf8_lossy(&saida.stderr),
            );
        }
        let primeira: serde_json::Value = serde_json::from_slice(&saidas[0].stdout).unwrap();
        let segunda: serde_json::Value = serde_json::from_slice(&saidas[1].stdout).unwrap();
        assert_eq!(primeira["confirmado"], true);
        assert_eq!(segunda["confirmado"], false);
        assert!(segunda["solicitado"].is_null());
        assert_eq!(segunda["atual"]["tipo"], tipo);
        assert_eq!(segunda["atual"][campo], valor);
        assert_eq!(pedidos.iter().filter(|p| p.starts_with("POST ")).count(), 1);
        let escrita = pedidos
            .iter()
            .position(|p| p == "POST controle_prazo_definir")
            .unwrap();
        assert!(
            pedidos[escrita + 1..]
                .iter()
                .filter(|p| *p == "GET controle_prazo_definir")
                .count()
                >= 2,
            "confirmacao e segundo comando devem consultar o estado: {pedidos:?}"
        );
        assert_eq!(
            estado_final.0,
            (tipo == "data").then(|| "20/11/2026".to_owned())
        );
        assert!(!estado_final.1);
    }
}

#[test]
fn bdd_272_03_outra_invocacao_da_cli_observa_prazo_excluido() {
    let comandos: [&[&str]; 2] = [&["--excluir", "--confirmar"], &[]];
    let (saidas, pedidos, estado_final) = executar_sequencia("data", &comandos, false);
    assert_eq!(saidas.len(), 2);
    for saida in &saidas {
        assert_eq!(
            saida.status.code(),
            Some(0),
            "stdout={} stderr={} pedidos={pedidos:?}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr),
        );
    }
    let primeira: serde_json::Value = serde_json::from_slice(&saidas[0].stdout).unwrap();
    let segunda: serde_json::Value = serde_json::from_slice(&saidas[1].stdout).unwrap();
    assert_eq!(primeira["confirmado"], true);
    assert_eq!(segunda["confirmado"], false);
    assert_eq!(segunda["atual"]["tipo"], "sem_prazo");
    assert!(segunda["solicitado"].is_null());
    assert_eq!(estado_final, (None, false));
    assert_eq!(pedidos.iter().filter(|p| p.starts_with("POST ")).count(), 1);
    let escrita = pedidos
        .iter()
        .position(|p| p == "POST controle_prazo_excluir")
        .unwrap();
    assert!(
        pedidos[escrita + 1..]
            .iter()
            .filter(|p| *p == "GET controle_prazo_definir")
            .count()
            >= 2,
        "confirmacao e segunda CLI devem reler o formulario: {pedidos:?}"
    );
}
