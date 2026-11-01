use std::process::Command;

fn ies(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .output()
        .expect("IES deve iniciar")
}

#[test]
fn ajuda_raiz() {
    let output = ies(&["--help"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("processo"));
}

#[test]
fn substantivo_sozinho_mostra_ajuda() {
    let output = ies(&["processo"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("enviar"));
}

#[test]
fn comando_sem_execucao_retorna_falha_estruturada() {
    let output = ies(&["documento", "77782513", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["codigo"], "nao_implementado");
}

#[test]
fn uso_invalido_falha_antes_da_execucao() {
    let output = ies(&["processo", "enviar", "123"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn escrita_reconhecida_nao_executa_sei() {
    let output = ies(&[
        "processo",
        "enviar",
        "123",
        "--para",
        "PGE-PPI",
        "--confirmar",
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["codigo"], "nao_implementado");
}

#[test]
fn leitura_de_processo_aceita_projecoes_sem_executar_sei() {
    let output = ies(&[
        "processo",
        "0064.001060/2023-08",
        "--sem-documentos",
        "--sem-marcadores",
        "--sem-andamentos",
        "--sem-propriedades",
        "--andamentos-completos",
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["codigo"], "nao_implementado");
    assert!(
        value["dados"]["comando"]
            .as_str()
            .unwrap()
            .contains("sem_documentos: true")
    );
}

#[test]
fn leitura_de_lote_de_documentos_aceita_opcoes_globais() {
    let output = ies(&[
        "documento",
        "77782513",
        "77782514",
        "--processo",
        "0064.001060/2023-08",
        "--saida",
        "conteudos",
        "--com-propriedades",
        "--ocultar-inacessiveis",
        "--unidade",
        "PPI",
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["codigo"], "nao_implementado");
    assert!(
        value["dados"]["comando"]
            .as_str()
            .unwrap()
            .contains("77782514")
    );
}

#[test]
fn unidade_listar_processos_nao_exporta_sem_sessao_autorizada() {
    let arquivo = std::env::temp_dir().join(format!(
        "ies-sem-sessao-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let output = ies(&[
        "unidade",
        "listar-processos",
        "--saida",
        arquivo.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["codigo"], "fonte_invalida");
    assert!(
        !arquivo.exists(),
        "não se cria arquivo sem leitura autorizada"
    );
}

#[test]
fn busca_documento_permanece_subcomando() {
    let output = ies(&["documento", "buscar", "--assunto", "previdencia", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["codigo"], "nao_implementado");
}

#[test]
fn replay_falha_sem_acervo_e_nao_executa_o_sei() {
    let output = ies(&[
        "replay",
        "processo-andamento-go",
        "--acervo",
        "inexistente",
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "acervo_ausente");
}

#[test]
fn processo_nao_aceita_parametros_de_replay() {
    let output = ies(&["processo", "202600005002487", "--zip-file", "x.zip"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn processo_rejeita_numero_vazio_com_erro_tipado() {
    let output = ies(&["processo", "   ", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let erro: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(erro["codigo"], "entrada_invalida");
    assert_eq!(erro["subcodigo"], "numero_processo_vazio");
}

#[test]
fn processo_rejeita_numero_com_controle_sem_vazar_entrada() {
    let output = ies(&["processo", "123\nSEGREDO", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let erro: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(erro["subcodigo"], "numero_processo_controle");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("SEGREDO"));
}

#[test]
fn processo_rejeita_controle_removivel_no_inicio_ou_fim_sem_rede() {
    for numero in ["\n000123/2026", "000123/2026\n"] {
        let output = ies(&["processo", numero, "--json"]);
        assert_eq!(output.status.code(), Some(2));
        let valor: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(valor["subcodigo"], "numero_processo_controle");
        assert!(!String::from_utf8_lossy(&output.stdout).contains("000123/2026"));
    }
}

#[test]
fn processo_preserva_zeros_e_nao_inventa_leitura() {
    let output = ies(&["processo", "000123/2026-01", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let resposta: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(resposta["codigo"], "nao_implementado");
}

#[test]
fn processo_aceita_protocolos_opacos_de_instalacoes_diferentes() {
    for numero in [
        "ABC-123",
        "550e8400-e29b-41d4-a716-446655440000",
        "CNJ 000001",
        "12345678901234567890123456789012345678901234567890123456789012345678901234567890123",
    ] {
        let output = ies(&["processo", numero, "--json"]);
        assert_eq!(output.status.code(), Some(1), "{numero}");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["codigo"], "nao_implementado", "{numero}");
    }
}

#[test]
fn processo_recusa_referencia_legada_por_tipo() {
    let output = ies(&["processo", "SEI://processo/abc", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "referencia_processo_legada");
}

#[test]
fn esquema_expoe_comandos_reais_sem_conectar_ao_sei() {
    let output = ies(&["esquema", "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let comandos = value["comandos"].as_array().unwrap();
    assert!(comandos.iter().any(|c| c == "processo"));
    assert!(comandos.iter().any(|c| c == "documento"));
    assert!(comandos.iter().any(|c| c == "unidade"));
    assert!(value["versao"].is_string());
}

#[test]
fn documento_rejeita_texto_sem_abrir_sessao() {
    let output = ies(&["documento", "abc", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["codigo"], "entrada_invalida");
    assert_eq!(value["subcodigo"], "numero_documento_nao_numerico");
}

#[test]
fn documento_valida_todos_os_itens_do_lote() {
    let output = ies(&["documento", "00123", "documento-ruim", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "numero_documento_nao_numerico");
}

#[test]
fn status_distingue_configuracao_ausente_de_sei_indisponivel() {
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["status", "--json"])
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["configuracao"], "caminhos_pendentes");
    assert_eq!(value["subcodigo"], "caminho_url_controle_nao_informado");
    assert_eq!(value["conexao"], "nao_verificada");
    assert_eq!(value["acesso_remoto"], false);
}

#[test]
fn status_local_com_caminhos_nao_faz_requisicoes() {
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["status", "--json"])
        .env("IES_URL_CONTROLE_FILE", "url-inexistente")
        .env("IES_COOKIE_FILE", "cookie-inexistente")
        .env("IES_ORIGEM_FILE", "origem-inexistente")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let status: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(status["configuracao"], "caminhos_informados");
    assert_eq!(status["conexao"], "nao_verificada");
    assert_eq!(status["acesso_remoto"], false);
}

#[test]
fn resumo_de_processo_exige_configuracao_sem_tentar_abrir_arvore() {
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["processo", "000123", "--resumo", "--json"])
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let resposta: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(resposta["subcodigo"], "caminho_url_controle_nao_informado");
}

#[test]
fn resumo_e_historico_completo_sao_pedidos_incompativeis() {
    let output = ies(&[
        "processo",
        "000123",
        "--resumo",
        "--andamentos-completos",
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(2));
    let resposta: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(resposta["codigo"], "opcoes_incompativeis");
}

#[test]
fn unidade_atual_sem_sessao_falha_sem_rede() {
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["unidade", "atual", "--json"])
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "caminho_url_controle_nao_informado");
}

#[test]
fn configuracao_explicita_ilegivel_nao_usa_caminhos_de_fallback() {
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["unidade", "atual", "--json"])
        .env("IES_CONFIG_FILE", "/configuracao-ies-inexistente.json")
        .env("IES_URL_CONTROLE_FILE", "url-fallback")
        .env("IES_COOKIE_FILE", "cookie-fallback")
        .env("IES_ORIGEM_FILE", "origem-fallback")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "arquivo_configuracao_nao_pode_ser_lido");
}

#[test]
fn documento_versao_zero_rejeitada_sem_rede() {
    let output = ies(&["documento", "0011519", "--versao", "0", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "versao_documento_invalida");
}

#[test]
fn documento_sem_copia_nao_prepara_arquivo_sem_sessao() {
    let output = ies(&["documento", "0011519", "--sem-copia", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["codigo"], "nao_implementado");
}

#[test]
fn salvar_local_sem_base_nao_envia_escrita_remota() {
    let pasta = std::env::temp_dir().join(format!("ies-salvar-local-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ies"))
        .current_dir(&pasta)
        .args([
            "documento",
            "salvar",
            "0011519",
            "--versao",
            "4",
            "--confirmar",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["subcodigo"], "base_versao_ausente");
    std::fs::remove_dir_all(pasta).unwrap();
}

#[cfg(unix)]
#[test]
fn resumo_de_caixa_local_nao_abre_sessao_e_preserva_nup_opaco() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    let arquivo = std::env::temp_dir().join(format!(
        "ies-caixa-local-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let dados = serde_json::json!({
        "origem": "web", "unidade": "PPI",
        "recebidos": {
            "total_declarado": 1,
            "processos": [{"id": "0042", "numero": "000123/2026", "visualizado": false}]
        },
        "gerados": {"total_declarado": 0, "processos": []}
    });
    fs::write(&arquivo, serde_json::to_vec(&dados).unwrap()).unwrap();
    fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();

    let consultar = |numero: &str| {
        Command::new(env!("CARGO_BIN_EXE_ies"))
            .args(["processo", numero, "--resumo", "--caixa-local"])
            .arg(&arquivo)
            .arg("--json")
            .env("IES_URL_CONTROLE_FILE", "nao-existe-e-nao-deve-ser-usado")
            .env("IES_COOKIE_FILE", "nao-existe-e-nao-deve-ser-usado")
            .env("IES_ORIGEM_FILE", "nao-existe-e-nao-deve-ser-usado")
            .output()
            .unwrap()
    };
    let encontrado = consultar("000123/2026");
    assert_eq!(encontrado.status.code(), Some(0));
    let valor: serde_json::Value = serde_json::from_slice(&encontrado.stdout).unwrap();
    assert_eq!(valor["numero"], "000123/2026");
    assert_eq!(valor["ocorrencias"][0]["processo"]["id"], "0042");
    assert_eq!(valor["fonte"], "arquivo_local");
    assert_eq!(valor["autenticacao_remota"], false);

    let ausente = consultar("123/2026");
    assert_eq!(ausente.status.code(), Some(1));
    let falha: serde_json::Value = serde_json::from_slice(&ausente.stdout).unwrap();
    assert_eq!(
        falha["subcodigo"],
        "numero_nao_observado_no_arquivo_da_caixa"
    );
    fs::remove_file(arquivo).unwrap();
}
