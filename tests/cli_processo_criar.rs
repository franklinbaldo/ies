//! A superfície CLI da #245: argumentos, saída 2 antes de acessar arquivos
//! e recusa sem sessão; nenhuma execução consulta ou escreve no SEI.
use std::process::{Command, Output};

#[path = "support/falha.rs"]
mod support;
use support::falha;

fn sem_sessao(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .output()
        .expect("IES deve iniciar")
}

#[test]
fn ajuda_publica_os_argumentos_de_criacao() {
    let saida = sem_sessao(&["processo", "criar", "--help"]);
    assert_eq!(saida.status.code(), Some(0));
    let ajuda = String::from_utf8_lossy(&saida.stdout);
    for flag in [
        "--tipo",
        "--especificacao",
        "--interessado",
        "--nivel",
        "--hipotese-legal",
        "--confirmar",
    ] {
        assert!(ajuda.contains(flag), "faltou {flag}");
    }
}

#[test]
fn argumentos_invalidos_falham_antes_da_sessao() {
    for args in [
        vec![
            "processo",
            "criar",
            "--tipo",
            " ",
            "--especificacao",
            "Teste",
            "--json",
        ],
        vec![
            "processo",
            "criar",
            "--tipo",
            "Ensaio",
            "--especificacao",
            " ",
            "--json",
        ],
    ] {
        let saida = sem_sessao(&args);
        // Erro de uso do clap: saída 2, nada no stdout.
        assert_eq!(saida.status.code(), Some(2), "{args:?}");
        assert_eq!(saida.stdout, b"", "{args:?}");
    }
}

#[test]
fn sem_sessao_recusa_previa_e_confirmacao_sem_rede() {
    for confirmar in [false, true] {
        let mut args = vec![
            "processo",
            "criar",
            "--tipo",
            "Ensaio",
            "--especificacao",
            "Teste de leitura",
            "--json",
        ];
        if confirmar {
            args.push("--confirmar");
        }
        let saida = sem_sessao(&args);
        falha(&saida, "ArquivoPrivadoAusenteOuInacessivel");
    }
}

#[test]
fn hipotese_legal_explicitamente_invalida_eh_recusada_antes_da_sessao() {
    for (nivel, escolha) in [
        ("publico", "31#Base A"),
        ("restrito", "31"),
        ("restrito", "#Base A"),
    ] {
        let saida = sem_sessao(&[
            "processo",
            "criar",
            "--tipo",
            "Ensaio",
            "--especificacao",
            "Teste",
            "--nivel",
            nivel,
            "--hipotese-legal",
            escolha,
            "--json",
        ]);
        assert_eq!(saida.status.code(), Some(2), "{nivel} {escolha}");
        assert_eq!(saida.stdout, b"");
    }
}
