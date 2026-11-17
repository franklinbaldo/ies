//! Interface CLI de cancelar: erros de uso são decididos antes de arquivos,
//! rede ou POST. Números, caminhos e hosts são sintéticos (#273).
use std::process::{Command, Output};

#[path = "support/falha.rs"]
mod falha_cli;
#[path = "support/falha_por.rs"]
mod falha_por_cli;
use falha_por_cli::falha_por;

fn sem_sessao(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .env("IES_INDICE_FILE", "/nao/existe/ies.sqlite3")
        .output()
        .expect("IES deve iniciar")
}
#[test]
fn motivo_obrigatorio_e_exigido_pelo_clap_antes_da_sessao() {
    let resultado = sem_sessao(&["documento", "cancelar", "0000101", "--json"]);
    assert_eq!(resultado.status.code(), Some(2));
    assert_eq!(resultado.stdout, b"");
}

#[test]
fn recusa_numero_invalido_sem_tocar_na_sessao() {
    let r = sem_sessao(&[
        "documento",
        "cancelar",
        "abc",
        "--motivo",
        "Teste",
        "--json",
    ]);
    assert_eq!(r.status.code(), Some(2));
    assert!(
        r.stdout.is_empty(),
        "uso inválido é recusado pelo clap, sem JSON"
    );
}

#[test]
fn motivo_valido_vai_ate_sessao_mas_nao_inventa_configuracao() {
    for confirmar in [false, true] {
        let mut args = vec![
            "documento",
            "cancelar",
            "0000101",
            "--motivo",
            "Teste",
            "--json",
        ];
        if confirmar {
            args.push("--confirmar");
        }
        let r = sem_sessao(&args);
        falha_por(&r, "UrlDoControleIlegivel", "Ausente");
    }
}

#[test]
fn motivo_e_texto_opaco_que_segue_ate_a_sessao() {
    for motivo in ["", "   ", "  Revisão\nnecessária  ", "Cópia 🚀"] {
        for confirmar in [false, true] {
            let mut args = vec![
                "documento",
                "cancelar",
                "0000101",
                "--motivo",
                motivo,
                "--json",
            ];
            if confirmar {
                args.push("--confirmar");
            }
            let saida = sem_sessao(&args);
            falha_por(&saida, "UrlDoControleIlegivel", "Ausente");
        }
    }
}
