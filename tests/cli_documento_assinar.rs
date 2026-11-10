//! Superfície de `ies documento assinar DOC [--cargo TEXTO] [--confirmar]`
//! (Refs #174): número inválido e `--confirmar` sem `--cargo` saem com 2
//! antes de qualquer arquivo; sem configuração, 1 `fonte_invalida`, sem
//! rede; com `--confirmar`, credenciais ausentes são recusa antes da rede e
//! a senha nunca aparece. Nenhum teste abre sessão nem acessa o SEI.
use std::process::{Command, Output};

fn sem_sessao(args: &[&str], credenciais: Option<&str>) -> Output {
    let mut comando = Command::new(env!("CARGO_BIN_EXE_ies"));
    comando
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .env("IES_INDICE_FILE", "/nao/existe/ies.sqlite3");
    match credenciais {
        Some(caminho) => comando.env("IES_CREDENCIAIS_FILE", caminho),
        None => comando.env_remove("IES_CREDENCIAIS_FILE"),
    };
    comando.output().expect("IES deve iniciar")
}

fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("JSON na saída")
}

#[test]
fn numero_invalido_e_erro_de_uso_antes_da_configuracao() {
    for extra in [&[][..], &["--cargo", "Assessor(a)", "--confirmar"][..]] {
        let mut args = vec!["documento", "assinar", "abc", "--json"];
        args.extend_from_slice(extra);
        let saida = sem_sessao(&args, None);
        assert_eq!(saida.status.code(), Some(2), "{args:?}");
        let valor = json(&saida);
        assert_eq!(valor["codigo"], "entrada_invalida");
        assert_eq!(valor["subcodigo"], "nao_numerico");
    }
}

#[test]
fn confirmar_sem_cargo_e_erro_de_uso_do_clap() {
    let saida = sem_sessao(&["documento", "assinar", "0000101", "--confirmar"], None);
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(saida.stdout, b"");
}

#[test]
fn sem_configuracao_falha_fechado_sem_rede() {
    for extra in [&[][..], &["--cargo", "Assessor(a)", "--confirmar"][..]] {
        let mut args = vec!["documento", "assinar", "0000101", "--json"];
        args.extend_from_slice(extra);
        let saida = sem_sessao(&args, Some("/nao/existe/credenciais"));
        assert_eq!(saida.status.code(), Some(1), "{args:?}");
        let valor = json(&saida);
        assert_eq!(valor["codigo"], "fonte_invalida");
        assert_ne!(valor["codigo"], "nao_implementado");
    }
}

#[test]
fn processo_invalido_e_erro_de_uso() {
    let saida = sem_sessao(
        &[
            "documento",
            "--processo",
            "sei://1",
            "assinar",
            "0000101",
            "--json",
        ],
        None,
    );
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(json(&saida)["codigo"], "entrada_invalida");
}

#[cfg(unix)]
#[test]
fn confirmar_sem_credenciais_e_recusa_antes_da_rede() {
    use std::os::unix::fs::PermissionsExt;
    let pasta = std::env::temp_dir().join(format!("ies-cli-assinar-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    let privado = |nome: &str, texto: &str| {
        let caminho = pasta.join(nome);
        std::fs::write(&caminho, texto).unwrap();
        std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o600)).unwrap();
        caminho
    };
    let url = privado(
        "url",
        "https://sei.exemplo.invalid/sei/controlador.php?acao=procedimento_controlar&infra_hash=aa\n",
    );
    let origem = privado("origem", "https://sei.exemplo.invalid/sei\n");
    let cookie = privado("cookie", "PHPSESSID=SINTETICO\n");
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "documento",
            "assinar",
            "0000101",
            "--cargo",
            "Assessor(a)",
            "--confirmar",
            "--json",
        ])
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE")
        .env("IES_URL_CONTROLE_FILE", &url)
        .env("IES_COOKIE_FILE", &cookie)
        .env("IES_ORIGEM_FILE", &origem)
        .env("IES_INDICE_FILE", pasta.join("ies.sqlite3"))
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .output()
        .expect("IES deve iniciar");
    let _ = std::fs::remove_dir_all(&pasta);
    assert_eq!(saida.status.code(), Some(1));
    let valor = json(&saida);
    assert_eq!(valor["subcodigo"], "caminho_das_credenciais_nao_informado");
    assert!(valor.get("requisicoes").is_none());
}
