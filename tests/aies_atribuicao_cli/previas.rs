//! Prévia e recusa segura da CLI real, reaproveitando o mesmo servidor local.
use super::*;

#[test]
fn bdd_264_01_cli_json_previa_atribuicao_sem_escrever() {
    let (saida, eventos, unidades_abertas, atribuicao_final) = executar(true, false);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} stdout={} eventos={eventos:?}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["nup"], NUP);
    assert_eq!(valor["unidade"], "TESTE");
    assert_eq!(valor["usuario_atual"], "aluno1");
    assert_eq!(valor["usuario_pedido"], "aluno2");
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert_eq!(valor["requisicoes"].as_u64(), Some(eventos.len() as u64));
    assert_eq!(eventos.len(), 5, "{eventos:?}");
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.as_str() == "GET procedimento_atribuicao_cadastrar")
            .count(),
        1
    );
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.starts_with("POST "))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["POST protocolo_pesquisa_rapida"],
        "nenhum POST de atribuição: {eventos:?}"
    );
    assert_eq!(
        unidades_abertas, 1,
        "bancada não altera unidades do processo"
    );
    assert_eq!(
        atribuicao_final, "aluno1",
        "prévia não grava atribuição no SQLite"
    );
}

#[test]
fn bdd_264_01_cli_humana_exibe_atribuicao_atual_e_pretendida() {
    let (saida, eventos, unidades_abertas, atribuicao_final) = executar(false, false);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} stdout={} eventos={eventos:?}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let texto = String::from_utf8(saida.stdout).unwrap();
    assert!(texto.contains("Atribuição atual: aluno1"), "{texto}");
    assert!(texto.contains("Atribuir a aluno2"), "{texto}");
    assert_eq!(eventos.len(), 5);
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.starts_with("POST "))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["POST protocolo_pesquisa_rapida"]
    );
    assert_eq!(unidades_abertas, 1);
    assert_eq!(
        atribuicao_final, "aluno1",
        "prévia não grava atribuição no SQLite"
    );
}

#[test]
fn confirmar_publico_sem_sessao_recusa_sem_rede() {
    // Sem arquivos privados, o comando falha sem acessar a rede, mesmo
    // com confirmação pedida e antes de existir qualquer requisição.
    // Arquivos privados inexistentes: qualquer tentativa de sessão falha.
    for escolha in [["--usuario", "aluno2"], ["--remover", ""]] {
        let mut argumentos = vec!["--json", "processo", "atribuir", NUP];
        if escolha[0] == "--remover" {
            argumentos.push("--remover");
        } else {
            argumentos.extend(["--usuario", "aluno2"]);
        }
        argumentos.push("--confirmar");
        let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
            .args(&argumentos)
            .env("IES_ORIGEM_FILE", "/arquivo/inexistente/ies-origem")
            .env("IES_COOKIE_FILE", "/arquivo/inexistente/ies-cookie")
            .env("IES_URL_CONTROLE_FILE", "/arquivo/inexistente/ies-url")
            .env("HOME", "/arquivo/inexistente/ies-home")
            .output()
            .unwrap();
        assert_eq!(saida.status.code(), Some(1));
        let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
        assert!(json["erro"].is_string(), "recusa deve ser pública: {json}");
        assert!(json["subcodigo"].is_string(), "recusa tipada: {json}");
    }
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "processo",
            "atribuir",
            NUP,
            "--usuario",
            "aluno2",
            "--confirmar",
        ])
        .env("IES_ORIGEM_FILE", "/arquivo/inexistente/ies-origem")
        .env("IES_COOKIE_FILE", "/arquivo/inexistente/ies-cookie")
        .env("IES_URL_CONTROLE_FILE", "/arquivo/inexistente/ies-url")
        .env("HOME", "/arquivo/inexistente/ies-home")
        .output()
        .unwrap();
    assert_eq!(saida.status.code(), Some(1));
    let erro = String::from_utf8(saida.stderr).unwrap();
    assert!(erro.contains("IES:"), "recusa humana ausente: {erro}");
}

#[test]
fn remover_previa_cli_real_json_nao_altera_anti_ies() {
    let (saida, eventos, unidades_abertas, atribuicao_final) = executar(true, true);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} stdout={} eventos={eventos:?}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["usuario_atual"], "aluno1");
    assert!(valor["usuario_pedido"].is_null());
    assert_eq!(valor["remover"], true);
    assert_eq!(valor["valor_da_opcao"], "null");
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert_eq!(valor["requisicoes"].as_u64(), Some(eventos.len() as u64));
    assert_eq!(eventos.len(), 5, "{eventos:?}");
    assert_eq!(
        eventos
            .iter()
            .filter(|e| e.starts_with("POST "))
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["POST protocolo_pesquisa_rapida"],
        "a prévia não deve remover atribuição: {eventos:?}"
    );
    assert_eq!(unidades_abertas, 1);
    assert_eq!(atribuicao_final, "aluno1");
}

#[test]
fn remover_previa_cli_real_humana_nomeia_operacao_sem_escrita() {
    let (saida, eventos, _, atribuicao_final) = executar(false, true);
    assert_eq!(saida.status.code(), Some(0));
    let texto = String::from_utf8(saida.stdout).unwrap();
    assert!(texto.contains("Atribuição atual: aluno1."), "{texto}");
    assert!(texto.contains("Remover a atribuição observada."), "{texto}");
    assert!(!texto.contains("Remoção confirmada"), "{texto}");
    assert_eq!(eventos.len(), 5);
    assert_eq!(atribuicao_final, "aluno1");
}

