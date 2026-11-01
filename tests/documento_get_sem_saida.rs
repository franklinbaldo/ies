//! Sem --saida, nenhum segredo e nenhum GET são necessários para recusar o comando.
use std::process::Command;

#[test]
fn get_opt_in_sem_destino_privado_falha_antes_de_sessao_ou_url() {
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "--json",
            "documento",
            "0007",
            "--url-documento-file",
            "/arquivo-nao-existente-para-teste",
            "--autorizar-efeito-de-leitura",
        ])
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(2));
    let saida: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(saida["codigo"], "entrada_invalida");
    assert_eq!(
        saida["subcodigo"],
        "saida_privada_obrigatoria_para_documento_web"
    );
    assert_eq!(saida["retentavel"], false);
    assert!(saida.get("texto").is_none());
    assert_eq!(resposta.stderr, Vec::<u8>::new());
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("PHPSESSID"));
}


fn executar_get_sem_sessao(destino: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "--json",
            "documento",
            "0007",
            "--url-documento-file",
            "/url-nao-lida-no-teste",
            "--autorizar-efeito-de-leitura",
            "--saida",
            destino,
        ])
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .output()
        .unwrap()
}

#[test]
fn diretorio_de_saida_ausente_recusa_antes_de_abrir_sessao_ou_gerar_get() {
    let destino = std::env::temp_dir().join(format!(
        "ies-destino-inexistente-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(!destino.exists());
    let resposta = executar_get_sem_sessao(destino.to_str().unwrap());
    assert_eq!(resposta.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(
        json["subcodigo"],
        "diretorio_texto_documento_ausente",
        "a falha no destino precisa anteceder qualquer consulta de sessão"
    );
    assert!(!destino.exists());
}

#[test]
fn arquivo_que_ja_existe_recusa_antes_de_abrir_sessao_ou_gerar_get() {
    let destino = std::env::temp_dir().join(format!(
        "ies-destino-ocupado-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&destino).unwrap();
    let anterior = destino.join("0007.txt");
    std::fs::write(&anterior, b"conteudo anterior nao pode ser modificado").unwrap();
    let resposta = executar_get_sem_sessao(destino.to_str().unwrap());
    assert_eq!(resposta.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(
        json["subcodigo"],
        "arquivo_texto_documento_ja_existe",
        "um GET com possivel efeito administrativo nao deve ocorrer para destino ocupado"
    );
    assert_eq!(
        std::fs::read(&anterior).unwrap(),
        b"conteudo anterior nao pode ser modificado"
    );
    std::fs::remove_dir_all(destino).unwrap();
}
